//! Text node implementation for the takumi layout system.
//!
//! This module contains the TextNode struct which is used to render
//! text content with configurable font properties and styling.

use serde::{Deserialize, Serialize};
use taffy::{AvailableSpace, Layout, Size};

use crate::{
  GlobalContext,
  layout::{
    node::Node,
    style::{SizedFontStyle, Style},
  },
  rendering::{
    Canvas, MaxHeight, RenderContext, apply_text_transform, create_text_layout, draw_text,
  },
};

/// A node that renders text content.
///
/// Text nodes display text with configurable font properties,
/// alignment, and styling options.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TextNode {
  /// The styling properties for this text node
  #[serde(default)]
  pub style: Style,
  /// The text content to be rendered
  pub text: String,
}

impl<Nodes: Node<Nodes>> Node<Nodes> for TextNode {
  fn get_style(&self) -> &Style {
    &self.style
  }

  fn draw_content(&self, context: &RenderContext, canvas: &Canvas, layout: Layout) {
    draw_text(&self.text, context, canvas, layout);
  }

  fn measure(
    &self,
    context: &RenderContext,
    available_space: Size<AvailableSpace>,
    known_dimensions: Size<Option<f32>>,
  ) -> Size<f32> {
    measure_text(
      context.global,
      &self.text,
      context.style.to_sized_font_style(context),
      known_dimensions,
      available_space,
    )
  }

  fn has_draw_content(&self) -> bool {
    true
  }
}

/// Measures the size of text based on font style and available space.
///
/// This function handles text wrapping, line height, and respects both explicit
/// dimensions and available space constraints.
pub(crate) fn measure_text(
  global: &GlobalContext,
  text: &str,
  style: SizedFontStyle,
  known_dimensions: Size<Option<f32>>,
  available_space: Size<AvailableSpace>,
) -> Size<f32> {
  if text.trim().is_empty()
    || known_dimensions.width == Some(0.0)
    || known_dimensions.height == Some(0.0)
  {
    return Size {
      width: 0.0,
      height: 0.0,
    };
  }

  let width_constraint = known_dimensions.width.or(match available_space.width {
    AvailableSpace::MinContent => Some(0.0),
    AvailableSpace::MaxContent => None,
    AvailableSpace::Definite(width) => Some(width),
  });

  let height_constraint = known_dimensions.height.or(match available_space.height {
    AvailableSpace::MinContent => Some(0.0),
    AvailableSpace::MaxContent => None,
    AvailableSpace::Definite(height) => Some(height),
  });

  let height_constraint_with_max_lines = match (style.parent.line_clamp, height_constraint) {
    (Some(max_lines), Some(height)) => Some(MaxHeight::Both(height, max_lines)),
    (Some(max_lines), None) => Some(MaxHeight::Lines(max_lines)),
    (None, Some(height)) => Some(MaxHeight::Absolute(height)),
    (None, None) => None,
  };

  let text = apply_text_transform(text, style.parent.text_transform);

  let buffer = create_text_layout(
    &text,
    &style,
    global,
    width_constraint.unwrap_or(f32::MAX),
    height_constraint_with_max_lines,
  );

  let (max_run_width, total_height) =
    buffer
      .lines()
      .fold((0.0, 0.0), |(max_run_width, total_height), line| {
        let metrics = line.metrics();
        (
          metrics.advance.max(max_run_width),
          total_height + metrics.line_height,
        )
      });

  taffy::Size {
    width: max_run_width
      .ceil()
      .min(width_constraint.unwrap_or(f32::MAX)),
    height: total_height.ceil(),
  }
}

#[cfg(test)]
mod tests {
  use std::sync::LazyLock;
  use taffy::{AvailableSpace, geometry::Size};

  use crate::{
    GlobalContext,
    layout::{
      DEFAULT_FONT_SIZE, DEFAULT_LINE_HEIGHT_SCALER, Viewport,
      node::measure_text,
      style::{Affine, InheritedStyle, LengthUnit, LineHeight, SizedFontStyle},
    },
    rendering::RenderContext,
  };

  const NOTO_SANS_REGULAR_BUFFER: &[u8] =
    include_bytes!("../../../../assets/fonts/noto-sans/NotoSans-Regular.ttf");

  // Viewport dimensions
  const VIEWPORT_WIDTH: u32 = 800;
  const VIEWPORT_HEIGHT: u32 = 600;

  static SHARED_GLOBAL_CONTEXT: LazyLock<GlobalContext> = LazyLock::new(|| {
    let global_context = GlobalContext::default();

    global_context
      .font_context
      .load_and_store(NOTO_SANS_REGULAR_BUFFER, None, None)
      .unwrap();

    global_context
  });

  // Helper function to create a basic RenderContext for testing
  fn create_test_context() -> RenderContext<'static> {
    RenderContext {
      global: &SHARED_GLOBAL_CONTEXT,
      font_size: DEFAULT_FONT_SIZE,
      viewport: Viewport {
        width: VIEWPORT_WIDTH,
        height: VIEWPORT_HEIGHT,
        font_size: DEFAULT_FONT_SIZE,
      },
      transform: Affine::identity(),
      style: InheritedStyle::default(),
    }
  }

  // Helper function to create known dimensions for testing
  fn create_known_dimensions(width: Option<f32>, height: Option<f32>) -> Size<Option<f32>> {
    Size { width, height }
  }

  // Helper function to create available space for testing
  fn create_available_space(width: AvailableSpace, height: AvailableSpace) -> Size<AvailableSpace> {
    Size { width, height }
  }

  // Helper function to create text measurement parameters
  fn measure_text_helper(
    text: &str,
    width: Option<f32>,
    height: Option<f32>,
    available_width: AvailableSpace,
    available_height: AvailableSpace,
  ) -> Size<f32> {
    let context = create_test_context();
    let style = context.style.to_sized_font_style(&context);

    measure_text(
      context.global,
      text,
      style,
      create_known_dimensions(width, height),
      create_available_space(available_width, available_height),
    )
  }

  // Helper function to measure text with custom style
  fn measure_text_with_style(
    text: &str,
    style: SizedFontStyle<'_>,
    width: Option<f32>,
    height: Option<f32>,
    available_width: AvailableSpace,
    available_height: AvailableSpace,
  ) -> Size<f32> {
    let context = create_test_context();

    measure_text(
      context.global,
      text,
      style,
      create_known_dimensions(width, height),
      create_available_space(available_width, available_height),
    )
  }

  #[test]
  fn test_measure_text_basic() {
    let result = measure_text_helper(
      "Hello, world!",
      None,
      None,
      AvailableSpace::MaxContent,
      AvailableSpace::MaxContent,
    );

    assert!(result.width > 0.0);
    assert_eq!(
      result.height,
      (DEFAULT_FONT_SIZE * DEFAULT_LINE_HEIGHT_SCALER).ceil()
    );
  }

  #[test]
  fn test_measure_text_with_width_constraint() {
    let result = measure_text_helper(
      "This is a long text that should wrap when given a width constraint",
      Some(200.0),
      None,
      AvailableSpace::Definite(300.0),
      AvailableSpace::MaxContent,
    );

    assert!(result.width <= 200.0);
    assert!(result.height >= (2.0 * DEFAULT_FONT_SIZE * DEFAULT_LINE_HEIGHT_SCALER).ceil());
    // Have to allow one pixel tolerance due to rounding
    assert!((result.height % (DEFAULT_FONT_SIZE * DEFAULT_LINE_HEIGHT_SCALER)).floor() == 0.0);
  }

  #[test]
  fn test_measure_text_with_height_constraint() {
    let context = create_test_context();
    let parent = InheritedStyle {
      line_clamp: Some(2),
      ..Default::default()
    };

    let result = measure_text_with_style(
      "This is a long text that should be clamped to a specific number of lines",
      parent.to_sized_font_style(&context),
      Some(200.0),
      None,
      AvailableSpace::Definite(300.0),
      AvailableSpace::MaxContent,
    );

    let expected_height = (2.0 * DEFAULT_FONT_SIZE * DEFAULT_LINE_HEIGHT_SCALER).ceil();
    assert_eq!(result.height, expected_height);
  }

  #[test]
  fn test_measure_text_empty_string() {
    let result = measure_text_helper(
      "",
      None,
      None,
      AvailableSpace::MaxContent,
      AvailableSpace::MaxContent,
    );

    assert_eq!(result.width, 0.0);
    assert_eq!(result.height, 0.0);
  }

  #[test]
  fn test_measure_text_single_character() {
    let result = measure_text_helper(
      "A",
      None,
      None,
      AvailableSpace::MaxContent,
      AvailableSpace::MaxContent,
    );

    assert!(result.width > 0.0);
    assert_eq!(
      result.height,
      (DEFAULT_FONT_SIZE * DEFAULT_LINE_HEIGHT_SCALER).ceil()
    );
  }

  #[test]
  fn test_measure_text_with_line_clamp() {
    let context = create_test_context();
    let parent = InheritedStyle {
      line_clamp: Some(3),
      ..Default::default()
    };

    let result = measure_text_with_style(
      "Line 1\nLine 2\nLine 3\nLine 4\nLine 5",
      parent.to_sized_font_style(&context),
      Some(300.0),
      None,
      AvailableSpace::Definite(400.0),
      AvailableSpace::MaxContent,
    );

    let expected_height = (3.0 * DEFAULT_FONT_SIZE * DEFAULT_LINE_HEIGHT_SCALER).ceil();
    assert_eq!(result.height, expected_height);
  }

  #[test]
  fn test_measure_text_zero_width_constraint() {
    let result = measure_text_helper(
      "This text has zero width constraint",
      Some(0.0),
      None,
      AvailableSpace::MinContent,
      AvailableSpace::MaxContent,
    );

    // With zero width, the text should not be rendered
    assert_eq!(result.width, 0.0);
    assert_eq!(result.height, 0.0);
  }

  #[test]
  fn test_measure_text_zero_height_constraint() {
    let result = measure_text_helper(
      "This text has zero height constraint",
      Some(200.0),
      Some(0.0),
      AvailableSpace::Definite(300.0),
      AvailableSpace::MinContent,
    );

    // With zero height, the text should not be rendered
    assert_eq!(result.width, 0.0);
    assert_eq!(result.height, 0.0);
  }

  #[test]
  fn test_measure_text_with_different_font_size() {
    let context = RenderContext {
      font_size: 24.0,
      ..create_test_context()
    };
    let parent = InheritedStyle {
      font_size: LengthUnit::Px(24.0),
      ..Default::default()
    };

    let result = measure_text_with_style(
      "Large font text",
      parent.to_sized_font_style(&context),
      None,
      None,
      AvailableSpace::MaxContent,
      AvailableSpace::MaxContent,
    );

    assert!(result.width > 0.0);
    assert_eq!(result.height, (24.0 * DEFAULT_LINE_HEIGHT_SCALER).ceil());
  }

  #[test]
  fn test_measure_text_with_different_line_height() {
    let context = create_test_context();
    let parent = InheritedStyle {
      line_height: LineHeight(LengthUnit::Em(1.5)),
      ..Default::default()
    };

    let result = measure_text_with_style(
      "Text with increased line height",
      parent.to_sized_font_style(&context),
      None,
      None,
      AvailableSpace::MaxContent,
      AvailableSpace::MaxContent,
    );

    assert!(result.width > 0.0);
    assert_eq!(result.height, DEFAULT_FONT_SIZE * 1.5);
  }

  #[test]
  fn test_measure_text_whitespace_only() {
    let result = measure_text_helper(
      "   \n\t  \n  ",
      Some(200.0),
      None,
      AvailableSpace::Definite(300.0),
      AvailableSpace::MaxContent,
    );

    assert!(result.width >= 0.0);
    assert!(result.height >= 0.0);
  }

  #[test]
  fn test_measure_text_very_long_word() {
    let result = measure_text_helper(
      "verylongwordwithoutanyspaces",
      Some(100.0),
      None,
      AvailableSpace::Definite(200.0),
      AvailableSpace::MaxContent,
    );

    assert!(result.width <= 100.0);
    assert!(result.height >= 19.0);
  }
}
