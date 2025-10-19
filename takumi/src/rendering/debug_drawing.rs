use taffy::{Layout, Point};

use crate::{
  layout::style::{Affine, Color, Sides},
  rendering::{BorderProperties, Canvas, draw_border},
};

/// Draws debug borders around the node's layout areas.
///
/// This function draws colored rectangles to visualize the content box
/// (red) and the full layout box (green) for debugging purposes.
pub fn draw_debug_border(canvas: &Canvas, layout: Layout, transform: Affine) {
  let x = layout.content_box_x();
  let y = layout.content_box_y();

  // border-box
  draw_border(
    canvas,
    layout.location,
    BorderProperties {
      width: Sides([1.0; 4]).into(),
      offset: Point::ZERO,
      size: layout.size,
      color: Color([255, 0, 0, 255]), // red
      radius: Sides([0.0; 4]),
      transform,
    },
  );

  // content-box
  draw_border(
    canvas,
    Point { x, y },
    BorderProperties {
      width: Sides([1.0; 4]).into(),
      offset: Point::ZERO,
      size: layout.content_box_size(),
      color: Color([0, 255, 0, 255]), // green
      radius: Sides([0.0; 4]),
      transform,
    },
  );
}
