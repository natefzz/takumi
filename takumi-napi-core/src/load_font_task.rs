use napi::bindgen_prelude::*;
use takumi::{
  GlobalContext,
  parley::{FontWeight, fontique::FontInfoOverride},
};

use crate::FontInputOwned;

pub struct LoadFontTask<'g> {
  pub context: &'g mut GlobalContext,
  pub(crate) buffers: Vec<FontInputOwned>,
}

impl Task for LoadFontTask<'_> {
  type Output = usize;
  type JsValue = u32;

  fn compute(&mut self) -> Result<Self::Output> {
    if self.buffers.is_empty() {
      return Ok(0);
    }

    let mut loaded_count = 0;

    for buffer in &self.buffers {
      if self
        .context
        .font_context
        .load_and_store(
          &buffer.data,
          Some(FontInfoOverride {
            family_name: buffer.name.as_deref(),
            width: None,
            style: buffer.style.map(Into::into),
            weight: buffer.weight.map(|weight| FontWeight::new(weight as f32)),
            axes: None,
          }),
          None,
        )
        .is_ok()
      {
        loaded_count += 1;
      }
    }

    Ok(loaded_count)
  }

  fn resolve(&mut self, _env: Env, output: Self::Output) -> Result<Self::JsValue> {
    Ok(output as u32)
  }
}
