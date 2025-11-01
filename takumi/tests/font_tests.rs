use takumi::{GlobalContext, resources::font::FontError};

// Include test font data using include_bytes!
static TTF_FONT: &[u8] =
  include_bytes!("../../assets/fonts/noto-sans/NotoSansTC-VariableFont_wght.ttf");
static WOFF2_FONT: &[u8] = include_bytes!("../../assets/fonts/geist/Geist[wght].woff2");

#[test]
fn test_ttf_font_loading() {
  let mut context = GlobalContext::default();

  assert!(
    context
      .font_context
      .load_and_store(TTF_FONT, None, None)
      .is_ok()
  );
}

#[test]
fn test_woff2_font_loading() {
  let mut context = GlobalContext::default();

  assert!(
    context
      .font_context
      .load_and_store(WOFF2_FONT, None, None)
      .is_ok()
  );
}

#[test]
fn test_invalid_format_detection() {
  // Test with invalid data
  let invalid_data = vec![0x00, 0x01, 0x02, 0x03];
  let mut context = GlobalContext::default();

  let result = context
    .font_context
    .load_and_store(&invalid_data, None, None);
  assert!(matches!(result, Err(FontError::UnsupportedFormat)));
}

#[test]
fn test_empty_data() {
  // Test with empty data
  let empty_data = &[];
  let mut context = GlobalContext::default();

  let result = context.font_context.load_and_store(empty_data, None, None);
  assert!(matches!(result, Err(FontError::UnsupportedFormat)));
}

#[test]
fn test_too_short_data() {
  // Test with data too short for format detection
  let short_data = &[0x00, 0x01, 0x00];
  let mut context = GlobalContext::default();

  let result = context.font_context.load_and_store(short_data, None, None);
  assert!(matches!(result, Err(FontError::UnsupportedFormat)));
}
