use ego_tree::Tree;

use crate::{Declaration, Pos2};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Node {
    pub pos: Pos2,
    pub name: String,
    pub attrs: HashMap<String, String>,
    pub style: Declaration,
    pub id: String,
}

impl Default for Node {
    fn default() -> Self {
        Self {
            pos: Pos2::new(0.0, 0.0),
            name: String::from(""),
            attrs: HashMap::new(),
            style: Declaration::default(),
            id: String::from(""),
        }
    }
}

impl Node {
    /// Create a new node with an element name.
    pub fn new(name: &str) -> Self {
        Self {
            pos: Pos2::new(0.0, 0.0),
            name: name.to_string(),
            attrs: HashMap::new(),
            style: Declaration::default(),
            id: String::from(""),
        }
    }

    /// Root `html` node.
    pub fn root() -> Self {
        let mut node = Self::default();
        node.name = String::from("html");
        node
    }
}

pub type DOMTree = Tree<Node>;
