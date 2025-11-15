//! Canvas operations and image blending for the takumi rendering system.
//!
//! This module provides performance-optimized canvas operations including
//! fast image blending and pixel manipulation operations.

use std::{borrow::Cow, ops::Neg};

use image::{
  Pixel, Rgba, RgbaImage,
  imageops::{interpolate_bilinear, interpolate_nearest},
};
use taffy::{Point, Size};
use zeno::{Mask, Placement};

use crate::{
  layout::style::{Affine, Color, Filters, ImageScalingAlgorithm},
  rendering::BorderProperties,
};

/// A canvas handle for sending drawing commands asynchronously.
///
/// This struct wraps a channel sender that can be cloned and used to send
/// drawing commands to a canvas rendering loop without blocking the main thread.
pub struct Canvas {
  image: RgbaImage,
  offset: Point<f32>,
}

impl Canvas {
  /// Creates a new canvas handle from a draw command sender.
  pub(crate) fn new(size: Size<u32>) -> Self {
    Self {
      image: RgbaImage::new(size.width, size.height),
      offset: Point::zero(),
    }
  }

  pub(crate) fn into_inner(self) -> RgbaImage {
    self.image
  }

  pub(crate) fn add_offset(&mut self, offset: Point<f32>) {
    self.offset = self.offset + offset;
  }

  /// Overlays an image onto the canvas with optional border radius.
  pub(crate) fn overlay_image(
    &mut self,
    image: &RgbaImage,
    border: BorderProperties,
    transform: Affine,
    algorithm: ImageScalingAlgorithm,
    filters: Option<&Filters>,
  ) {
    if image.is_empty() {
      return;
    }

    overlay_image(
      &mut self.image,
      image,
      border,
      transform,
      algorithm,
      filters,
      self.offset,
    );
  }

  /// Draws a mask with the specified color onto the canvas.
  pub(crate) fn draw_mask(
    &mut self,
    mask: &[u8],
    mut placement: Placement,
    color: Color,
    image: Option<RgbaImage>,
  ) {
    if mask.is_empty() {
      return;
    }

    placement.left += self.offset.x as i32;
    placement.top += self.offset.y as i32;

    draw_mask(&mut self.image, mask, placement, color, image.as_ref());
  }

  /// Fills a rectangular area with the specified color and optional border radius.
  pub(crate) fn fill_color(
    &mut self,
    size: Size<u32>,
    color: Color,
    border: BorderProperties,
    transform: Affine,
  ) {
    if color.0[3] == 0 {
      return;
    }

    // Fast path: if drawing on the entire canvas, we can just replace the entire canvas with the color
    if transform.is_identity()
      && border.is_zero()
      && color.0[3] == 255
      && self.offset.x == 0.0
      && self.offset.y == 0.0
      && size.width == self.image.width()
      && size.height == self.image.height()
    {
      let image_mut = self.image.as_mut();

      for chunk in image_mut.chunks_exact_mut(4) {
        chunk.copy_from_slice(&color.0);
      }

      return;
    }

    let can_direct_draw = transform.only_translation() && border.is_zero();

    // Fast path: if no sub-pixel interpolation is needed, we can just draw the color directly
    if can_direct_draw {
      let transformed_offset = transform.decompose_translation() + self.offset;

      let color: Rgba<u8> = color.into();
      return overlay_area(&mut self.image, transformed_offset, size, |_, _| color);
    }

    let mut paths = Vec::new();

    border.append_mask_commands(&mut paths);

    let (mask, placement) = Mask::new(&paths).transform(Some(*transform)).render();

    self.draw_mask(&mask, placement, color, None);
  }
}

/// Draws a single pixel on the canvas with alpha blending.
///
/// If the color is fully transparent (alpha = 0), no operation is performed.
/// Otherwise, the pixel is blended with the existing canvas pixel using alpha blending.
#[inline]
pub(crate) fn draw_pixel(canvas: &mut RgbaImage, x: u32, y: u32, color: Rgba<u8>) {
  if color.0[3] == 0 {
    return;
  }

  // image-rs blend will skip the operation if the source color is fully transparent
  if let Some(pixel) = canvas.get_pixel_mut_checked(x, y) {
    if pixel.0[3] == 0 {
      // If the destination pixel is fully transparent, we directly assign the new color.
      // This is a performance optimization: blending with a fully transparent pixel is
      // equivalent to assignment, so we skip the blend operation. This deviates from the
      // standard alpha blending approach for efficiency.
      *pixel = color;
    } else {
      pixel.blend(&color);
    }
  }
}

pub(crate) fn apply_mask_alpha_to_pixel(mut pixel: Rgba<u8>, alpha: u8) -> Rgba<u8> {
  if alpha == u8::MAX {
    pixel
  } else {
    pixel.0[3] = ((pixel.0[3] as f32) * (alpha as f32 / 255.0)).round() as u8;

    pixel
  }
}

pub(crate) fn draw_mask<C: Into<Rgba<u8>>>(
  canvas: &mut RgbaImage,
  mask: &[u8],
  placement: Placement,
  color: C,
  image: Option<&RgbaImage>,
) {
  let offset = Point {
    x: placement.left as f32,
    y: placement.top as f32,
  };
  let top_size = Size {
    width: placement.width,
    height: placement.height,
  };

  let color = color.into();

  overlay_area(canvas, offset, top_size, |x, y| {
    let alpha = mask[mask_index_from_coord(x, y, placement.width)];

    if alpha == 0 {
      return Color::transparent().into();
    }

    let pixel = image.map(|image| *image.get_pixel(x, y)).unwrap_or(color);

    apply_mask_alpha_to_pixel(pixel, alpha)
  });
}

pub(crate) fn overlay_image(
  canvas: &mut RgbaImage,
  image: &RgbaImage,
  border: BorderProperties,
  transform: Affine,
  algorithm: ImageScalingAlgorithm,
  filters: Option<&Filters>,
  offset: Point<f32>,
) {
  let can_direct_draw = transform.only_translation() && border.is_zero();

  let mut image = Cow::Borrowed(image);

  if let Some(filters) = filters
    && !filters.0.is_empty()
  {
    let mut owned_image = image.into_owned();

    filters.apply_to(&mut owned_image);

    image = Cow::Owned(owned_image);
  }

  if can_direct_draw {
    let transformed_offset = transform.decompose_translation() + offset;

    return overlay_area(
      canvas,
      transformed_offset,
      Size {
        width: image.width(),
        height: image.height(),
      },
      |x, y| *image.get_pixel(x, y),
    );
  }

  let Some(inverse) = transform.invert() else {
    return;
  };

  let mut paths = Vec::new();

  border.append_mask_commands(&mut paths);

  let (mask, placement) = Mask::new(&paths).transform(Some(*transform)).render();

  let get_original_pixel = |x, y| {
    let alpha = mask[mask_index_from_coord(x, y, image.width())];

    if alpha == 0 {
      return Color::transparent().into();
    }

    let point = inverse.transform_point(
      (
        x as f32 + placement.left as f32,
        y as f32 + placement.top as f32,
      )
        .into(),
    );

    let sampled_pixel = match algorithm {
      ImageScalingAlgorithm::Pixelated => interpolate_nearest(&*image, point.x, point.y),
      _ => interpolate_bilinear(&*image, point.x, point.y),
    };

    let Some(mut pixel) = sampled_pixel else {
      return Color::transparent().into();
    };

    if alpha != u8::MAX {
      pixel = apply_mask_alpha_to_pixel(pixel, alpha);
    }

    pixel
  };

  overlay_area(
    canvas,
    offset,
    Size {
      width: image.width(),
      height: image.height(),
    },
    get_original_pixel,
  );
}

#[inline(always)]
pub(crate) fn mask_index_from_coord(x: u32, y: u32, width: u32) -> usize {
  (y * width + x) as usize
}

pub(crate) fn overlay_area(
  bottom: &mut RgbaImage,
  offset: Point<f32>,
  top_size: Size<u32>,
  f: impl Fn(u32, u32) -> Rgba<u8>,
) {
  let top_start_x = offset.x.neg().max(0.0) as u32;
  let top_start_y = offset.y.neg().max(0.0) as u32;

  let top_steps_x = top_size
    .width
    .min((bottom.width() as f32 - offset.x) as u32);
  let top_steps_y = top_size
    .height
    .min((bottom.height() as f32 - offset.y) as u32);

  for y in top_start_y..top_steps_y {
    for x in top_start_x..top_steps_x {
      let pixel = f(x, y);

      let dest_x = x + offset.x as u32;
      let dest_y = y + offset.y as u32;

      draw_pixel(bottom, dest_x, dest_y, pixel);
    }
  }
}
