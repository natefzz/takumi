//! Container node implementation for the takumi layout system.
//!
//! This module contains the ContainerNode struct which is used to group
//! other nodes and apply layout properties like flexbox layout.

use std::fmt::Debug;

use serde::{Deserialize, Serialize};

use crate::layout::{node::Node, style::Style};

/// A container node that can hold child nodes.
///
/// Container nodes are used to group other nodes and apply layout
/// properties like flexbox layout to arrange their children.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ContainerNode<Nodes: Node<Nodes>> {
  /// The styling properties for this container
  pub style: Option<Style>,
  /// The child nodes contained within this container
  pub children: Option<Vec<Nodes>>,
}

impl<Nodes: Node<Nodes>> Node<Nodes> for ContainerNode<Nodes> {
  fn take_style(&mut self) -> Style {
    self.style.take().unwrap_or_default()
  }

  fn take_children(&mut self) -> Option<Vec<Nodes>> {
    self.children.take()
  }
}
