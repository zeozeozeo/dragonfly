use crate::{DOMNode, Declaration, FontManager, GlobalStyle};
use ego_tree::NodeRef as EgoNodeRef;
use indextree::{Arena, NodeId};
use scraper::{node::Element, Html};

#[derive(Debug, Clone)]
pub struct Layout {
    /// DOM node arena. Has a root node by default.
    pub arena: Arena<DOMNode>,
    root_id: NodeId,
    pub style: GlobalStyle,
}

impl Default for Layout {
    fn default() -> Self {
        let mut arena = Arena::new();
        let root_id = arena.new_node(DOMNode::root());
        Self {
            arena,
            root_id,
            style: GlobalStyle::default_css(),
        }
    }
}

impl Layout {
    pub fn compute(document: &mut Html, fonts: &mut FontManager) -> Self {
        let mut layout = Self::default();
        let root = document.tree.root();

        // compute all nodes recursively
        layout.compute_node(root, 0, layout.root_id, fonts);

        log::debug!("computed layout tree:\n{:?}", layout.arena);
        layout
    }

    fn compute_node(
        &mut self,
        html_node: EgoNodeRef<'_, scraper::Node>,
        depth: usize,
        parent: NodeId,
        fonts: &mut FontManager,
    ) {
        if html_node.value().is_element() {
            log::info!(
                "compute node {}, recurse depth {depth}",
                html_node.value().as_element().unwrap().name()
            );
        }

        let parent = match html_node.value() {
            scraper::Node::Element(el) => self.handle_element(el, parent, fonts),
            scraper::Node::Text(text) => {
                log::debug!("adding text to parent node {parent:?}",);
                parent.append_value(DOMNode::text_node(text), &mut self.arena);
                parent
            }
            _ => {
                log::warn!("unhandled html node {:?}", html_node.value());
                parent
            }
        };

        for child in html_node.children() {
            self.compute_node(child, depth + 1, parent, fonts);
        }
    }

    fn handle_element(&mut self, el: &Element, parent: NodeId, fonts: &mut FontManager) -> NodeId {
        let el_name = el.name();
        log::debug!("layout element '{}'", el_name);

        // create new node
        let mut node = DOMNode::new(el_name);

        // process node attrs
        for attr in el.attrs() {
            node.attrs.insert(attr.0.to_string(), attr.1.to_string());
            log::debug!("parsing attribute: {:?}", attr);

            match attr.0 {
                "style" => node.style = Some(Declaration::from_inline(attr.1)),
                _ => log::warn!("unhandled attribute '{}'", attr.0),
            }
        }

        // add node to document
        self.add_node(node, parent, fonts)
    }

    fn add_node(&mut self, node: DOMNode, parent: NodeId, fonts: &mut FontManager) -> NodeId {
        let node_id = match node.name.as_str() {
            "html" => {
                log::debug!("update root node");
                *self.arena.get_mut(self.root_id).unwrap().get_mut() = node.clone();
                self.root_id
            }
            _ => parent.append_value(node, &mut self.arena),
        };

        // get mutable node ref of parent
        let node = self.arena.get_mut(node_id).unwrap().get_mut();

        // compute node bounds
        node.bounds(fonts);

        /*
        log::debug!(
            "node attrs: {:?}; computing node bounds",
            node.value().attrs
        );
        log::debug!(
            "character metrics: {:?}",
            fonts.sans_serif().metrics('R', 16.0),
        );
        */

        // return node id (will be used as a parent of children nodes)
        node_id
    }
}
