mod load_font_task;
mod put_persistent_image_task;
mod render_animation_task;
mod render_task;
mod renderer;

use napi::bindgen_prelude::{Buffer, BufferSlice};
use napi_derive::napi;
pub use renderer::Renderer;
use takumi::parley::FontStyle;

#[napi(object)]
pub(crate) struct FontInput<'ctx> {
  pub name: Option<String>,
  pub data: BufferSlice<'ctx>,
  pub weight: Option<f64>,
  pub style: Option<FontStyleInput>,
}

#[napi(object)]
pub(crate) struct FontInputOwned {
  pub name: Option<String>,
  pub data: Buffer,
  pub weight: Option<f64>,
  pub style: Option<FontStyleInput>,
}

#[napi(string_enum)]
#[allow(non_camel_case_types)]
#[derive(Clone, Copy)]
pub enum FontStyleInput {
  normal,
  italic,
  oblique,
}

impl From<FontStyleInput> for FontStyle {
  fn from(value: FontStyleInput) -> Self {
    match value {
      FontStyleInput::normal => FontStyle::Normal,
      FontStyleInput::italic => FontStyle::Italic,
      FontStyleInput::oblique => FontStyle::Oblique(None),
    }
  }
}
