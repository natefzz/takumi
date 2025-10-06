//! Text node implementation for the takumi layout system.
//!
//! This module contains the TextNode struct which is used to render
//! text content with configurable font properties and styling.

use serde::{Deserialize, Serialize};
use taffy::{AvailableSpace, Layout, Size};

use crate::{
  layout::{
    inline::{InlineContentKind, break_lines, create_inline_constraint},
    node::Node,
    style::Style,
  },
  rendering::{Canvas, RenderContext, apply_text_transform, draw_text},
};

/// A node that renders text content.
///
/// Text nodes display text with configurable font properties,
/// alignment, and styling options.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TextNode {
  /// The styling properties for this text node
  pub style: Option<Style>,
  /// The text content to be rendered
  pub text: String,
}

impl<Nodes: Node<Nodes>> Node<Nodes> for TextNode {
  fn take_style(&mut self) -> Style {
    self.style.take().unwrap_or_default()
  }

  fn inline_content(&self, context: &RenderContext) -> Option<InlineContentKind> {
    Some(InlineContentKind::Text(
      apply_text_transform(&self.text, context.style.text_transform).to_string(),
    ))
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
    let (max_width, max_height) =
      create_inline_constraint(context, available_space, known_dimensions);

    let font_style = context.style.to_sized_font_style(context);

    let mut layout =
      context
        .global
        .font_context
        .create_inline_layout((&font_style).into(), |builder| {
          builder.push_text(&apply_text_transform(
            &self.text,
            context.style.text_transform,
          ));
        });

    break_lines(&mut layout, max_width, max_height);

    let (max_run_width, total_height) =
      layout
        .lines()
        .fold((0.0, 0.0), |(max_run_width, total_height), line| {
          let metrics = line.metrics();
          (
            metrics.advance.max(max_run_width),
            total_height + metrics.line_height,
          )
        });

    taffy::Size {
      width: max_run_width.ceil().min(max_width),
      height: total_height.ceil(),
    }
  }
}
