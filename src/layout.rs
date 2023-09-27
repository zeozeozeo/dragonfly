use crate::{DOMTree, Declaration, FontStorage, Node, Pos2};
use ego_tree::{NodeId, NodeRef};
use scraper::{node::Element, Html};

#[derive(Debug, Clone)]
pub struct Layout {
    pub tree: DOMTree,
}

impl Default for Layout {
    fn default() -> Self {
        Self {
            tree: DOMTree::new(Node::root()),
        }
    }
}

impl Layout {
    pub fn compute(document: &mut Html, fonts: &FontStorage) -> Self {
        let mut layout = Self::default();

        let root = document.tree.root();

        // compute all nodes recursively
        let root_id = layout.tree.root_mut().id();
        layout.compute_node(root, 0, fonts, root_id);

        log::debug!("HTML tree:\n{}", layout);
        layout
    }

    fn compute_node(
        &mut self,
        html_node: NodeRef<'_, scraper::Node>,
        depth: usize,
        fonts: &FontStorage,
        parent: NodeId,
    ) {
        if html_node.value().is_element() {
            log::info!(
                "compute node {}, recurse depth {depth}",
                html_node.value().as_element().unwrap().name()
            );
        }

        let parent = match html_node.value() {
            scraper::Node::Element(el) => self.handle_element(el, fonts, parent),
            _ => {
                log::warn!("unhandled html node {:?}", html_node.value());
                parent
            }
        };

        for child in html_node.children() {
            self.compute_node(child, depth + 1, fonts, parent);
        }
    }

    fn handle_element(&mut self, el: &Element, fonts: &FontStorage, parent: NodeId) -> NodeId {
        let el_name = el.name();
        log::debug!("layout element '{}'", el_name);

        // create new node
        let mut node = Node::new(el_name);

        // process node attrs
        for attr in el.attrs() {
            node.attrs.insert(attr.0.to_string(), attr.1.to_string());
            log::debug!("parsing attribute: {:?}", attr);

            match attr.0 {
                "style" => node.style = Declaration::from_inline(attr.1),
                "lang" => log::debug!("ignoring lang attribute, value = '{}'", attr.1),
                _ => log::warn!("unhandled attribute '{}'", attr.0),
            }
        }

        // add node to document
        node.pos = Pos2::new(9.0, 28.0);
        let mut parent = self.tree.get_mut(parent).unwrap();

        let mut node = match el_name {
            "html" => {
                log::debug!("update root node");
                *self.tree.root_mut().value() = node.clone();
                self.tree.root_mut()
            }
            _ => parent.append(node),
        };

        log::debug!(
            "node attrs: {:?}; computing node bounds",
            node.value().attrs
        );
        log::debug!(
            "character metrics: {:?}",
            fonts.sans_serif().metrics('R', 16.0),
        );

        node.id()
    }

    fn format_node(&self, s: &mut String, node: NodeId, depth: usize) {
        for child in self.tree.get(node).unwrap().children() {
            s.push_str(&("  ".repeat(depth) + &format!("{:?}\n", child.value())));
            self.format_node(s, child.id(), depth + 1);
        }
    }
}

impl std::fmt::Display for Layout {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = format!("{:?}\n", self.tree.root().value());
        for child in self.tree.root().children() {
            self.format_node(&mut s, child.id(), 0);
        }
        write!(f, "{}", s)?;
        Ok(())
    }
}
