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

#[derive(Debug, Clone, Display, Default, EnumString)]
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
    Custom(String),
}

#[derive(Debug, Clone, Copy, Display, Default, EnumString)]
pub enum Display {
    #[strum(serialize = "block")]
    #[default]
    Block,
    #[strum(serialize = "inline")]
    Inline,
    #[strum(serialize = "inline-block")]
    InlineBlock,
    #[strum(serialize = "flex")]
    Flex,
    #[strum(serialize = "inline-flex")]
    InlineFlex,
    #[strum(serialize = "grid")]
    Grid,
    #[strum(serialize = "inline-grid")]
    InlineGrid,
    #[strum(serialize = "flow-root")]
    FlowRoot,
    #[strum(serialize = "none")]
    None,
    #[strum(serialize = "contents")]
    Contents,
}

/// CSS rule declaration for one or multiple selectors.
#[derive(Debug, Clone, Default)]
pub struct Declaration {
    pub display: Display,
    pub position: Position,
    pub color: Option<Srgb>,
    pub background_color: Option<Srgb>,
    pub font_family: Option<FontFamily>,
    pub margin: [Option<Dimension>; 4],
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
    #[inline]
    pub fn from_inline(inline: &str) -> Self {
        CssParser::parse_inline(inline)
    }
}

#[derive(Debug, Clone, Default)]
pub struct GlobalStyle {
    /// Selector, declarations
    pub rules: Vec<(String, Declaration)>,
}

impl GlobalStyle {
    pub fn add_rule(&mut self, selector: &str, decl: Declaration) {
        log::debug!("adding rule '{decl:?} to GlobalStyle (selector: {selector})'");
        self.rules.push((selector.to_string(), decl));
    }

    pub fn from_css(css: &str, mode: ParserMode) -> Self {
        CssParser::new(css, mode).parse()
    }

    pub fn default_css() -> Self {
        Self::from_css(include_str!("internal/default.css"), ParserMode::DefaultCss)
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
            result.push(if c.is_whitespace() { ' ' } else { c });
        }
    }
    result
}

/// Represents the parsing behaviour of the CSS parser.
#[derive(Debug, Clone, Copy)]
pub enum ParserMode {
    /// Parse regular CSS files.
    Normal,
    /// Parse the browsers `default.css` file.
    DefaultCss,
}

/// Dragonfly's CSS parser. Does not use any external libraries except for [`css_color`] for color parsing.
#[derive(Debug, Clone)]
pub struct CssParser {
    input: String,
    pos: usize,
    brace_level: usize,
    decl_brace_level: Option<usize>,
    selector: Option<String>,
    attr_name: Option<String>,
    decl: Declaration,
    mode: ParserMode,
    style: GlobalStyle,
}

impl CssParser {
    pub fn new(css: &str, mode: ParserMode) -> Self {
        let input = remove_comments_and_extra_whitespace(css);
        log::debug!("processed input string: '{input}'");
        Self {
            input,
            pos: 0,
            brace_level: 0,
            decl_brace_level: None,
            selector: None,
            attr_name: None,
            decl: Declaration::default(),
            mode,
            style: GlobalStyle::default(),
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
            "display" => self.decl.display = Display::from_str(value).unwrap_or(Display::default()),
            "position" => {
                self.decl.position = Position::from_str(&value).unwrap_or(Position::default())
            }
            "color" => self.decl.color = Srgb::from_str(&value).ok(),
            "background-color" => self.decl.background_color = Srgb::from_str(&value).ok(),
            "font-family" => {
                self.decl.font_family = Some(
                    FontFamily::from_str(value).unwrap_or(FontFamily::Custom(value.to_string())),
                )
            }
            "margin" => {
                // top, right, bottom, left
                for (i, s) in value.split_whitespace().enumerate() {
                    self.decl.margin[i] = Some(Dimension::from_str(s));
                }
            }
            "margin-top" => self.decl.margin[0] = Some(Dimension::from_str(value)),
            "margin-right" => self.decl.margin[1] = Some(Dimension::from_str(value)),
            "margin-bottom" => self.decl.margin[2] = Some(Dimension::from_str(value)),
            "margin-left" => self.decl.margin[3] = Some(Dimension::from_str(value)),
            _ => {
                log::warn!("unhandled attr '{attr_name}'")
            }
        }

        log::debug!("declparse step:\n{:?}", self.decl);
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

                // check if current selector rule list has been closed
                if let Some(decl_brace_level) = self.decl_brace_level {
                    if decl_brace_level == self.brace_level {
                        self.style
                            .add_rule(&self.selector.clone().unwrap(), self.decl.clone());
                        self.decl_brace_level = None;
                        self.selector = None;
                    }
                }
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
                    self.decl_brace_level = Some(self.brace_level);
                    return;
                }

                // if we're inside braces, we might need to parse attributes, so regular selector parsing wont work
                // consume everything until the next ';' or ':' (so stuff like `rgb(255, 255, 255)` is parsed correctly)
                let name = self.consume_while(|c| c != ';' && c != ':');
                if name.is_empty() {
                    self.consume(); // always consume something
                    return;
                }

                if self.brace_level == 1 && self.attr_name.is_none() {
                    log::debug!("raw attr name: '{name}'");
                    self.attr_name = Some(name); // attr name
                } else if self.brace_level == 1 {
                    log::debug!("raw attr value: '{name}'");
                    self.parse_attr_value(&name); // attr value
                    self.attr_name = None; // parsed attr, get ready for parsing the next one
                }
            }
        }
    }

    pub fn parse(&mut self) -> GlobalStyle {
        while !self.eof() {
            self.advance();
        }
        log::debug!("eof, done parsing. final style:\n{:?}", self.style);
        self.style.clone()
    }

    pub fn parse_inline(inline: &str) -> Declaration {
        let mut parser = CssParser::new("", ParserMode::Normal);
        for attr in inline.split(';') {
            let mut parts = attr.split(':');

            let key = parts.next().unwrap_or("").trim();
            let value = parts.next().unwrap_or("").trim();

            // don't attempt to parse failed values
            if key.is_empty() && value.is_empty() {
                continue;
            }

            parser.attr_name = Some(key.to_string());
            parser.parse_attr_value(value);
        }
        parser.decl
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Unit {
    /// Units that are not relative to anything else, and are generally considered to always be the same size.
    /// Value is in pixels.
    Absolute(f32),
    /// Relative to the font size of the parent, in the case of typographical properties like `font-size`,
    /// and font size of the element itself, in the case of other properties like `width`.
    RelativeToParentFontSize(f32),
    /// Relative to the height of the element's font.
    RelativeToParentFontHeight(f32),
    /// Relative to the advance measure (width) of the glyph "0" of the element's font.
    RelativeToGlyph0Width(f32),
    /// Relative to the font size of the root element.
    RelativeToRootFontSize(f32),
    /// Relative to the line height of the element.
    RelativeToLineHeight(f32),
}

impl Default for Unit {
    fn default() -> Self {
        Self::Absolute(0.0)
    }
}

impl Unit {
    /// Parses a unit from a string.
    pub fn from_str(s: &str, num: f32) -> Self {
        // only leave lowercase alphabetic characters and whitespace
        // without unnecessary whitespace on the left and right
        let mut s = s.trim().to_lowercase();
        s.retain(|c| c.is_alphabetic() || c.is_whitespace());

        match s.as_str() {
            "px" => Self::Absolute(num),
            "in" => Self::Absolute(num * 96.0),
            "cm" => Self::Absolute(num * 96.0 / 2.54),
            "mm" => Self::Absolute((num * 96.0 / 2.54) / 10.0),
            "em" => Self::RelativeToParentFontSize(num),
            "ex" => Self::RelativeToParentFontHeight(num),
            _ => {
                // TODO: what should we do here?
                log::warn!("unhandled unit '{s}'");
                Self::Absolute(num)
            }
        }
    }
}

/// Represents and parses CSS dimensions (number + unit) (e.g. `4px`, `.7em`, `1.2rem`).
#[derive(Debug, Clone, Copy, Default)]
pub struct Dimension {
    /// The number part of the dimension.
    pub number: f32,
    /// Dimension unit
    pub unit: Unit,
}

impl Dimension {
    pub fn from_str(s: &str) -> Self {
        log::debug!("parsing dimension '{s}'");
        let (number, number_len) = Self::parse_number(s);
        let unit = Unit::from_str(&s[number_len..], number);
        log::debug!("parsed dimension: {number}, unit: {unit:?}");
        Self { number, unit }
    }

    fn parse_number(s: &str) -> (f32, usize) {
        let mut number_str = String::new();
        for c in s.chars() {
            if c.is_numeric() || c == '.' {
                number_str.push(c)
            }
        }
        let parsed = number_str.parse::<f32>();
        if let Ok(num) = parsed {
            log::debug!("dimension number str: {number_str}");
            (num, number_str.len())
        } else {
            log::debug!(
                "failed to parse dimension number: {}",
                parsed.err().unwrap()
            );
            (0.0, 0)
        }
    }
}
