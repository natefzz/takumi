use std::{fs::File, io::BufWriter, path::Path, sync::Arc};

use image::load_from_memory;
use parley::{GenericFamily, fontique::FontInfoOverride};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use takumi::{
  GlobalContext,
  layout::{Viewport, node::NodeKind},
  rendering::{
    AnimationFrame, ImageOutputFormat, RenderOptionsBuilder, encode_animated_png,
    encode_animated_webp, render, write_image,
  },
  resources::image::ImageSource,
};

const TEST_FONTS: &[(&[u8], &str, GenericFamily)] = &[
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
  (
    include_bytes!("../../assets/fonts/noto-sans/NotoColorEmoji.ttf"),
    "Noto Color Emoji",
    GenericFamily::Emoji,
  ),
];

fn create_test_context() -> GlobalContext {
  let context = GlobalContext::default();

  context.persistent_image_store.insert(
    "assets/images/yeecord.png",
    Arc::new(ImageSource::Bitmap(
      load_from_memory(include_bytes!("../../assets/images/yeecord.png"))
        .unwrap()
        .into_rgba8(),
    )),
  );

  for (font, name, generic) in TEST_FONTS {
    context
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

  context
}

pub fn create_test_viewport() -> Viewport {
  Viewport::new(1200, 630)
}

/// Helper function to run style width tests
#[allow(dead_code)]
pub fn run_style_width_test(node: NodeKind, fixture_path: &str) {
  let context = create_test_context();
  let viewport = create_test_viewport();

  let image = render(
    RenderOptionsBuilder::default()
      .viewport(viewport)
      .node(node)
      .global(&context)
      .build()
      .unwrap(),
  )
  .unwrap();

  let path = Path::new(fixture_path);

  let mut file = File::create(path).unwrap();
  let mut buf = BufWriter::new(&mut file);

  write_image(&image, &mut buf, ImageOutputFormat::WebP, None).unwrap();
}

#[allow(dead_code)]
pub fn run_webp_animation_test(
  nodes: Vec<(NodeKind, u32)>,
  fixture_path: &str,
  blend: bool,
  dispose: bool,
  loop_count: Option<u16>,
) {
  assert_ne!(nodes.len(), 0);

  let context = create_test_context();
  let viewport = create_test_viewport();

  let frames: Vec<_> = nodes
    .into_par_iter()
    .map(|(node, duration_ms)| {
      AnimationFrame::new(
        render(
          RenderOptionsBuilder::default()
            .viewport(viewport)
            .node(node)
            .global(&context)
            .build()
            .unwrap(),
        )
        .unwrap(),
        duration_ms,
      )
    })
    .collect();

  let mut out = File::create(fixture_path).unwrap();
  encode_animated_webp(&frames, &mut out, blend, dispose, loop_count).unwrap();
}

#[allow(dead_code)]
pub fn run_png_animation_test(
  nodes: Vec<(NodeKind, u32)>,
  fixture_path: &str,
  loop_count: Option<u16>,
) {
  assert_ne!(nodes.len(), 0);

  let context = create_test_context();
  let viewport = create_test_viewport();

  let frames: Vec<_> = nodes
    .into_par_iter()
    .map(|(node, duration_ms)| {
      AnimationFrame::new(
        render(
          RenderOptionsBuilder::default()
            .viewport(viewport)
            .node(node)
            .global(&context)
            .build()
            .unwrap(),
        )
        .unwrap(),
        duration_ms,
      )
    })
    .collect();

  let mut out = File::create(fixture_path).unwrap();
  encode_animated_png(&frames, &mut out, loop_count).unwrap();
}
