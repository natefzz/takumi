use std::sync::mpsc::channel;

use derive_builder::Builder;
use image::RgbaImage;
use taffy::{AvailableSpace, Layout, NodeId, Point, TaffyTree, geometry::Size};

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

#[derive(Clone, Builder)]
/// Options for rendering a node. Construct using [`RenderOptionsBuilder`] to avoid breaking changes.
pub struct RenderOptions<'g, N: Node<N>> {
  /// The viewport to render the node in.
  pub(crate) viewport: Viewport,
  /// The global context.
  pub(crate) global: &'g GlobalContext,
  /// The node to render.
  pub(crate) node: N,
  /// Whether to draw debug borders.
  #[builder(default)]
  pub(crate) draw_debug_border: bool,
}

/// Renders a node to an image.
pub fn render<'g, N: Node<N>>(options: RenderOptions<'g, N>) -> Result<RgbaImage, crate::Error> {
  let mut taffy = TaffyTree::new();

  let (tx, rx) = channel();
  let canvas = Canvas::new(tx);

  let render_context = (&options).into();

  let root_node_id = insert_taffy_node(&mut taffy, options.node, &render_context);

  let available_space = Size {
    width: AvailableSpace::Definite(render_context.viewport.width as f32),
    height: AvailableSpace::Definite(render_context.viewport.height as f32),
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

    create_blocking_canvas_loop(render_context.viewport, rx)
  };

  #[cfg(not(target_arch = "wasm32"))]
  let canvas = {
    let handler =
      std::thread::spawn(move || create_blocking_canvas_loop(render_context.viewport, rx));

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

fn create_transform(style: &InheritedStyle, layout: &Layout, context: &RenderContext) -> Affine {
  let mut transform = Affine::identity();

  let transform_origin = style.transform_origin.0.unwrap_or_default();

  let center = Point {
    x: transform_origin
      .x
      .to_length_unit()
      .resolve_to_px(context, layout.size.width),
    y: transform_origin
      .y
      .to_length_unit()
      .resolve_to_px(context, layout.size.height),
  };

  // According to https://www.w3.org/TR/css-transforms-2/#ctm
  // the order is `translate` -> `rotate` -> `scale` -> `transform`
  if let Some(translate) = *style.translate {
    transform = transform
      * Affine::translation(Size {
        width: translate.x.resolve_to_px(context, layout.size.width),
        height: translate.y.resolve_to_px(context, layout.size.height),
      });
  }

  if let Some(rotate) = *style.rotate {
    transform = transform * Affine::rotation(rotate, center);
  }

  if let Some(scale) = *style.scale {
    transform = transform * Affine::scale(scale.into(), center);
  }

  if let Some(node_transform) = &*style.transform {
    let node_transform = node_transform.to_affine(context, layout, center);

    transform = transform * node_transform;
  }

  transform
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

  transform =
    transform * create_transform(&node_context.context.style, &layout, &node_context.context);

  node_context.context.transform = transform;

  node_context
    .node
    .draw_on_canvas(&node_context.context, canvas, layout);

  if node_context.context.draw_debug_border {
    draw_debug_border(canvas, layout, node_context.context.transform);
  }

  for child_id in taffy.children(node_id).unwrap() {
    render_node(taffy, child_id, canvas, layout.location, transform);
  }
}

fn insert_taffy_node<'ctx, Nodes: Node<Nodes>>(
  taffy: &mut TaffyTree<NodeContext<'ctx, Nodes>>,
  mut node: Nodes,
  parent_context: &RenderContext<'ctx>,
) -> NodeId {
  let children = node.take_children();
  let node_style = node.get_style().inherit(&parent_context.style);

  // First resolves the font size for this node from parent font size
  let font_size = node_style
    .font_size
    .map(|font_size| font_size.resolve_to_px(parent_context, parent_context.font_size))
    .unwrap_or(parent_context.font_size);

  // Overrides the font size placeholder to the resolved font size
  let child_context = RenderContext {
    style: node_style,
    font_size,
    ..*parent_context
  };

  let node_id = taffy
    .new_leaf_with_context(
      child_context.style.to_taffy_style(&child_context),
      NodeContext {
        context: child_context.clone(),
        node,
      },
    )
    .unwrap();

  if let Some(children) = children {
    let children_ids = children
      .into_iter()
      .map(|child| insert_taffy_node(taffy, child, &child_context))
      .collect::<Vec<_>>();

    taffy.set_children(node_id, &children_ids).unwrap();
  }

  node_id
}
