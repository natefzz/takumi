use std::{collections::HashMap, sync::Arc};

use derive_builder::Builder;
use image::RgbaImage;
use taffy::{AvailableSpace, Layout, NodeId, Point, TaffyTree, geometry::Size};

use crate::{
  GlobalContext,
  layout::{
    Viewport,
    node::Node,
    style::{Affine, Color, Display, InheritedStyle, LengthUnit, Overflow},
    tree::NodeTree,
  },
  rendering::{BorderProperties, Canvas},
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

  render_node(
    &mut taffy,
    root_node_id,
    &mut canvas,
    Affine::identity(),
    root_size,
  );

  Ok(canvas.into_inner())
}

fn create_transform(
  style: &InheritedStyle,
  border_box: Size<f32>,
  context: &RenderContext,
) -> Affine {
  let transform_origin = style.transform_origin.0.unwrap_or_default();

  let center_x = LengthUnit::from(transform_origin.0.x).resolve_to_px(context, border_box.width);
  let center_y = LengthUnit::from(transform_origin.0.y).resolve_to_px(context, border_box.height);

  let mut transform = zeno::Transform::translation(-center_x, -center_y);

  // https://github.com/servo/servo/blob/9dfd6990ba381cbb7b7f9faa63d3425656ceac0a/components/layout/display_list/stacking_context.rs#L1717-L1720
  if let Some(node_transform) = &*style.transform {
    transform = transform.then(&node_transform.to_affine(context, border_box));
  }

  if let Some(rotate) = *style.rotate {
    transform = transform.then_rotate(rotate.into());
  }

  if let Some(scale) = *style.scale {
    transform = transform.then_scale(scale.x.0, scale.y.0);
  }

  if let Some(translate) = *style.translate {
    transform = transform.then_translate(
      translate.x.resolve_to_px(context, border_box.width),
      translate.y.resolve_to_px(context, border_box.height),
    );
  }

  transform.then_translate(center_x, center_y).into()
}

fn render_node<'g, Nodes: Node<Nodes>>(
  taffy: &mut TaffyTree<NodeTree<'g, Nodes>>,
  node_id: NodeId,
  canvas: &mut Canvas,
  mut transform: Affine,
  root_size: Size<u32>,
) {
  let layout = *taffy.layout(node_id).unwrap();
  let node_context = taffy.get_node_context_mut(node_id).unwrap();

  if node_context.context.opacity == 0.0 || node_context.context.style.display == Display::None {
    return;
  }

  transform = transform
    .then_translate(layout.location.x, layout.location.y)
    .then(&create_transform(
      &node_context.context.style,
      layout.size,
      &node_context.context,
    ))
    .into();

  node_context.context.transform = transform;

  if let Some(clip) = &node_context.context.style.clip_path.0 {
    let translation = node_context.context.transform.decompose_translation();

    node_context.context.transform = node_context
      .context
      .transform
      .then_translate(-translation.x, -translation.y)
      .into();

    let (mask, mut placement) = clip.render_mask(&node_context.context, layout.size);

    node_context.context.transform = node_context
      .context
      .transform
      .then_translate(-placement.left as f32, -placement.top as f32)
      .into();

    let mut inner_canvas = Canvas::new(Size {
      width: placement.width,
      height: placement.height,
    });

    let inner_layout = Layout {
      location: Point::zero(),
      ..layout
    };

    node_context.draw_on_canvas(&mut inner_canvas, inner_layout);

    if node_context.should_create_inline_layout() {
      node_context.draw_inline(&mut inner_canvas, inner_layout);
    } else {
      for child_id in taffy.children(node_id).unwrap() {
        render_node(taffy, child_id, &mut inner_canvas, transform, root_size);
      }
    }

    placement.left += translation.x as i32;
    placement.top += translation.y as i32;

    return canvas.draw_mask(
      &mask,
      placement,
      Color::transparent(),
      Some(inner_canvas.into_inner()),
    );
  }

  node_context.draw_on_canvas(canvas, layout);

  let overflow = node_context.context.style.resolve_overflows();

  if overflow.should_clip_content() {
    // if theres no space for canvas to draw, just return.
    let Some(mut inner_canvas) = overflow.create_clip_canvas(root_size, layout) else {
      return;
    };

    let image_rendering = node_context.context.style.image_rendering;
    let filters = node_context.context.style.filter.0.clone();

    let offset = Point {
      x: if overflow.0.x == Overflow::Visible {
        layout.location.x
      } else {
        0.0
      },
      y: if overflow.0.y == Overflow::Visible {
        layout.location.y
      } else {
        0.0
      },
    };

    if node_context.should_create_inline_layout() {
      node_context.draw_inline(
        &mut inner_canvas,
        Layout {
          size: layout.content_box_size(),
          location: offset,
          ..Default::default()
        },
      );
    } else {
      for child_id in taffy.children(node_id).unwrap() {
        render_node(taffy, child_id, &mut inner_canvas, transform, root_size);
      }
    }

    return canvas.overlay_image(
      &inner_canvas.into_inner(),
      BorderProperties::zero(),
      transform,
      image_rendering,
      filters.as_ref(),
    );
  }

  if node_context.should_create_inline_layout() {
    node_context.draw_inline(canvas, layout);
  } else {
    for child_id in taffy.children(node_id).unwrap() {
      render_node(taffy, child_id, canvas, transform, root_size);
    }
  }
}
