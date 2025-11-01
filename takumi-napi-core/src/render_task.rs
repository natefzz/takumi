use std::{collections::HashMap, io::Cursor};

use crossbeam_channel::{Receiver, bounded};
use napi::bindgen_prelude::*;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use std::sync::Arc;
use takumi::{
  GlobalContext,
  layout::{
    Viewport,
    node::{Node, NodeKind},
  },
  rendering::{RenderOptionsBuilder, render, write_image},
  resources::{
    image::{ImageSource, load_image_source_from_bytes},
    task::{FetchTask, FetchTaskCollection},
  },
};

use crate::{
  ArrayBufferFn, MaybeInitialized,
  renderer::{OutputFormat, RenderOptions, ResourceCache},
};

pub struct RenderTask<'g> {
  pub draw_debug_border: bool,
  pub node: Option<NodeKind>,
  pub global: &'g GlobalContext,
  pub viewport: Viewport,
  pub format: OutputFormat,
  pub quality: Option<u8>,
  pub(crate) resource_cache: ResourceCache,
  pub(crate) tasks_rx: Receiver<(FetchTask, MaybeInitialized<Buffer, Arc<ImageSource>>)>,
}

impl<'g> RenderTask<'g> {
  pub fn from_options(
    env: Env,
    node: NodeKind,
    options: RenderOptions,
    resources_cache: &ResourceCache,
    global: &'g GlobalContext,
  ) -> Result<Self> {
    let mut collection = FetchTaskCollection::default();

    node.collect_fetch_tasks(&mut collection);
    node.collect_style_fetch_tasks(&mut collection);

    let collection = collection.into_inner();

    let fetch = options.fetch.unwrap_or_else(|| {
      env
        .get_global()
        .unwrap()
        .get_named_property("fetch")
        .expect(
          "No global fetch() function available. Please provide one using a third-party package.",
        )
    });

    let (tx, rx) = bounded(1);

    for task in collection {
      if let Some(resources_cache) = resources_cache.as_ref() {
        let mut lock = resources_cache.lock().unwrap();

        if let Some(cached) = lock.get(&task).cloned() {
          drop(lock);

          tx.send((task, MaybeInitialized::Initialized(cached)))
            .unwrap();

          continue;
        }
      }

      let tx = tx.clone();

      fetch.call(env.create_string(&task)?)?.then(move |ctx| {
        let array_buffer_fn = ctx
          .value
          .get_named_property::<ArrayBufferFn>("arrayBuffer")?;

        array_buffer_fn.apply(ctx.value, ())?.then(move |ctx| {
          tx.send((
            task,
            MaybeInitialized::Uninitialized(ctx.value.into_buffer(&ctx.env)?),
          ))
          .unwrap();

          Ok(())
        })
      })?;
    }

    Ok(RenderTask {
      node: Some(node),
      global,
      viewport: Viewport::new(options.width, options.height),
      format: options.format.unwrap_or(OutputFormat::png),
      quality: options.quality,
      draw_debug_border: options.draw_debug_border.unwrap_or_default(),
      tasks_rx: rx,
      resource_cache: resources_cache.clone(),
    })
  }
}

impl Task for RenderTask<'_> {
  type Output = Vec<u8>;
  type JsValue = Buffer;

  fn compute(&mut self) -> Result<Self::Output> {
    let node = self.node.take().unwrap();

    let resources: Vec<_> = self.tasks_rx.iter().collect();

    let fetched_resources: HashMap<_, _> = resources
      .into_par_iter()
      .filter_map(|(task, buffer)| {
        Some((
          task.clone(),
          match buffer {
            MaybeInitialized::Initialized(source) => source,
            MaybeInitialized::Uninitialized(buffer) => {
              let source = load_image_source_from_bytes(&buffer).ok()?;

              if let Some(cache) = self.resource_cache.clone() {
                let mut lock = cache.lock().unwrap();

                lock.put(task, source.clone());
              }

              source
            }
          },
        ))
      })
      .collect();

    let image = render(
      RenderOptionsBuilder::default()
        .viewport(self.viewport)
        .fetched_resources(fetched_resources)
        .node(node)
        .global(self.global)
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
