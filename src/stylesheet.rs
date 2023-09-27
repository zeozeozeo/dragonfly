use css_color::Srgb;
use std::str::FromStr;
use strum_macros::{Display, EnumString};

#[derive(Debug, Clone, Copy, Display, Default, EnumString)]
pub enum Position {
    /// Default. The element renders in the order as they appear in the document flow
    #[strum(serialize = "static")]
    #[default]
    Static,
    /// The element is positioned relative to its normal position
    #[strum(serialize = "relative")]
    Relative,
    /// The element is positioned relative to a positioned (not static) parent element
    #[strum(serialize = "absolute")]
    Absolute,
    /// The element is positioned relative to the browser window
    #[strum(serialize = "fixed")]
    Fixed,
    /// The element is positioned based on the scrolling position of a parent element
    #[strum(serialize = "sticky")]
    Sticky,
}

#[derive(Debug, Clone, Copy, Display, Default, EnumString)]
pub enum FontFamily {
    /// Glyphs have finishing strokes, flared or tapering ends, or have actual serifed endings.
    #[strum(serialize = "serif")]
    #[default]
    Serif,
    /// Glyphs have stroke endings that are plain.
    #[strum(serialize = "sans-serif")]
    SansSerif,
    /// All glyphs have the same fixed width.
    #[strum(serialize = "monospace")]
    Monospace,
    /// Glyphs in cursive fonts generally have either joining strokes or
    /// other cursive characteristics beyond those of italic typefaces.
    /// The glyphs are partially or completely connected, and the result
    /// looks more like handwritten pen or brush writing than printed letter work.
    #[strum(serialize = "cursive")]
    Cursive,
    /// Fantasy fonts are primarily decorative fonts that contain playful representations of characters.
    #[strum(serialize = "fantasy")]
    Fantasy,
    /// Glyphs are taken from the default user interface font on a given platform.
    /// Because typographic traditions vary widely across the world,
    /// this generic is provided for typefaces that don't map cleanly into the other generics.
    #[strum(serialize = "system-ui")]
    SystemUi,
    /// The default user interface serif font.
    #[strum(serialize = "ui-serif")]
    UiSerif,
    /// The default user interface sans-serif font.
    #[strum(serialize = "ui-sans-serif")]
    UiSansSerif,
    /// The default user interface monospace font.
    #[strum(serialize = "ui-monospace")]
    UiMonospace,
    /// The default user interface font that has rounded features.
    #[strum(serialize = "ui-rounded")]
    UiRounded,
    /// This is for the particular stylistic concerns of representing mathematics:
    /// superscript and subscript, brackets that cross several lines, nesting expressions,
    /// and double struck glyphs with distinct meanings.
    #[strum(serialize = "math")]
    Math,
    /// Fonts that are specifically designed to render emoji.
    #[strum(serialize = "emoji")]
    Emoji,
    /// A particular style of Chinese characters that are between serif-style Song and
    /// cursive-style Kai forms. This style is often used for government documents.
    #[strum(serialize = "fangsong")]
    Fangsong,
}

/// CSS rule declaration for one or multiple selectors.
#[derive(Debug, Clone)]
pub struct Declaration {
    pub position: Position,
    pub color: Option<Srgb>,
}

impl Default for Declaration {
    fn default() -> Self {
        Self {
            position: Position::default(),
            color: None,
        }
    }
}

impl Declaration {
    /// Parse an inline style string into a stylesheet.
    ///
    /// # Example
    ///
    /// ```rust
    /// use dragonfly::Style;
    /// let style = Style::from_inline("position: absolute; color: red;");
    /// let style = Style::from_inline("color: yellow");
    /// ```
    pub fn from_inline(css: &str) -> Self {
        let mut style = Self::default();
        for attr in css.split(';') {
            let mut parts = attr.split(':');

            // TODO: should we convert those to lowercase?
            let key = parts.next().unwrap_or("").trim();
            let value = parts.next().unwrap_or("").trim();

            // don't attempt to parse failed values
            if key.is_empty() && value.is_empty() {
                continue;
            }

            log::debug!("parsing CSS attribute: '{key}': '{value}'");
            match key {
                "position" => {
                    style.position = Position::from_str(value).unwrap_or(Position::default())
                }
                "color" => style.color = value.parse().ok(),
                _ => log::warn!("unhandled CSS attribute: '{key}'"),
            }
        }

        log::debug!("parsed inline stylesheet: {style:?}");
        style
    }

    pub fn from_css(css: &str, mode: ParserMode) -> Self {
        let mut parser = CssParser::new(css, mode);
        parser.parse()
        // Self::default()
    }
}

/// Remove all block comments & extra whitespace (multiple consecutive whitespace characters) from a string.
///
/// Note that this does not remove nested comments.
///
/// # Example
///
/// ```rust
/// use dragonfly::remove_comments;
/// assert!(remove_comments("body{/* comment */color:/**/red/* hi */;}") == "body{color:red;}");
/// ```
pub fn remove_comments_and_extra_whitespace(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let get_char = |pos: usize| s[pos..].chars().next().unwrap();
    let mut in_comment = false;
    for i in 0..s.len() {
        let c = get_char(i);
        if c == '/' && get_char(i + 1) == '*' {
            in_comment = true;
        } else if i > 2 && get_char(i - 2) == '*' && get_char(i - 1) == '/' {
            in_comment = false;
        }
        // if previous and current chars are whitespace or if we're in a comment, skip the current character
        if !in_comment && !(i > 0 && c.is_whitespace() && get_char(i - 1).is_whitespace()) {
            if c.is_whitespace() {
                result.push(' '); // if we hit any whitespace, just push a space
            } else {
                result.push(c); // if it's not a whitespace, push it to the string
            }
        }
    }
    result
}

/// Represents the parsing behaviour of the CSS parser.
#[derive(Debug, Clone, Copy)]
pub enum ParserMode {
    /// Parse regular CSS files.
    Normal,
    /// Parse inline stylesheets.
    Inline,
    /// Parse the browsers `default.css` file.
    DefaultCss,
}

/// Dragonfly's CSS parser. Does not use any external libraries except for [`css_color`] for color parsing.
#[derive(Debug, Clone)]
pub struct CssParser {
    input: String,
    pos: usize,
    brace_level: usize,
    selector: Option<String>,
    attr_name: Option<String>,
    decl: Declaration,
    mode: ParserMode,
}

impl CssParser {
    pub fn new(css: &str, mode: ParserMode) -> Self {
        let input = remove_comments_and_extra_whitespace(css);
        log::debug!("processed input string: '{input}'");
        Self {
            input,
            pos: 0,
            brace_level: 0,
            selector: None,
            attr_name: None,
            decl: Declaration::default(),
            mode,
        }
    }

    fn eof(&self) -> bool {
        self.pos >= self.input.len()
    }

    /// Return the current character
    fn peek(&self) -> char {
        self.input[self.pos..].chars().next().unwrap()
    }

    /// Consume the current character, return the consumed character
    fn consume(&mut self) -> char {
        let mut iter = self.input[self.pos..].char_indices();
        let (_, cur_char) = iter.next().unwrap();
        let (next_pos, _) = iter.next().unwrap_or((1, ' '));
        self.pos += next_pos;
        return cur_char;
    }

    fn consume_while<F: Fn(char) -> bool>(&mut self, test: F) -> String {
        let mut s = String::new();
        while !self.eof() && test(self.peek()) {
            s.push(self.consume());
        }
        s
    }

    fn consume_name(&mut self) -> String {
        self.consume_while(|c| match c {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' => true,
            _ => false,
        })
    }

    fn replace_browser_keyword(value: &str) -> &str {
        match value {
            "DfTextColor" => "black",
            "DfPageBackgroundColor" => "white",
            "DfButtonBorderColor" => "gray",
            "DfInputPlaceholderTextColor" => "gray",
            "DfButtonBackgroundColor" => "lightgray",
            "DfButtonTextColor" => "black",
            "DfLinkColor" => "lightblue",
            "DfVisitedColor" => "purple",
            "DfActiveColor" => "blue",
            "DfMarkBackgroundColor" => "lightgray",
            "DfMarkTextColor" => "yellow",
            "DfFieldsetBorderColor" => "black",
            _ => value,
        }
    }

    fn parse_attr_value(&mut self, value: &str) {
        let attr_name = self.attr_name.clone().unwrap();
        log::debug!("parsing attr '{attr_name}: {value}'");
        let value = match self.mode {
            ParserMode::DefaultCss => Self::replace_browser_keyword(value),
            _ => value,
        };
        log::debug!("new value (mode: {:?}) => '{value}'", self.mode);

        match attr_name.as_str() {
            "position" => {
                self.decl.position = Position::from_str(&value).unwrap_or(Position::default())
            }
            "color" => self.decl.color = Srgb::from_str(&value).ok(),
            _ => {
                log::warn!("unhandled attr '{attr_name}'")
            }
        }

        log::debug!("declparse step: {:?}", self.decl);
    }

    fn advance(&mut self) {
        let c = self.peek();
        match c {
            '{' => {
                self.consume();
                self.brace_level += 1;
            }
            '}' => {
                self.consume();
                self.brace_level = self.brace_level.saturating_sub(1); // if already 0, we don't want a panic
            }
            ' ' => {
                self.consume(); // skip whitespace (extra whitespace is removed/replaced by the preprocessing step)
            }
            _ => {
                // if brace level is 0, we just want to consume a selector
                if self.brace_level == 0 {
                    let name = self.consume_name();
                    if name.is_empty() {
                        self.consume(); // always consume something
                        return;
                    }
                    log::debug!("raw selector: '{name}'");
                    self.selector = Some(name);
                    return;
                }

                // if we're inside braces, we might need to parse attributes, so regular selector parsing wont work
                // consume everything until the next ';' or ':' (so stuff like `rgb(255, 255, 255)` is parsed correctly)
                let name = self.consume_while(|c| c != ';' && c != ':');
                if name.is_empty() {
                    self.consume(); // always consume something
                    return;
                }
                log::debug!("raw attr name/value: '{name}'");

                if self.brace_level == 1 && self.attr_name.is_none() {
                    self.attr_name = Some(name); // attr name
                } else if self.brace_level == 1 {
                    self.parse_attr_value(&name); // attr value
                    self.attr_name = None; // parsed attr, get ready for parsing the next one
                }
            }
        }
    }

    pub fn parse(&mut self) -> Declaration {
        while !self.eof() {
            self.advance();
        }
        Declaration::default()
    }
}
