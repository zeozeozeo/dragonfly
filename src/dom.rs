use crate::{Declaration, FontManager, Pos2, Vec2};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct DOMNode {
    pub pos: Pos2,
    pub name: String,
    pub attrs: HashMap<String, String>,
    pub id: String,
    pub style: Option<Declaration>,
    /// Node text, if it is a text node. This is NOT the inner text of the node,
    /// this is a part of the inner text of another node!!
    pub text: String,
}

impl Default for DOMNode {
    fn default() -> Self {
        Self {
            pos: Pos2::new(0.0, 0.0),
            name: String::new(),
            attrs: HashMap::new(),
            id: String::new(),
            style: None,
            text: String::new(),
        }
    }
}

impl DOMNode {
    /// Create a new node with an element name.
    pub fn new(name: &str) -> Self {
        Self {
            pos: Pos2::new(0.0, 0.0),
            name: name.to_string(),
            attrs: HashMap::new(),
            id: String::new(),
            style: None,
            text: String::new(),
        }
    }

    pub fn text_node(text: &str) -> Self {
        let mut node = Self::default();
        node.set_text(text);
        node
    }

    /// Updates the node text (string with no consecutive whitespace)
    ///
    /// Only do this if it is a text node. This is not meant for setting the inner text of the node.
    pub fn set_text(&mut self, text: &str) {
        self.text = String::new();
        let get_char = |pos: usize| text[pos..].chars().next().unwrap();

        for i in 0..text.len() {
            let c = get_char(i);
            if i > 0 && c.is_whitespace() && get_char(i - 1).is_whitespace() {
                continue;
            }
            self.text.push(if c.is_whitespace() { ' ' } else { c });
        }
        log::debug!("set node text: '{}'", self.text);
    }

    /// Root `html` node.
    pub fn root() -> Self {
        let mut node = Self::default();
        node.name = String::from("html");
        node
    }

    pub fn bounds(&self, fonts: &mut FontManager) {
        // calculate text size in node
        let mut bounds = Vec2::new(0.0, 0.0);
        for c in self.text.chars() {
            let metrics = fonts.glyph_metrics(
                c,
                14.0,
                self.style
                    .clone()
                    .unwrap_or_default()
                    .font_family
                    .unwrap_or_default(),
            );
            bounds.x += metrics.width as f32 + metrics.advance_width;
            log::debug!("char '{c}' metrics: {metrics:?}");
        }
        log::debug!("calculated node bounds: {bounds:?}");
    }
}
