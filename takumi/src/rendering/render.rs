use std::{collections::HashMap, sync::Arc};

use derive_builder::Builder;
use image::RgbaImage;
use taffy::{AvailableSpace, NodeId, TaffyTree, geometry::Size};

use crate::{
  GlobalContext,
  layout::{
    Viewport,
    node::Node,
    style::{Affine, Color, Display, InheritedStyle},
    tree::NodeTree,
  },
  rendering::Canvas,
  resources::image::ImageSource,
};

use crate::rendering::RenderContext;

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
  /// The resources fetched externally.
  #[builder(default)]
  pub(crate) fetched_resources: HashMap<Arc<str>, Arc<ImageSource>>,
}

/// Renders a node to an image.
pub fn render<'g, N: Node<N>>(options: RenderOptions<'g, N>) -> Result<RgbaImage, crate::Error> {
  let mut taffy = TaffyTree::new();

  let render_context = RenderContext {
    draw_debug_border: options.draw_debug_border,
    ..RenderContext::new(options.global, options.viewport, options.fetched_resources)
  };

  let tree = NodeTree::from_node(&render_context, options.node);

  let root_node_id = tree.insert_into_taffy(&mut taffy);

  taffy
    .compute_layout_with_measure(
      root_node_id,
      render_context.viewport.into(),
      |known_dimensions, available_space, _node_id, node_context, style| {
        if let Size {
          width: Some(width),
          height: Some(height),
        } = known_dimensions.maybe_apply_aspect_ratio(style.aspect_ratio)
        {
          Size { width, height }
        } else {
          node_context
            .unwrap()
            .measure(available_space, known_dimensions, style)
        }
      },
    )
    .unwrap();

  let root_size = taffy
    .layout(root_node_id)
    .unwrap()
    .size
    .map(|size| size.round() as u32);

  if root_size.width == 0 || root_size.height == 0 {
    return Err(crate::Error::InvalidViewport);
  }

  let root_size = root_size.zip_map(options.viewport.into(), |size, viewport| {
    if let AvailableSpace::Definite(defined) = viewport {
      defined as u32
    } else {
      size
    }
  });

  let mut canvas = Canvas::new(root_size);

  render_node(&mut taffy, root_node_id, &mut canvas, Affine::IDENTITY);

  Ok(canvas.into_inner())
}

fn create_transform(
  mut transform: Affine,
  style: &InheritedStyle,
  border_box: Size<f32>,
  context: &RenderContext,
) -> Affine {
  let transform_origin = style.transform_origin.unwrap_or_default();

  let center = transform_origin.to_point(context, border_box) + transform.decompose_translation();

  transform *= Affine::translation(-center.x, -center.y);

  // https://github.com/servo/servo/blob/9dfd6990ba381cbb7b7f9faa63d3425656ceac0a/components/layout/display_list/stacking_context.rs#L1717-L1720
  if let Some(node_transform) = &*style.transform {
    transform *= node_transform.to_affine(context, border_box);
  }

  if let Some(rotate) = *style.rotate {
    transform *= Affine::rotation(rotate);
  }

  if let Some(scale) = *style.scale {
    transform *= Affine::scale(scale.x.0, scale.y.0);
  }

  if let Some(translate) = *style.translate {
    transform *= Affine::translation(
      translate.x.resolve_to_px(context, border_box.width),
      translate.y.resolve_to_px(context, border_box.height),
    );
  }

  transform *= Affine::translation(center.x, center.y);

  transform
}

fn render_node<'g, Nodes: Node<Nodes>>(
  taffy: &mut TaffyTree<NodeTree<'g, Nodes>>,
  node_id: NodeId,
  canvas: &mut Canvas,
  mut transform: Affine,
) {
  let layout = *taffy.layout(node_id).unwrap();
  let node = taffy.get_node_context_mut(node_id).unwrap();

  if node.context.opacity == 0.0 || node.context.style.display == Display::None {
    return;
  }

  transform = Affine::translation(layout.location.x, layout.location.y) * transform;

  transform = create_transform(transform, &node.context.style, layout.size, &node.context);

  // If a transform function causes the current transformation matrix of an object to be non-invertible, the object and its content do not get displayed.
  // https://drafts.csswg.org/css-transforms/#transform-function-lists
  if !transform.is_invertible() {
    return;
  }

  node.context.transform = transform;

  if let Some(clip) = &node.context.style.clip_path.0 {
    let translation = transform.decompose_translation();

    node.context.transform.x = 0.0;
    node.context.transform.y = 0.0;

    let (mask, mut placement) = clip.render_mask(&node.context, layout.size);

    let mut inner_canvas = Canvas::new(Size {
      width: placement.width,
      height: placement.height,
    });

    node.context.transform =
      Affine::translation(-placement.left as f32, -placement.top as f32) * node.context.transform;

    node.draw_on_canvas(&mut inner_canvas, layout);

    if node.should_create_inline_layout() {
      node.draw_inline(&mut inner_canvas, layout);
    } else {
      for child_id in taffy.children(node_id).unwrap() {
        render_node(taffy, child_id, &mut inner_canvas, transform);
      }
    }

    placement.left += translation.x as i32;
    placement.top += translation.y as i32;

    let inner_image = inner_canvas.into_inner();

    return canvas.draw_mask(
      &mask,
      placement,
      Color::transparent(),
      Some((&inner_image).into()),
    );
  }

  node.draw_on_canvas(canvas, layout);

  let overflow = node.context.style.resolve_overflows();

  if overflow.should_clip_content() {
    let Some(overflow_constrain) = overflow.create_constrain(layout, transform) else {
      return;
    };

    canvas.push_constrain(overflow_constrain);
  }

  if node.should_create_inline_layout() {
    node.draw_inline(canvas, layout);
  } else {
    for child_id in taffy.children(node_id).unwrap() {
      render_node(taffy, child_id, canvas, transform);
    }
  }

  if overflow.should_clip_content() {
    canvas.pop_constrain();
  }
}
