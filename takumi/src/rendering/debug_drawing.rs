use taffy::{Layout, Point};

use crate::{
  layout::style::{Affine, Color, Sides},
  rendering::{BorderProperties, Canvas, draw_border},
};

/// Draws debug borders around the node's layout areas.
///
/// This function draws colored rectangles to visualize the content box
/// (red) and the full layout box (green) for debugging purposes.
pub fn draw_debug_border(canvas: &mut Canvas, layout: Layout, transform: Affine) {
  // border-box
  draw_border(
    canvas,
    BorderProperties {
      width: Sides([1.0; 4]).into(),
      size: layout.size,
      color: Color([255, 0, 0, 255]), // red
      radius: Sides([0.0; 4]),
      offset: Point::ZERO,
    },
    transform,
  );

  // content-box
  draw_border(
    canvas,
    BorderProperties {
      width: Sides([1.0; 4]).into(),
      size: layout.content_box_size(),
      color: Color([0, 255, 0, 255]), // green
      radius: Sides([0.0; 4]),
      offset: Point {
        x: layout.content_box_x(),
        y: layout.content_box_y(),
      },
    },
    transform,
  );
}
