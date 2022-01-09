use std::collections::HashMap;
use std::fmt::{Display, Formatter};

use indextree::{Arena, NodeId};

use crate::compiler::WritableStack;
use crate::operation::OperationType;
pub struct OperationTree {
  arena: Arena<OperationType>,
  root: NodeId,
  cur: NodeId,
  child_count: HashMap<NodeId, usize>,
}

impl OperationTree {
  pub fn new(class_name: String) -> Self {
    let mut arena = Arena::new();
    let root = arena.new_node(OperationType::Class(class_name));
    let mut child_count = HashMap::new();
    child_count.insert(root, 0);
    Self {
      arena: arena,
      root,
      cur: root,
      child_count: child_count,
    }
  }

  pub fn root(&self) -> NodeId {
    self.root
  }

  pub fn travel(&self, root: NodeId) -> indextree::Children<OperationType> {
    root.children(&self.arena).into_iter()
  }

  pub fn get_node(&self, node_id: NodeId) -> &indextree::Node<OperationType> {
    self.arena.get(node_id).unwrap()
  }

  pub fn get_next_n_sub(&self, node_id: NodeId, n: usize) -> Option<NodeId> {
    node_id.following_siblings(&self.arena).skip(n).next()
  }

  pub fn get_following_sibling(
    &self,
    node_id: NodeId,
  ) -> indextree::FollowingSiblings<OperationType> {
    node_id.following_siblings(&self.arena)
  }

  pub fn get_children(&self, node: NodeId) -> indextree::Children<OperationType> {
    node.children(&self.arena)
  }

  pub fn get_mut_node(&mut self, node_id: NodeId) -> &mut indextree::Node<OperationType> {
    self.arena.get_mut(node_id).unwrap()
  }

  pub fn dfs_fmt(&self, node_id: NodeId, f: &mut Formatter<'_>, dep: usize) -> std::fmt::Result {
    let mut ret = write!(f, "{}", "-".repeat(dep));
    if ret.is_err() {
      return ret;
    }
    ret = writeln!(f, "{:?}\n", self.get_node(node_id).get());
    if ret.is_err() {
      return ret;
    }
    let children = node_id.children(&self.arena);
    for child in children {
      let ret = self.dfs_fmt(child, f, dep + 1);
      if ret.is_err() {
        return ret;
      }
    }
    return Ok(());
  }
}

impl Default for OperationTree {
  fn default() -> Self {
    OperationTree::new("tmp".to_string())
  }
}

impl Display for OperationTree {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    // for entry in &self.child_count {
    //   println!("{:?} {}", self.get_node(*entry.0).get(), entry.1);
    // }
    self.dfs_fmt(self.root, f, 0)
  }
}

impl WritableStack for OperationTree {
  fn push(&mut self, node_type: OperationType) {
    // println!("push: {:?}", node_type);
    let has_child = node_type.has_child();
    let new_node = self.arena.new_node(node_type);
    self.cur.append(new_node, &mut self.arena);
    self.child_count.entry(self.cur).and_modify(|e| *e += 1);
    if has_child {
      self.cur = new_node;
      self.child_count.insert(self.cur, 0);
    }
  }

  fn pop(&mut self) {
    // println!("pop: {:?}", self.arena.get(self.cur).unwrap().get());
    if self.child_count[&self.cur] == 0 {
      let mut ancestors = self.cur.ancestors(&self.arena);
      ancestors.next();
      let next_cur = ancestors.next();
      if next_cur.is_some() {
        self.cur = next_cur.unwrap();
      }
    }
    self.child_count.entry(self.cur).and_modify(|e| *e -= 1);
  }
}
