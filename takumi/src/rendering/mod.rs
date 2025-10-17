/// Background and color drawing functions
mod background_drawing;
/// Canvas operations and image blending
mod canvas;
mod components;
/// Debug drawing utilities
mod debug_drawing;
/// Image drawing functions
mod image_drawing;
pub(crate) mod inline_drawing;
/// Main image renderer and viewport management
mod render;
/// Text drawing functions
mod text_drawing;
mod write;

pub(crate) use background_drawing::*;
pub(crate) use canvas::*;
pub(crate) use components::*;
pub(crate) use debug_drawing::*;
pub(crate) use image_drawing::*;
pub use render::*;
pub(crate) use text_drawing::*;
pub use write::*;

use crate::{
  GlobalContext,
  layout::{
    Viewport,
    node::Node,
    style::{Affine, Color, InheritedStyle},
  },
};

/// The context for the internal rendering. You should not construct this directly.
#[derive(Clone)]
pub struct RenderContext<'g> {
  /// The global context.
  pub(crate) global: &'g GlobalContext,
  /// The viewport for the image renderer.
  pub(crate) viewport: Viewport,
  /// The font size in pixels.
  pub(crate) font_size: f32,
  /// The scale factor for the image renderer.
  pub(crate) transform: Affine,
  /// What the `currentColor` value is resolved to.
  pub(crate) current_color: Color,
  /// The opacity to apply to all colors.
  pub(crate) opacity: f32,
  /// The style after inheritance.
  pub(crate) style: InheritedStyle,
  /// Whether to draw debug borders.
  pub(crate) draw_debug_border: bool,
}

impl<'g> RenderContext<'g> {
  pub(crate) fn new(global: &'g GlobalContext, viewport: Viewport) -> Self {
    Self {
      global,
      viewport,
      font_size: viewport.font_size,
      transform: Affine::identity(),
      current_color: Color::black(),
      opacity: 1.0,
      style: InheritedStyle::default(),
      draw_debug_border: false,
    }
  }
}

impl<'g, N: Node<N>> From<&RenderOptions<'g, N>> for RenderContext<'g> {
  fn from(options: &RenderOptions<'g, N>) -> Self {
    let mut context = RenderContext::new(options.global, options.viewport);

    context.draw_debug_border = options.draw_debug_border;

    context
  }
}
