use image::RgbaImage;
use parley::{GlyphRun, PositionedInlineBox, PositionedLayoutItem};
use taffy::{Layout, Size};

use crate::{
  Result,
  layout::{
    inline::{InlineBrush, InlineLayout},
    node::Node,
    style::{Affine, SizedFontStyle, TextDecorationLine},
  },
  rendering::{
    Canvas, RenderContext, draw_decoration, draw_glyph, overlay_image, resolve_layers_tiles,
  },
};

fn draw_glyph_run(
  style: &SizedFontStyle,
  glyph_run: &GlyphRun<'_, InlineBrush>,
  canvas: &mut Canvas,
  layout: Layout,
  context: &RenderContext,
  image_fill: Option<&RgbaImage>,
) -> Result<()> {
  let decoration_line = style
    .parent
    .text_decoration_line
    .as_ref()
    .unwrap_or(&style.parent.text_decoration.line);

  let run = glyph_run.run();
  let metrics = run.metrics();

  // decoration underline should not overlap with the glyph descent part,
  // as a temporary workaround, we draw the decoration under the glyph.
  if decoration_line.has(TextDecorationLine::Underline) {
    draw_decoration(
      canvas,
      glyph_run,
      style.text_decoration_color,
      glyph_run.baseline() - metrics.underline_offset,
      glyph_run.run().font_size() / 18.0,
      layout,
      context.transform,
    );
  }

  // Collect all glyph IDs for batch processing
  let glyph_ids = glyph_run.positioned_glyphs().map(|glyph| glyph.id);

  let resolved_glyphs = context.global.font_context.resolve_glyphs(run, glyph_ids)?;

  // Draw each glyph using the batch-resolved cache
  for glyph in glyph_run.positioned_glyphs() {
    if let Some(cached_glyph) = resolved_glyphs.get(&glyph.id) {
      draw_glyph(
        glyph,
        cached_glyph,
        canvas,
        style,
        layout,
        image_fill,
        context.transform,
        glyph_run.style(),
      )?;
    }
  }

  if decoration_line.has(TextDecorationLine::LineThrough) {
    let size = glyph_run.run().font_size() / 18.0;
    let offset = glyph_run.baseline() - metrics.strikethrough_offset;

    draw_decoration(
      canvas,
      glyph_run,
      style.text_decoration_color,
      offset,
      size,
      layout,
      context.transform,
    );
  }

  if decoration_line.has(TextDecorationLine::Overline) {
    draw_decoration(
      canvas,
      glyph_run,
      style.text_decoration_color,
      glyph_run.baseline() - metrics.ascent - metrics.underline_offset,
      glyph_run.run().font_size() / 18.0,
      layout,
      context.transform,
    );
  }

  Ok(())
}

pub(crate) fn draw_inline_box<N: Node<N>>(
  inline_box: &PositionedInlineBox,
  node: &N,
  context: &RenderContext,
  canvas: &mut Canvas,
  transform: Affine,
) -> Result<()> {
  if context.opacity == 0.0 {
    return Ok(());
  }

  let context = RenderContext {
    transform: Affine::translation(inline_box.x, inline_box.y) * transform,
    ..context.clone()
  };

  node.draw_content(
    &context,
    canvas,
    Layout {
      size: Size {
        width: inline_box.width,
        height: inline_box.height,
      },
      ..Default::default()
    },
  )
}

pub(crate) fn draw_inline_layout(
  context: &RenderContext,
  canvas: &mut Canvas,
  layout: Layout,
  inline_layout: InlineLayout,
  font_style: &SizedFontStyle,
) -> Result<Vec<PositionedInlineBox>> {
  let content_box = layout.content_box_size();

  // If we have a mask image on the style, render it using the background tiling logic into a
  // temporary image and use that as the glyph fill.
  let fill_image = create_fill_image(context, layout, content_box)?;

  let mut positioned_inline_boxes = Vec::new();

  for line in inline_layout.lines() {
    for item in line.items() {
      match item {
        PositionedLayoutItem::GlyphRun(glyph_run) => {
          draw_glyph_run(
            font_style,
            &glyph_run,
            canvas,
            layout,
            context,
            fill_image.as_ref(),
          )?;
        }
        PositionedLayoutItem::InlineBox(inline_box) => positioned_inline_boxes.push(inline_box),
      }
    }
  }

  Ok(positioned_inline_boxes)
}

fn create_fill_image(
  context: &RenderContext,
  layout: Layout,
  size: Size<f32>,
) -> Result<Option<RgbaImage>> {
  let images = match context.style.mask_image.as_ref() {
    Some(images) => images,
    None => return Ok(None),
  };
  let resolved_tiles = resolve_layers_tiles(
    images,
    context.style.mask_position.as_ref(),
    context.style.mask_size.as_ref(),
    context.style.mask_repeat.as_ref(),
    context,
    layout,
  )?;

  if resolved_tiles.is_empty() {
    return Ok(None);
  }

  let mut composed = RgbaImage::new(size.width as u32, size.height as u32);

  for (tile_image, xs, ys) in resolved_tiles {
    for y in &ys {
      for x in &xs {
        overlay_image(
          &mut composed,
          &tile_image,
          Default::default(),
          Affine::translation(*x as f32, *y as f32),
          context.style.image_rendering,
          context.style.filter.as_ref(),
          None,
        );
      }
    }
  }

  Ok(Some(composed))
}
