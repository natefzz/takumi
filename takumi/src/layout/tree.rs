use std::mem::take;

use taffy::{AvailableSpace, NodeId, Size, TaffyTree};

use crate::{
  layout::{
    inline::InlineTree,
    node::Node,
    style::{Display, InheritedStyle},
  },
  rendering::RenderContext,
};

pub(crate) struct NodeTreeItem<'g, N: Node<N>> {
  context: RenderContext<'g>,
  node: Option<N>,
  children: Option<Vec<NodeTreeItem<'g, N>>>,
}

pub(crate) struct TaffyContext<'g, N: Node<N>> {
  pub context: RenderContext<'g>,
  pub content: RenderContent<'g, N>,
}

pub(crate) enum RenderContent<'g, N: Node<N>> {
  Node(Option<N>),
  Inline(InlineTree<'g, N>),
}

impl<'g, N: Node<N>> TaffyContext<'g, N> {
  pub(crate) fn measure(
    &self,
    available_space: Size<AvailableSpace>,
    known_dimensions: Size<Option<f32>>,
  ) -> Size<f32> {
    match &self.content {
      RenderContent::Node(None) => Size::zero(),
      RenderContent::Node(Some(node)) => {
        node.measure(&self.context, available_space, known_dimensions)
      }
      RenderContent::Inline(tree) => tree.measure(&self.context, available_space, known_dimensions),
    }
  }
}

impl<'g, N: Node<N>> NodeTreeItem<'g, N> {
  pub fn is_inline(&self) -> bool {
    self.context.style.display == Display::Inline
  }

  pub fn from_node(parent_context: &RenderContext<'g>, mut node: N) -> Self {
    let style = node.take_style().inherit(&parent_context.style);

    // First resolves the font size for this node from parent font size
    let font_size = style
      .font_size
      .map(|font_size| font_size.resolve_to_px(parent_context, parent_context.font_size))
      .unwrap_or(parent_context.font_size);

    let current_color = style.color.resolve(parent_context.current_color);

    // Overrides the font size placeholder to the resolved font size
    let mut context = RenderContext {
      style,
      font_size,
      current_color,
      ..*parent_context
    };

    let children = node.take_children().map(|children| {
      children
        .into_iter()
        .map(|child| Self::from_node(&context, child))
        .collect::<Vec<_>>()
    });

    let Some(children) = children else {
      return Self {
        context,
        node: Some(node),
        children: None,
      };
    };

    if !context.style.display.is_inline() || children.iter().all(NodeTreeItem::is_inline) {
      return Self {
        context,
        node: Some(node),
        children: Some(children),
      };
    }

    context.style.display = context.style.display.to_block();

    let mut final_children = Vec::new();
    let mut inline_group = Vec::new();

    for item in children {
      if !item.is_inline() {
        if !inline_group.is_empty() {
          final_children.push(NodeTreeItem {
            context: RenderContext {
              style: InheritedStyle::default(),
              ..context
            },
            children: Some(take(&mut inline_group)),
            node: None,
          });
        }

        final_children.push(item);
        continue;
      }

      inline_group.push(item);
    }

    Self {
      context,
      node: Some(node),
      children: Some(final_children),
    }
  }

  pub(crate) fn insert_into_taffy(self, tree: &mut TaffyTree<TaffyContext<'g, N>>) -> NodeId {
    if self.is_inline() {
      let mut inline_tree = InlineTree::new();
      let context = self.context.clone();

      self.collect_inline_nodes(&mut inline_tree);

      return tree
        .new_leaf_with_context(
          context.style.to_taffy_style(&context),
          TaffyContext {
            context,
            content: RenderContent::Inline(inline_tree),
          },
        )
        .unwrap();
    }

    let node_id = tree
      .new_leaf_with_context(
        self.context.style.to_taffy_style(&self.context),
        TaffyContext {
          context: self.context,
          content: RenderContent::Node(self.node),
        },
      )
      .unwrap();

    if let Some(children) = self.children {
      let children_ids = children
        .into_iter()
        .map(|child| child.insert_into_taffy(tree))
        .collect::<Vec<_>>();

      tree.set_children(node_id, &children_ids).unwrap();
    }

    node_id
  }

  fn collect_inline_nodes(self, tree: &mut InlineTree<'g, N>) {
    if let Some(node) = self.node {
      tree.try_insert_node(node, self.context);
    }

    if let Some(children) = self.children {
      for child in children {
        child.collect_inline_nodes(tree);
      }
    }
  }
}
