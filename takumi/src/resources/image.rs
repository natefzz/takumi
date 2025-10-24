//! Image resource management for the takumi rendering system.
//!
//! This module provides types and utilities for managing image resources,
//! including loading states, error handling, and image processing operations.

use std::{
  borrow::Cow,
  collections::HashMap,
  sync::{Arc, RwLock},
};

use image::{
  RgbaImage,
  imageops::{FilterType, resize},
};

/// Represents the state of an image resource.
pub type ImageResult = Result<Arc<ImageSource>, ImageResourceError>;

#[derive(Debug, Clone)]
/// Represents the source of an image.
pub enum ImageSource {
  /// An svg image source
  #[cfg(feature = "svg")]
  Svg(Box<resvg::usvg::Tree>),
  /// A bitmap image source
  Bitmap(RgbaImage),
}

/// Represents a persistent image store.
#[derive(Default, Debug)]
pub struct PersistentImageStore(RwLock<HashMap<String, Arc<ImageSource>>>);

impl PersistentImageStore {
  /// Get an image from the store.
  pub fn get(&self, src: &str) -> Option<Arc<ImageSource>> {
    self.0.read().unwrap().get(src).cloned()
  }

  /// Insert an image into the store.
  pub fn insert(&self, src: &str, image: Arc<ImageSource>) {
    self.0.write().unwrap().insert(src.to_string(), image);
  }

  /// Clear the store.
  pub fn clear(&self) {
    self.0.write().unwrap().clear();
  }
}

impl From<RgbaImage> for ImageSource {
  fn from(bitmap: RgbaImage) -> Self {
    ImageSource::Bitmap(bitmap)
  }
}

impl ImageSource {
  /// Get the size of the image source.
  pub fn size(&self) -> (f32, f32) {
    match self {
      #[cfg(feature = "svg")]
      ImageSource::Svg(svg) => (svg.size().width(), svg.size().height()),
      ImageSource::Bitmap(bitmap) => (bitmap.width() as f32, bitmap.height() as f32),
    }
  }

  /// Render the image source to an RGBA image with the specified dimensions.
  pub fn render_to_rgba_image<'i>(
    &'i self,
    width: u32,
    height: u32,
    filter_type: FilterType,
  ) -> Cow<'i, RgbaImage> {
    match self {
      ImageSource::Bitmap(bitmap) => {
        if bitmap.width() == width && bitmap.height() == height {
          return Cow::Borrowed(bitmap);
        }

        Cow::Owned(resize(bitmap, width, height, filter_type))
      }
      #[cfg(feature = "svg")]
      ImageSource::Svg(svg) => {
        use resvg::{tiny_skia::Pixmap, usvg::Transform};

        let mut pixmap = Pixmap::new(width, height).unwrap();

        let original_size = svg.size();
        let sx = width as f32 / original_size.width();
        let sy = height as f32 / original_size.height();

        resvg::render(svg, Transform::from_scale(sx, sy), &mut pixmap.as_mut());

        Cow::Owned(RgbaImage::from_raw(width, height, pixmap.take()).unwrap())
      }
    }
  }
}

/// Try to load an image source from raw bytes.
///
/// - When the `svg` feature is enabled and the bytes look like SVG XML, they
///   are parsed as an SVG using `resvg::usvg`.
/// - Otherwise, the bytes are decoded as a raster image using the `image` crate.
pub fn load_image_source_from_bytes(bytes: &[u8]) -> ImageResult {
  #[cfg(feature = "svg")]
  {
    use std::str::from_utf8;

    if let Ok(text) = from_utf8(bytes) {
      if is_svg(text) {
        return parse_svg(text);
      }
    }
  }

  let img = image::load_from_memory(bytes).map_err(ImageResourceError::DecodeError)?;
  Ok(Arc::new(img.into_rgba8().into()))
}

/// Check if the bytes are an SVG image.
pub(crate) fn is_svg(src: &str) -> bool {
  src.trim_start().starts_with("<svg") && src.contains("xmlns=\"http://www.w3.org/2000/svg\"")
}

#[cfg(feature = "svg")]
pub(crate) fn parse_svg(src: &str) -> ImageResult {
  let tree = resvg::usvg::Tree::from_str(src, &resvg::usvg::Options::default())
    .map_err(ImageResourceError::SvgParseError)?;

  Ok(Arc::new(ImageSource::Svg(Box::new(tree))))
}

/// Represents the state of an image in the rendering system.
///
/// This enum tracks whether an image has been successfully loaded and decoded,
/// or if there was an error during the process.
#[derive(Debug)]
pub enum ImageResourceError {
  /// An error occurred while decoding the image data
  DecodeError(image::ImageError),
  /// The image data URI is in an invalid format
  InvalidDataUriFormat,
  /// The image data URI is malformed and cannot be parsed
  MalformedDataUri,
  #[cfg(feature = "svg")]
  /// An error occurred while parsing an SVG image
  SvgParseError(resvg::usvg::Error),
  /// SVG parsing is not supported in this build
  #[cfg(not(feature = "svg"))]
  SvgParseNotSupported,
  /// The image source is unknown
  Unknown,
}
