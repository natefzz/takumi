/// Background and color drawing functions
mod background_drawing;
/// Canvas operations and image blending
mod canvas;
mod components;
/// Debug drawing utilities
mod debug_drawing;
/// Image drawing functions
mod image_drawing;
/// Main image renderer and viewport management
mod render;
/// Text drawing functions
mod text_drawing;

pub(crate) use background_drawing::*;
pub(crate) use canvas::*;
pub(crate) use components::*;
pub(crate) use debug_drawing::*;
pub(crate) use image_drawing::*;
pub use render::*;
pub(crate) use text_drawing::*;

use crate::{
  GlobalContext,
  layout::{
    Viewport,
    style::{Affine, InheritedStyle},
  },
};

/// The context for the image renderer.
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
  /// The style after inheritance.
  pub(crate) style: InheritedStyle,
}
