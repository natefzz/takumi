use std::io::Cursor;

use napi::bindgen_prelude::*;
use std::sync::Arc;
use takumi::{
  GlobalContext,
  layout::{Viewport, node::NodeKind},
  rendering::{RenderOptionsBuilder, render, write_image},
};

use crate::renderer::OutputFormat;

pub struct RenderTask {
  pub draw_debug_border: bool,
  pub node: Option<NodeKind>,
  pub context: Arc<GlobalContext>,
  pub viewport: Viewport,
  pub format: OutputFormat,
  pub quality: Option<u8>,
}

impl Task for RenderTask {
  type Output = Vec<u8>;
  type JsValue = Buffer;

  fn compute(&mut self) -> Result<Self::Output> {
    let node = self.node.take().unwrap();

    let image = render(
      RenderOptionsBuilder::default()
        .viewport(self.viewport)
        .node(node)
        .global(&self.context)
        .draw_debug_border(self.draw_debug_border)
        .build()
        .unwrap(),
    )
    .map_err(|e| napi::Error::from_reason(format!("Failed to render: {e:?}")))?;

    if self.format == OutputFormat::raw {
      return Ok(image.into_raw());
    }

    let mut buffer = Vec::new();
    let mut cursor = Cursor::new(&mut buffer);

    write_image(&image, &mut cursor, self.format.into(), self.quality)
      .map_err(|e| napi::Error::from_reason(format!("Failed to write to buffer: {e:?}")))?;

    Ok(buffer)
  }

  fn resolve(&mut self, _env: Env, output: Self::Output) -> Result<Self::JsValue> {
    Ok(output.into())
  }
}
