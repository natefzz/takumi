use parley::InlineBox;
use taffy::{AvailableSpace, Size};

use crate::{
  layout::{node::Node, style::Color},
  rendering::{MaxHeight, RenderContext},
};

pub(crate) enum InlineItem<N: Node<N>> {
  Node(N),
  Text(String),
}

pub enum InlineContentKind {
  Text(String),
  Box,
}

pub struct InlineTree<'g, N: Node<N>> {
  items: Vec<(InlineItem<N>, RenderContext<'g>)>,
}

impl<'g, N: Node<N>> InlineTree<'g, N> {
  pub fn new() -> Self {
    Self { items: Vec::new() }
  }

  pub(crate) fn try_insert_node(&mut self, node: N, context: RenderContext<'g>) {
    if let Some(content) = node.inline_content(&context) {
      self.items.push((
        match content {
          InlineContentKind::Box => InlineItem::Node(node),
          InlineContentKind::Text(text) => InlineItem::Text(text),
        },
        context,
      ));
    }
  }

  pub(crate) fn create_render_layout(
    &self,
    context: &RenderContext,
    size: Size<f32>,
  ) -> (parley::Layout<Color>, Vec<Size<f32>>) {
    let font_style = context.style.to_sized_font_style(context);
    let mut boxes = Vec::new();

    let mut layout =
      context
        .global
        .font_context
        .create_inline_layout((&font_style).into(), |builder| {
          let mut index_pos = 0;

          for (item, context) in &self.items {
            match item {
              InlineItem::Text(text) => {
                builder.push_style_span((&context.style.to_sized_font_style(context)).into());
                builder.push_text(text);
                builder.pop_style_span();

                index_pos += text.len();
              }
              InlineItem::Node(node) => {
                let size = node.measure(
                  context,
                  Size {
                    width: AvailableSpace::Definite(size.width),
                    height: AvailableSpace::Definite(size.height),
                  },
                  Size::NONE,
                );

                builder.push_inline_box(InlineBox {
                  index: index_pos,
                  id: boxes.len() as u64,
                  width: size.width,
                  height: size.height,
                });

                boxes.push(size);
              }
            }
          }
        });

    let max_height = match font_style.parent.line_clamp.as_ref() {
      Some(clamp) => Some(MaxHeight::Both(size.height, clamp.count)),
      None => Some(MaxHeight::Absolute(size.height)),
    };

    break_lines(&mut layout, size.width, max_height);

    layout.align(
      Some(size.width),
      context.style.text_align.into(),
      Default::default(),
    );

    (layout, boxes)
  }

  pub(crate) fn measure(
    &self,
    context: &RenderContext,
    available_space: Size<AvailableSpace>,
    known_dimensions: Size<Option<f32>>,
  ) -> Size<f32> {
    let (max_width, max_height) =
      create_inline_constraint(context, available_space, known_dimensions);

    let font_style = context.style.to_sized_font_style(context);

    let mut boxes = Vec::new();

    let mut layout =
      context
        .global
        .font_context
        .create_inline_layout((&font_style).into(), |builder| {
          let mut idx = 0;
          let mut index_pos = 0;

          for (item, context) in &self.items {
            match item {
              InlineItem::Text(text) => {
                builder.push_style_span((&context.style.to_sized_font_style(context)).into());
                builder.push_text(text);
                builder.pop_style_span();

                index_pos += text.len();
              }
              InlineItem::Node(node) => {
                let size = node.measure(context, available_space, Size::NONE);

                boxes.push(size);

                builder.push_inline_box(InlineBox {
                  index: index_pos,
                  id: idx,
                  width: size.width,
                  height: size.height,
                });

                idx += 1;
              }
            }
          }
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

pub(crate) fn create_inline_constraint(
  context: &RenderContext,
  available_space: Size<AvailableSpace>,
  known_dimensions: Size<Option<f32>>,
) -> (f32, Option<MaxHeight>) {
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

  let height_constraint_with_max_lines =
    match (context.style.line_clamp.as_ref(), height_constraint) {
      (Some(clamp), Some(height)) => Some(MaxHeight::Both(height, clamp.count)),
      (Some(clamp), None) => Some(MaxHeight::Lines(clamp.count)),
      (None, Some(height)) => Some(MaxHeight::Absolute(height)),
      (None, None) => None,
    };

  (
    width_constraint.unwrap_or(f32::MAX),
    height_constraint_with_max_lines,
  )
}

pub(crate) fn break_lines(
  layout: &mut parley::Layout<Color>,
  max_width: f32,
  max_height: Option<MaxHeight>,
) {
  let Some(max_height) = max_height else {
    return layout.break_all_lines(Some(max_width));
  };

  match max_height {
    MaxHeight::Lines(lines) => {
      let mut breaker = layout.break_lines();

      for _ in 0..lines {
        if breaker.break_next(max_width).is_none() {
          // no more lines to break
          break;
        };
      }

      breaker.finish();
    }
    MaxHeight::Absolute(max_height) => {
      let mut total_height = 0.0;
      let mut breaker = layout.break_lines();

      while total_height < max_height {
        let Some((_, height)) = breaker.break_next(max_width) else {
          // no more lines to break
          break;
        };

        total_height += height;
      }

      // if its over the max height after last break, revert the break
      if total_height > max_height {
        breaker.revert();
      }

      breaker.finish();
    }
    MaxHeight::Both(max_height, max_lines) => {
      let mut total_height = 0.0;
      let mut line_count = 0;
      let mut breaker = layout.break_lines();

      while total_height < max_height {
        if line_count >= max_lines {
          break;
        }

        let Some((_, height)) = breaker.break_next(max_width) else {
          // no more lines to break
          break;
        };

        line_count += 1;
        total_height += height;
      }

      if total_height > max_height {
        breaker.revert();
      }

      breaker.finish();
    }
  }
}
