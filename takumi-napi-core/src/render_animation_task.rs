use napi::bindgen_prelude::*;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use std::io::Cursor;
use takumi::{
  GlobalContext,
  layout::{Viewport, node::NodeKind},
  rendering::{
    AnimationFrame, RenderOptionsBuilder, encode_animated_png, encode_animated_webp, render,
  },
};

use crate::renderer::AnimationOutputFormat;

pub struct RenderAnimationTask<'g> {
  pub nodes: Option<Vec<(NodeKind, u32)>>,
  pub context: &'g GlobalContext,
  pub viewport: Viewport,
  pub format: AnimationOutputFormat,
  pub draw_debug_border: bool,
}

impl Task for RenderAnimationTask<'_> {
  type Output = Vec<u8>;
  type JsValue = Buffer;

  fn compute(&mut self) -> Result<Self::Output> {
    let nodes = self.nodes.take().unwrap();

    let frames: Vec<_> = nodes
      .into_par_iter()
      .map(|(node, duration_ms)| {
        AnimationFrame::new(
          render(
            RenderOptionsBuilder::default()
              .viewport(self.viewport)
              .node(node)
              .global(self.context)
              .draw_debug_border(self.draw_debug_border)
              .build()
              .unwrap(),
          )
          .unwrap(),
          duration_ms,
        )
      })
      .collect();

    let mut buffer = Vec::new();
    let mut cursor = Cursor::new(&mut buffer);

    match self.format {
      AnimationOutputFormat::webp => {
        encode_animated_webp(&frames, &mut cursor, true, false, None)
          .map_err(|e| napi::Error::from_reason(format!("Failed to write to buffer: {e:?}")))?;
      }
      AnimationOutputFormat::apng => {
        encode_animated_png(&frames, &mut cursor, None)
          .map_err(|e| napi::Error::from_reason(format!("Failed to write to buffer: {e:?}")))?;
      }
    }

    Ok(buffer)
  }

  fn resolve(&mut self, _env: Env, output: Self::Output) -> Result<Self::JsValue> {
    Ok(output.into())
  }
}
