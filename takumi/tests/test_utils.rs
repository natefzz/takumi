use std::{fs::File, io::BufWriter, path::Path, sync::Arc};

use image::load_from_memory;
use parley::{GenericFamily, fontique::FontInfoOverride};
use takumi::{
  GlobalContext,
  layout::{Viewport, node::NodeKind},
  rendering::{ImageOutputFormat, render, write_image},
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

fn create_test_viewport() -> Viewport {
  Viewport::new(1200, 630)
}

/// Helper function to run style width tests
pub fn run_style_width_test(node: NodeKind, fixture_path: &str) {
  let context = create_test_context();
  let viewport = create_test_viewport();

  let image = render(viewport, &context, node).unwrap();

  let path = Path::new(fixture_path);

  let mut file = File::create(path).unwrap();
  let mut buf = BufWriter::new(&mut file);

  write_image(&image, &mut buf, ImageOutputFormat::Png, None).unwrap();
}
