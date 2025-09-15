use std::{
  io::{Seek, Write},
  sync::mpsc::channel,
};

use image::{
  ExtendedColorType, ImageEncoder, ImageFormat, RgbaImage,
  codecs::{
    jpeg::JpegEncoder,
    png::{CompressionType, FilterType, PngEncoder},
  },
};
use serde::{Deserialize, Serialize};
use taffy::{AvailableSpace, NodeId, Point, TaffyTree, geometry::Size};

use crate::{
  GlobalContext,
  layout::{
    Viewport,
    node::Node,
    style::{Affine, InheritedStyle},
  },
  rendering::{Canvas, create_blocking_canvas_loop, draw_debug_border},
};

use crate::rendering::RenderContext;

/// Stores the context and node for rendering.
struct NodeContext<'ctx, N: Node<N>> {
  context: RenderContext<'ctx>,
  node: N,
}

/// Output format for the rendered image.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ImageOutputFormat {
  /// WebP format, suitable for web images with good compression.
  WebP,
  /// AVIF format, even better compression than WebP, but requires more CPU time to encode.
  #[cfg(feature = "avif")]
  Avif,
  /// PNG format, lossless and supports transparency.
  Png,
  /// JPEG format, lossy compression suitable for photographs.
  Jpeg,
}

impl ImageOutputFormat {
  /// Returns the MIME type for the image output format.
  pub fn content_type(&self) -> &'static str {
    match self {
      ImageOutputFormat::WebP => "image/webp",
      #[cfg(feature = "avif")]
      ImageOutputFormat::Avif => "image/avif",
      ImageOutputFormat::Png => "image/png",
      ImageOutputFormat::Jpeg => "image/jpeg",
    }
  }
}

impl From<ImageOutputFormat> for ImageFormat {
  fn from(format: ImageOutputFormat) -> Self {
    match format {
      ImageOutputFormat::WebP => Self::WebP,
      #[cfg(feature = "avif")]
      ImageOutputFormat::Avif => Self::Avif,
      ImageOutputFormat::Png => Self::Png,
      ImageOutputFormat::Jpeg => Self::Jpeg,
    }
  }
}

/// Writes the rendered image to the specified destination.
pub fn write_image<T: Write + Seek>(
  image: &RgbaImage,
  destination: &mut T,
  format: ImageOutputFormat,
  jpeg_quality: Option<u8>,
) -> Result<(), image::ImageError> {
  match format {
    ImageOutputFormat::Jpeg => {
      // Strip alpha channel into a tightly packed RGB buffer
      let raw = image.as_raw();
      let mut rgb = Vec::with_capacity(raw.len() / 4 * 3);
      for px in raw.chunks_exact(4) {
        rgb.extend_from_slice(&px[..3]);
      }

      let encoder = JpegEncoder::new_with_quality(destination, jpeg_quality.unwrap_or(75));
      encoder.write_image(&rgb, image.width(), image.height(), ExtendedColorType::Rgb8)?;
    }
    ImageOutputFormat::Png => {
      let encoder =
        PngEncoder::new_with_quality(destination, CompressionType::Fast, FilterType::Sub);

      encoder.write_image(
        image.as_raw(),
        image.width(),
        image.height(),
        ExtendedColorType::Rgba8,
      )?;
    }
    _ => {
      image.write_to(destination, format.into())?;
    }
  }

  Ok(())
}

/// Renders a node to an image.
pub fn render<Nodes: Node<Nodes>>(
  viewport: Viewport,
  global: &GlobalContext,
  root_node: Nodes,
) -> Result<RgbaImage, crate::Error> {
  let mut taffy = TaffyTree::new();

  let (tx, rx) = channel();
  let canvas = Canvas::new(tx);

  let render_context = RenderContext {
    global,
    viewport,
    font_size: viewport.font_size,
    transform: Affine::identity(),
    style: InheritedStyle::default(),
  };

  let root_node_id = insert_taffy_node(&mut taffy, root_node, &render_context, viewport.font_size);

  let available_space = Size {
    width: AvailableSpace::Definite(viewport.width as f32),
    height: AvailableSpace::Definite(viewport.height as f32),
  };

  taffy
    .compute_layout_with_measure(
      root_node_id,
      available_space,
      |known_dimensions, available_space, _node_id, node_context, _style| {
        let node = node_context.unwrap();

        if let Size {
          width: Some(width),
          height: Some(height),
        } = known_dimensions
        {
          return Size { width, height };
        }

        node
          .node
          .measure(&node.context, available_space, known_dimensions)
      },
    )
    .unwrap();

  #[cfg(target_arch = "wasm32")]
  let canvas = {
    render_node(
      &mut taffy,
      root_node_id,
      &canvas,
      Point::ZERO,
      Affine::identity(),
    );

    drop(canvas);

    create_blocking_canvas_loop(viewport, rx)
  };

  #[cfg(not(target_arch = "wasm32"))]
  let canvas = {
    let handler = std::thread::spawn(move || create_blocking_canvas_loop(viewport, rx));

    render_node(
      &mut taffy,
      root_node_id,
      &canvas,
      Point::ZERO,
      Affine::identity(),
    );

    drop(canvas);

    handler.join().unwrap()
  };

  Ok(canvas)
}

fn render_node<Nodes: Node<Nodes>>(
  taffy: &mut TaffyTree<NodeContext<Nodes>>,
  node_id: NodeId,
  canvas: &Canvas,
  offset: Point<f32>,
  mut transform: Affine,
) {
  let mut layout = *taffy.layout(node_id).unwrap();

  layout.location.x += offset.x;
  layout.location.y += offset.y;

  let node_context = taffy.get_node_context_mut(node_id).unwrap();

  if let Some(node_transform) = &node_context.context.style.transform {
    let node_transform = node_transform.to_affine(
      &node_context.context,
      &layout,
      node_context
        .context
        .style
        .transform_origin
        .unwrap_or_default(),
    );

    transform = transform * node_transform;
  }

  node_context.context.transform = transform;

  node_context
    .node
    .draw_on_canvas(&node_context.context, canvas, layout);

  if node_context.context.global.draw_debug_border {
    draw_debug_border(canvas, layout, node_context.context.transform);
  }

  for child_id in taffy.children(node_id).unwrap() {
    render_node(taffy, child_id, canvas, layout.location, transform);
  }
}

fn insert_taffy_node<'ctx, Nodes: Node<Nodes>>(
  taffy: &mut TaffyTree<NodeContext<'ctx, Nodes>>,
  mut node: Nodes,
  render_context: &RenderContext<'ctx>,
  parent_font_size: f32,
) -> NodeId {
  let children = node.take_children();
  let node_style = node.get_style().inherit(&render_context.style);

  let font_size = node_style
    .font_size
    .resolve_to_px(render_context, parent_font_size);

  let new_context = RenderContext {
    style: node_style.clone(),
    font_size,
    ..*render_context
  };

  let node_id = taffy
    .new_leaf_with_context(
      node_style.to_taffy_style(&new_context),
      NodeContext {
        context: new_context,
        node,
      },
    )
    .unwrap();

  if let Some(children) = children {
    let render_context = RenderContext {
      style: node_style,
      font_size,
      ..*render_context
    };

    let children_ids = children
      .into_iter()
      .map(|child| insert_taffy_node(taffy, child, &render_context, font_size))
      .collect::<Vec<_>>();

    taffy.set_children(node_id, &children_ids).unwrap();
  }

  node_id
}
