use std::f32::consts::SQRT_2;

use taffy::{Point, Rect, Size};
use zeno::{Command, Fill, Mask, PathBuilder};

use crate::{
  layout::style::{Affine, Color, ColorInput, LengthUnit, Sides},
  rendering::{Canvas, RenderContext},
};

fn resolve_border_radius_from_percentage_css<const DEFAULT_AUTO: bool>(
  context: &RenderContext,
  radius: LengthUnit<DEFAULT_AUTO>,
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
  pub width: Rect<f32>,
  /// The color of the border
  pub color: Color,
  /// Corner radii: top, right, bottom, left (in pixels)
  pub radius: Sides<f32>,
}

impl BorderProperties {
  /// The amount of path commands to append for this border.
  /// This is used to pre-allocate the vector size for the mask commands.
  const PATH_COMMANDS_AMOUNT: usize = 10;

  /// Create an empty BorderProperties with zeroed radii and default values.
  pub const fn zero() -> Self {
    Self {
      width: Rect::ZERO,
      color: Color([0, 0, 0, 255]),
      radius: Sides([0.0; 4]),
    }
  }

  /// Resolves the border radius from the context and layout.
  pub fn from_context(
    context: &RenderContext,
    border_box: Size<f32>,
    border_width: Rect<f32>,
  ) -> Self {
    let resolved = context.style.resolved_border_radius();
    let reference_size = border_box.width.min(border_box.height);

    let top_left = resolve_border_radius_from_percentage_css(context, resolved.top, reference_size);
    let top_right =
      resolve_border_radius_from_percentage_css(context, resolved.right, reference_size);
    let bottom_right =
      resolve_border_radius_from_percentage_css(context, resolved.bottom, reference_size);
    let bottom_left =
      resolve_border_radius_from_percentage_css(context, resolved.left, reference_size);

    Self {
      width: border_width,
      color: context
        .style
        .border_color
        .or(context.style.border.color)
        .unwrap_or(ColorInput::CurrentColor)
        .resolve(context.current_color, context.opacity),
      radius: Sides([top_left, top_right, bottom_right, bottom_left]),
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
      color: self.color,
      radius: Sides([
        (self.radius.0[0] + amount).max(0.0),
        (self.radius.0[1] + amount).max(0.0),
        (self.radius.0[2] + amount).max(0.0),
        (self.radius.0[3] + amount).max(0.0),
      ]),
    }
  }

  /// Shrink radii by average border width to get inner radius path.
  pub fn inset_by_border_width(&self) -> Self {
    let avg_width = (self.width.top + self.width.right + self.width.bottom + self.width.left) / 4.0;
    self.expand_by(-avg_width)
  }

  /// Append rounded-rect path commands for this border's corner radii.
  pub fn append_mask_commands(
    &self,
    path: &mut Vec<Command>,
    border_box: Size<f32>,
    offset: Point<f32>,
  ) {
    path.reserve_exact(BorderProperties::PATH_COMMANDS_AMOUNT);

    const KAPPA: f32 = 4.0 / 3.0 * (SQRT_2 - 1.0);

    let top_edge_width = (border_box.width - self.radius.0[0] - self.radius.0[1]).max(0.0);
    let right_edge_height = (border_box.height - self.radius.0[1] - self.radius.0[2]).max(0.0);
    let bottom_edge_width = (border_box.width - self.radius.0[3] - self.radius.0[2]).max(0.0);
    let left_edge_height = (border_box.height - self.radius.0[3] - self.radius.0[0]).max(0.0);

    path.move_to((offset.x + self.radius.0[0], offset.y));

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

  pub(crate) fn draw(&self, canvas: &mut Canvas, border_box: Size<f32>, transform: Affine) {
    if self.width.left == 0.0
      && self.width.right == 0.0
      && self.width.top == 0.0
      && self.width.bottom == 0.0
    {
      return;
    }

    let mut paths = Vec::with_capacity(BorderProperties::PATH_COMMANDS_AMOUNT * 2);

    self.append_mask_commands(&mut paths, border_box, Point::ZERO);

    let avg_width = (self.width.top + self.width.right + self.width.bottom + self.width.left) / 4.0;

    self.expand_by(-avg_width).append_mask_commands(
      &mut paths,
      border_box
        - Size {
          width: avg_width * 2.0,
          height: avg_width * 2.0,
        },
      Point {
        x: avg_width,
        y: avg_width,
      },
    );

    let (mask, placement) = Mask::new(&paths)
      .style(Fill::EvenOdd)
      .transform(Some(transform.into()))
      .render();

    canvas.draw_mask(&mask, placement, self.color, None);
  }
}
