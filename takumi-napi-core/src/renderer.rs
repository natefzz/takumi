use napi::bindgen_prelude::*;
use napi_derive::napi;
use takumi::{
  GlobalContext,
  layout::{Viewport, node::NodeKind},
  parley::{FontWeight, GenericFamily, fontique::FontInfoOverride},
  rendering::{ImageOutputFormat, render, write_image},
  resources::image::load_image_source_from_bytes,
};

use crate::{
  FontInput, FontInputOwned, load_font_task::LoadFontTask,
  put_persistent_image_task::PutPersistentImageTask, render_animation_task::RenderAnimationTask,
  render_task::RenderTask,
};
use std::{io::Cursor, sync::Arc};

#[napi]
#[derive(Default)]
pub struct Renderer(Arc<GlobalContext>);

#[napi(object)]
pub struct RenderOptions {
  pub width: u32,
  pub height: u32,
  pub format: Option<OutputFormat>,
  pub quality: Option<u8>,
}

#[napi(object)]
pub struct AnimationFrameSource<'ctx> {
  #[napi(ts_type = "AnyNode")]
  pub node: Object<'ctx>,
  pub duration_ms: u32,
}

#[napi(object)]
pub struct RenderAnimationOptions {
  pub width: u32,
  pub height: u32,
  pub format: Option<AnimationOutputFormat>,
}

#[napi(string_enum)]
#[allow(non_camel_case_types)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum AnimationOutputFormat {
  webp,
  apng,
}

#[napi(string_enum)]
#[allow(non_camel_case_types)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
  webp,
  avif,
  png,
  jpeg,
  WebP,
  Avif,
  Jpeg,
  Png,
  raw,
}

impl From<OutputFormat> for ImageOutputFormat {
  fn from(format: OutputFormat) -> Self {
    match format {
      OutputFormat::WebP => ImageOutputFormat::WebP,
      OutputFormat::Avif => ImageOutputFormat::Avif,
      OutputFormat::Jpeg => ImageOutputFormat::Jpeg,
      OutputFormat::Png => ImageOutputFormat::Png,
      OutputFormat::png => ImageOutputFormat::Png,
      OutputFormat::jpeg => ImageOutputFormat::Jpeg,
      OutputFormat::webp => ImageOutputFormat::WebP,
      OutputFormat::avif => ImageOutputFormat::Avif,
      // SAFETY: It's handled in the render task
      OutputFormat::raw => unreachable!(),
    }
  }
}

#[napi(object)]
pub struct PersistentImage<'ctx> {
  pub src: String,
  #[napi(ts_type = "Buffer | ArrayBuffer")]
  pub data: BufferSlice<'ctx>,
}

#[napi(object)]
#[derive(Default)]
pub struct ConstructRendererOptions<'ctx> {
  pub debug: Option<bool>,
  pub persistent_images: Option<Vec<PersistentImage<'ctx>>>,
  #[napi(ts_type = "Font[] | undefined")]
  pub fonts: Option<Vec<Object<'ctx>>>,
  pub load_default_fonts: Option<bool>,
}

const EMBEDDED_FONTS: &[(&[u8], &str, GenericFamily)] = &[
  (
    include_bytes!("../../assets/fonts/geist/Geist[wght].woff2"),
    "Geist",
    GenericFamily::SansSerif,
  ),
  (
    include_bytes!("../../assets/fonts/geist/GeistMono[wght].woff2"),
    "Geist Mono",
    GenericFamily::Monospace,
  ),
];

#[napi]
impl Renderer {
  #[napi(constructor)]
  pub fn new(env: Env, options: Option<ConstructRendererOptions>) -> Self {
    let options = options.unwrap_or_default();

    let load_default_fonts = options
      .load_default_fonts
      .unwrap_or_else(|| options.fonts.is_none());

    let renderer = Self(Arc::new(GlobalContext {
      draw_debug_border: options.debug.unwrap_or_default(),
      ..Default::default()
    }));

    if load_default_fonts {
      for (font, name, generic) in EMBEDDED_FONTS {
        renderer
          .0
          .font_context
          .load_and_store(
            font,
            Some(FontInfoOverride {
              family_name: Some(name),
              ..Default::default()
            }),
            Some(*generic),
          )
          .unwrap();
      }
    }

    if let Some(images) = options.persistent_images {
      for image in images {
        let image_source = load_image_source_from_bytes(&image.data).unwrap();

        renderer
          .0
          .persistent_image_store
          .insert(&image.src, image_source);
      }
    }

    if let Some(fonts) = options.fonts {
      for font in fonts {
        if font.is_arraybuffer().unwrap() || font.is_buffer().unwrap() {
          // SAFETY: We know the font is a buffer
          let buffer = unsafe { BufferSlice::from_napi_value(env.raw(), font.raw()).unwrap() };

          renderer
            .0
            .font_context
            .load_and_store(&buffer, None, None)
            .unwrap();

          continue;
        }

        let font: FontInput = unsafe { FontInput::from_napi_value(env.raw(), font.raw()).unwrap() };

        let font_override = FontInfoOverride {
          family_name: font.name.as_deref(),
          style: font.style.map(Into::into),
          weight: font.weight.map(|weight| FontWeight::new(weight as f32)),
          axes: None,
          width: None,
        };

        renderer
          .0
          .font_context
          .load_and_store(&font.data, Some(font_override), None)
          .unwrap();
      }
    }

    renderer
  }

  #[napi]
  pub fn purge_font_cache(&self) {
    self.0.font_context.purge_cache();
  }

  #[napi(
    ts_args_type = "src: string, data: Buffer | ArrayBuffer, signal?: AbortSignal",
    ts_return_type = "Promise<void>"
  )]
  pub fn put_persistent_image_async(
    &self,
    src: String,
    data: Buffer,
    signal: Option<AbortSignal>,
  ) -> AsyncTask<PutPersistentImageTask> {
    AsyncTask::with_optional_signal(
      PutPersistentImageTask {
        src: Some(src),
        context: Arc::clone(&self.0),
        buffer: data,
      },
      signal,
    )
  }

  #[napi(
    ts_args_type = "data: Font, signal?: AbortSignal",
    ts_return_type = "Promise<number>"
  )]
  pub fn load_font_async(
    &self,
    env: Env,
    data: Object,
    signal: Option<AbortSignal>,
  ) -> AsyncTask<LoadFontTask> {
    self.load_fonts_async(env, vec![data], signal)
  }

  #[napi(
    ts_args_type = "fonts: Font[], signal?: AbortSignal",
    ts_return_type = "Promise<number>"
  )]
  pub fn load_fonts_async(
    &self,
    env: Env,
    fonts: Vec<Object>,
    signal: Option<AbortSignal>,
  ) -> AsyncTask<LoadFontTask> {
    let fonts = fonts
      .into_iter()
      .map(|font| {
        if font.is_arraybuffer().unwrap() || font.is_buffer().unwrap() {
          FontInputOwned {
            name: None,
            // SAFETY: We know the font is a buffer
            data: unsafe { Buffer::from_napi_value(env.raw(), font.raw()).unwrap() },
            weight: None,
            style: None,
          }
        } else {
          unsafe { FontInputOwned::from_napi_value(env.raw(), font.raw()).unwrap() }
        }
      })
      .collect();

    AsyncTask::with_optional_signal(
      LoadFontTask {
        context: Arc::clone(&self.0),
        buffers: fonts,
      },
      signal,
    )
  }

  #[napi]
  pub fn clear_image_store(&self) {
    self.0.persistent_image_store.clear();
  }

  #[napi(
    ts_args_type = "source: AnyNode, options: RenderOptions, signal?: AbortSignal",
    ts_return_type = "Promise<Buffer>"
  )]
  pub fn render_async(
    &self,
    env: Env,
    source: Object,
    options: RenderOptions,
    signal: Option<AbortSignal>,
  ) -> Result<AsyncTask<RenderTask>> {
    let node = env.from_js_value(source)?;

    Ok(AsyncTask::with_optional_signal(
      RenderTask {
        node: Some(node),
        context: Arc::clone(&self.0),
        viewport: Viewport::new(options.width, options.height),
        format: options.format.unwrap_or(OutputFormat::png),
        quality: options.quality,
      },
      signal,
    ))
  }

  #[napi(
    ts_args_type = "source: AnimationFrameSource[], options: RenderAnimationOptions, signal?: AbortSignal",
    ts_return_type = "Promise<Buffer>"
  )]
  pub fn render_animation(
    &self,
    env: Env,
    source: Vec<AnimationFrameSource>,
    options: RenderAnimationOptions,
    signal: Option<AbortSignal>,
  ) -> Result<AsyncTask<RenderAnimationTask>> {
    let nodes = source
      .into_iter()
      .map(|frame| (env.from_js_value(frame.node).unwrap(), frame.duration_ms))
      .collect::<Vec<_>>();

    Ok(AsyncTask::with_optional_signal(
      RenderAnimationTask {
        nodes: Some(nodes),
        context: Arc::clone(&self.0),
        viewport: Viewport::new(options.width, options.height),
        format: options.format.unwrap_or(AnimationOutputFormat::webp),
      },
      signal,
    ))
  }

  #[napi(ts_args_type = "source: AnyNode, options: RenderOptions")]
  pub fn render(&self, env: Env, source: Object, options: RenderOptions) -> Result<Buffer> {
    let node: NodeKind = env.from_js_value(source)?;

    let viewport = Viewport::new(options.width, options.height);
    let image = render(viewport, &self.0, node).unwrap();

    let format = options.format.unwrap_or(OutputFormat::png);

    if format == OutputFormat::raw {
      return Ok(image.into_raw().into());
    }

    let mut buffer = Vec::new();
    let mut cursor = Cursor::new(&mut buffer);

    write_image(&image, &mut cursor, format.into(), options.quality).unwrap();

    Ok(buffer.into())
  }
}
