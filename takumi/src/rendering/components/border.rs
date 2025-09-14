use std::f32::consts::SQRT_2;

use taffy::{Layout, Point, Size};
use zeno::{Command, Fill, Mask, PathBuilder};

use crate::{
  layout::style::{Affine, Color, LengthUnit, Sides},
  rendering::{Canvas, RenderContext},
};

fn resolve_border_radius_from_percentage_css(
  context: &RenderContext,
  radius: LengthUnit,
  reference_size: f32,
) -> f32 {
  radius
    .resolve_to_px(context, reference_size)
    .min(reference_size / 2.0)
}

/// Represents the properties of a border, including corner radii and drawing metadata.
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct BorderProperties {
  /// The width of the border on each side (top, right, bottom, left)
  pub width: taffy::Rect<f32>,
  /// The offset of the border in the local coordinate space
  pub offset: Point<f32>,
  /// The size of the border area
  pub size: Size<f32>,
  /// The color of the border
  pub color: Color,
  /// Corner radii: top, right, bottom, left (in pixels)
  pub radius: Sides<f32>,
  /// The transform applied when drawing this border
  pub transform: Affine,
}

impl BorderProperties {
  /// Create an empty BorderProperties with zeroed radii and default values.
  pub const fn zero() -> Self {
    Self {
      width: taffy::Rect::ZERO,
      offset: Point::ZERO,
      size: Size::ZERO,
      color: Color([0, 0, 0, 255]),
      radius: Sides([0.0; 4]),
      transform: Affine::identity(),
    }
  }

  /// Resolves the border radius from the context and layout.
  pub fn from_context(context: &RenderContext, layout: &Layout) -> Self {
    let resolved = context.style.resolved_border_radius();
    let reference_size = layout.size.width.min(layout.size.height);

    let top_left = resolve_border_radius_from_percentage_css(context, resolved.top, reference_size);
    let top_right =
      resolve_border_radius_from_percentage_css(context, resolved.right, reference_size);
    let bottom_right =
      resolve_border_radius_from_percentage_css(context, resolved.bottom, reference_size);
    let bottom_left =
      resolve_border_radius_from_percentage_css(context, resolved.left, reference_size);

    Self {
      width: layout.border,
      offset: Point::ZERO,
      size: layout.size,
      color: context
        .style
        .border_color
        .or(context.style.border.color)
        .unwrap_or(Color::black()),
      radius: Sides([top_left, top_right, bottom_right, bottom_left]),
      transform: context.transform,
    }
  }

  /// Returns true if all corner radii are zero.
  #[inline]
  pub fn is_zero(&self) -> bool {
    self.radius.0[0] == 0.0
      && self.radius.0[1] == 0.0
      && self.radius.0[2] == 0.0
      && self.radius.0[3] == 0.0
  }

  /// Expand/shrink all corner radii and adjust radius bounds/offset.
  pub fn expand_by(&self, amount: f32) -> Self {
    Self {
      width: self.width,
      offset: Point {
        x: self.offset.x - amount,
        y: self.offset.y - amount,
      },
      size: Size {
        width: (self.size.width + amount * 2.0).max(0.0),
        height: (self.size.height + amount * 2.0).max(0.0),
      },
      color: self.color,
      radius: Sides([
        (self.radius.0[0] + amount).max(0.0),
        (self.radius.0[1] + amount).max(0.0),
        (self.radius.0[2] + amount).max(0.0),
        (self.radius.0[3] + amount).max(0.0),
      ]),
      transform: self.transform,
    }
  }

  /// Shrink radii by average border width to get inner radius path.
  pub fn inset_by_border_width(&self) -> Self {
    let avg_width = (self.width.top + self.width.right + self.width.bottom + self.width.left) / 4.0;
    self.expand_by(-avg_width)
  }

  /// Append rounded-rect path commands for this border's corner radii.
  pub fn append_mask_commands(&self, path: &mut Vec<Command>) {
    const KAPPA: f32 = 4.0 / 3.0 * (SQRT_2 - 1.0);

    let top_edge_width = (self.size.width - self.radius.0[0] - self.radius.0[1]).max(0.0);
    let right_edge_height = (self.size.height - self.radius.0[1] - self.radius.0[2]).max(0.0);
    let bottom_edge_width = (self.size.width - self.radius.0[3] - self.radius.0[2]).max(0.0);
    let left_edge_height = (self.size.height - self.radius.0[3] - self.radius.0[0]).max(0.0);

    path.move_to((self.offset.x + self.radius.0[0], self.offset.y));

    if top_edge_width > 0.0 {
      path.rel_line_to((top_edge_width, 0.0));
    }

    if self.radius.0[1] > 0.0 {
      let control_offset = self.radius.0[1] * KAPPA;
      path.rel_curve_to(
        (control_offset, 0.0),
        (self.radius.0[1], self.radius.0[1] - control_offset),
        (self.radius.0[1], self.radius.0[1]),
      );
    }

    if right_edge_height > 0.0 {
      path.rel_line_to((0.0, right_edge_height));
    }

    if self.radius.0[2] > 0.0 {
      let control_offset = self.radius.0[2] * KAPPA;
      path.rel_curve_to(
        (0.0, control_offset),
        (-self.radius.0[2] + control_offset, self.radius.0[2]),
        (-self.radius.0[2], self.radius.0[2]),
      );
    }

    if bottom_edge_width > 0.0 {
      path.rel_line_to((-bottom_edge_width, 0.0));
    }

    if self.radius.0[3] > 0.0 {
      let control_offset = self.radius.0[3] * KAPPA;
      path.rel_curve_to(
        (-control_offset, 0.0),
        (-self.radius.0[3], -self.radius.0[3] + control_offset),
        (-self.radius.0[3], -self.radius.0[3]),
      );
    }

    if left_edge_height > 0.0 {
      path.rel_line_to((0.0, -left_edge_height));
    }

    if self.radius.0[0] > 0.0 {
      let control_offset = self.radius.0[0] * KAPPA;
      path.rel_curve_to(
        (0.0, -control_offset),
        (self.radius.0[0] - control_offset, -self.radius.0[0]),
        (self.radius.0[0], -self.radius.0[0]),
      );
    }

    path.close();
  }
}

// duplicate/old BorderProperties removed; canonical `BorderProperties` defined above.

/// Draws borders around the node with optional border radius.
///
/// This function draws borders with specified size and color. If border_radius is specified,
/// it creates a rounded border using a custom drawing approach.
pub(crate) fn draw_border(canvas: &Canvas, canvas_offset: Point<f32>, border: BorderProperties) {
  if border.width.left == 0.0
    && border.width.right == 0.0
    && border.width.top == 0.0
    && border.width.bottom == 0.0
  {
    return;
  }

  let mut paths = Vec::new();

  border.append_mask_commands(&mut paths);

  border
    .inset_by_border_width()
    .append_mask_commands(&mut paths);

  border.transform.apply_on_paths(&mut paths);

  let (mask, mut placement) = Mask::new(&paths).style(Fill::EvenOdd).render();

  placement.left += border.offset.x as i32 + canvas_offset.x as i32;
  placement.top += border.offset.y as i32 + canvas_offset.y as i32;

  canvas.draw_mask(mask, placement, border.color, None);
}
