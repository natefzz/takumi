use std::{io::Cursor, sync::Arc};

use base64::{Engine, prelude::BASE64_STANDARD};
use serde_wasm_bindgen::from_value;
use takumi::{
  GlobalContext,
  image::load_from_memory,
  layout::{Viewport, node::NodeKind},
  parley::{FontWeight, fontique::FontInfoOverride},
  rendering::{
    AnimationFrame, RenderOptionsBuilder, encode_animated_png, encode_animated_webp, render,
    write_image,
  },
  resources::image::ImageSource,
};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = r#"
export interface AnyNode {
  type: string;
  [key: string]: any;
}
"#;

#[wasm_bindgen]
extern "C" {
  #[wasm_bindgen(typescript_type = "AnyNode")]
  #[derive(Debug)]
  pub type AnyNode;
}

/// Proxy type for the ImageOutputFormat enum.
/// This is needed because wasm-bindgen doesn't support cfg macro in enum variants.
/// https://github.com/erwanvivien/fast_qr/pull/41/files
#[wasm_bindgen]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageOutputFormat {
  WebP = "webp",
  Png = "png",
  Jpeg = "jpeg",
}

#[wasm_bindgen]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnimationOutputFormat {
  WebP = "webp",
  APng = "apng",
}

#[wasm_bindgen]
pub struct FontInfo {
  name: Option<String>,
  data: Vec<u8>,
  weight: Option<f64>,
  style: Option<FontStyle>,
}

#[wasm_bindgen]
pub enum FontStyle {
  Normal = "normal",
  Italic = "italic",
  Oblique = "oblique",
}

#[wasm_bindgen]
pub struct AnimationFrameSource {
  node: AnyNode,
  #[wasm_bindgen(js_name = durationMs)]
  duration_ms: u32,
}

#[wasm_bindgen]
impl AnimationFrameSource {
  #[wasm_bindgen(constructor)]
  pub fn new(
    node: AnyNode,
    #[wasm_bindgen(js_name = durationMs)] duration_ms: u32,
  ) -> AnimationFrameSource {
    AnimationFrameSource { node, duration_ms }
  }
}

impl From<FontStyle> for takumi::parley::FontStyle {
  fn from(style: FontStyle) -> Self {
    match style {
      FontStyle::Italic => takumi::parley::FontStyle::Italic,
      FontStyle::Oblique => takumi::parley::FontStyle::Oblique(None),
      FontStyle::Normal | FontStyle::__Invalid => takumi::parley::FontStyle::Normal,
    }
  }
}

impl From<ImageOutputFormat> for takumi::rendering::ImageOutputFormat {
  fn from(format: ImageOutputFormat) -> Self {
    match format {
      ImageOutputFormat::WebP => takumi::rendering::ImageOutputFormat::WebP,
      ImageOutputFormat::Jpeg => takumi::rendering::ImageOutputFormat::Jpeg,
      ImageOutputFormat::Png | ImageOutputFormat::__Invalid => {
        takumi::rendering::ImageOutputFormat::Png
      }
    }
  }
}

#[wasm_bindgen]
#[derive(Default)]
pub struct Renderer {
  context: GlobalContext,
}

#[wasm_bindgen]
impl Renderer {
  #[wasm_bindgen(constructor)]
  pub fn new() -> Renderer {
    Renderer::default()
  }

  #[wasm_bindgen(js_name = loadFontWithInfo)]
  pub fn load_font_with_info(&self, #[wasm_bindgen(js_name = fontData)] font_data: FontInfo) {
    self
      .context
      .font_context
      .load_and_store(
        &font_data.data,
        Some(FontInfoOverride {
          family_name: font_data.name.as_deref(),
          style: font_data.style.map(Into::into),
          weight: font_data
            .weight
            .map(|weight| FontWeight::new(weight as f32)),
          axes: None,
          width: None,
        }),
        None,
      )
      .unwrap();
  }

  #[wasm_bindgen(js_name = loadFont)]
  pub fn load_font(&self, buffer: &[u8]) {
    self
      .context
      .font_context
      .load_and_store(buffer, None, None)
      .unwrap();
  }

  #[wasm_bindgen(js_name = putPersistentImage)]
  pub fn put_persistent_image(&self, src: String, data: &[u8]) {
    self.context.persistent_image_store.insert(
      &src,
      Arc::new(ImageSource::Bitmap(
        load_from_memory(data).unwrap().into_rgba8(),
      )),
    );
  }

  #[wasm_bindgen(js_name = clearImageStore)]
  pub fn clear_image_store(&self) {
    self.context.persistent_image_store.clear();
  }

  #[wasm_bindgen]
  pub fn render(
    &self,
    node: AnyNode,
    width: u32,
    height: u32,
    format: Option<ImageOutputFormat>,
    quality: Option<u8>,
    draw_debug_border: Option<bool>,
  ) -> Vec<u8> {
    let node: NodeKind = from_value(node.into()).unwrap();

    let viewport = Viewport::new(width, height);
    let image = render(
      RenderOptionsBuilder::default()
        .viewport(viewport)
        .draw_debug_border(draw_debug_border.unwrap_or_default())
        .node(node)
        .global(&self.context)
        .build()
        .unwrap(),
    )
    .unwrap();

    let mut buffer = Vec::new();
    let mut cursor = Cursor::new(&mut buffer);

    write_image(
      &image,
      &mut cursor,
      format.unwrap_or(ImageOutputFormat::Png).into(),
      quality,
    )
    .unwrap();

    buffer
  }

  #[wasm_bindgen(js_name = "renderAsDataUrl")]
  pub fn render_as_data_url(
    &self,
    node: AnyNode,
    width: u32,
    height: u32,
    format: Option<ImageOutputFormat>,
    quality: Option<u8>,
    draw_debug_border: Option<bool>,
  ) -> String {
    let buffer = self.render(node, width, height, format, quality, draw_debug_border);
    let format: takumi::rendering::ImageOutputFormat =
      format.unwrap_or(ImageOutputFormat::Png).into();

    let mut data_uri = String::new();

    data_uri.push_str("data:");
    data_uri.push_str(format.content_type());
    data_uri.push_str(";base64,");
    data_uri.push_str(&BASE64_STANDARD.encode(buffer));

    data_uri
  }

  #[wasm_bindgen(js_name = renderAnimation)]
  pub fn render_animation(
    &self,
    frames: Vec<AnimationFrameSource>,
    width: u32,
    height: u32,
    format: Option<AnimationOutputFormat>,
    draw_debug_border: Option<bool>,
  ) -> Vec<u8> {
    let viewport = Viewport::new(width, height);

    let rendered_frames: Vec<AnimationFrame> = frames
      .into_iter()
      .map(|frame| {
        let node: NodeKind = from_value(frame.node.into()).unwrap();
        let duration_ms = frame.duration_ms;

        let image = render(
          RenderOptionsBuilder::default()
            .viewport(viewport)
            .node(node)
            .global(&self.context)
            .draw_debug_border(draw_debug_border.unwrap_or_default())
            .build()
            .unwrap(),
        )
        .unwrap();
        AnimationFrame::new(image, duration_ms)
      })
      .collect();

    let mut buffer = Vec::new();
    let mut cursor = Cursor::new(&mut buffer);

    match format.unwrap_or(AnimationOutputFormat::WebP) {
      AnimationOutputFormat::WebP | AnimationOutputFormat::__Invalid => {
        encode_animated_webp(&rendered_frames, &mut cursor, true, false, None).unwrap();
      }
      AnimationOutputFormat::APng => {
        encode_animated_png(&rendered_frames, &mut cursor, None).unwrap();
      }
    }

    buffer
  }
}
