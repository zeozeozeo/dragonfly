use crate::{DfError, DfResult, FontFamily};
use font_kit::{
    family_name::FamilyName, handle::Handle, properties::Properties, source::SystemSource,
};
use fontdue::{Font, Metrics};
use std::io::Read;

/// Default fallback font (Cruft) data.
pub const CRUFT_TTF_DATA: &[u8] = include_bytes!("./internal/cruft.ttf");

#[derive(Debug, Clone)]
pub struct FontManager {
    pub serif: Font,
    pub sans_serif: Font,
    pub monospace: Font,
    pub cursive: Font,
    pub cache_fonts: bool,
    pub fantasy: Font,
    /// internal/cruft.ttf
    pub fallback_font: Font,
    cached_font: Option<(String, Font)>,
}

impl Default for FontManager {
    fn default() -> Self {
        let fallback = Font::from_bytes(CRUFT_TTF_DATA, fontdue::FontSettings::default()).unwrap();
        Self {
            serif: fallback.clone(),
            sans_serif: fallback.clone(),
            monospace: fallback.clone(),
            cursive: fallback.clone(),
            fantasy: fallback.clone(),
            cache_fonts: true,
            cached_font: None,
            fallback_font: fallback,
        }
    }
}

fn get_font_data(family: FamilyName, properties: &Properties) -> DfResult<Font> {
    log::info!("looking for font family '{family:?}' with properties '{properties:?}'");
    let handle = SystemSource::new().select_best_match(&[family], properties)?;
    let data = match handle {
        Handle::Memory {
            ref bytes,
            font_index,
        } => {
            log::info!("copying font from memory, font idx {font_index}");
            log::info!("font size: {}", bytesize::ByteSize(bytes.len() as u64));
            bytes.to_vec()
        }
        #[cfg(not(target_arch = "wasm32"))]
        Handle::Path {
            ref path,
            font_index,
        } => {
            log::info!("reading font '{path:?}' from disk, font idx {font_index}");
            let mut f = std::fs::File::open(path)?;
            let mut buf: Vec<u8> = vec![];
            f.read_to_end(&mut buf)?;
            if let Ok(metadata) = f.metadata() {
                log::info!("font size: {}", bytesize::ByteSize(metadata.len()));
            }
            buf
        }
        #[cfg(target_arch = "wasm32")]
        Handle::Path { .. } => {
            log::error!("cannot read font from disk; no filesystem");
            return Err(DfError::NoFilesystemError);
        }
    };

    // load the font with fontdue
    log::info!("loading font...");
    let font = Font::from_bytes(data, fontdue::FontSettings::default());
    if font.is_ok() {
        log::info!("loaded font successfully");
        Ok(font.unwrap())
    } else {
        let err_str = font.err().unwrap().to_string();
        log::error!("failed to load font (fontdue): {err_str}");
        Err(DfError::FontLoadingError(err_str))
    }
}

impl FontManager {
    pub fn with_system_fonts() -> Self {
        let mut store = Self::default();
        store.load_system_fonts();
        store
    }

    #[inline]
    pub fn with_fallback_font() -> Self {
        Self::default()
    }

    pub fn load_system_fonts(&mut self) {
        // TODO: load fonts in parallel
        let start = std::time::Instant::now();
        log::info!("loading system fonts");
        let properties = Properties::new();
        self.serif = get_font_data(FamilyName::Serif, &properties).unwrap();
        self.sans_serif = get_font_data(FamilyName::SansSerif, &properties).unwrap();
        self.monospace = get_font_data(FamilyName::Monospace, &properties).unwrap();
        self.cursive = get_font_data(FamilyName::Cursive, &properties).unwrap();
        self.fantasy = get_font_data(FamilyName::Fantasy, &properties).unwrap();
        log::info!("loaded fonts in {:?}", start.elapsed());
    }

    /// Get font by name. If the font is already present in the font cache, no font lookup is made.
    pub fn by_name(&mut self, name: &str) -> Option<Font> {
        // check if we cached the font already
        // TODO: add an option to cache multiple fonts
        if let Some(cached_font) = &self.cached_font {
            if cached_font.0 == name {
                log::info!("found cached font '{name}'");
                return Some(cached_font.1.clone());
            }
        }

        // otherwise, load the font
        log::info!("looking up font '{name}'");
        let data =
            get_font_data(FamilyName::Title(name.to_string()), &Properties::default()).ok()?;
        self.cached_font = Some((name.to_string(), data.clone())); // update cached font data
        Some(data.clone())
    }

    pub fn get_font(&mut self, family: FontFamily) -> &Font {
        match family {
            FontFamily::Serif => &self.serif,
            FontFamily::SansSerif => &self.sans_serif,
            FontFamily::Monospace => &self.monospace,
            FontFamily::Cursive => &self.cursive,
            FontFamily::Fantasy => &self.fantasy,
            FontFamily::SystemUi => &self.serif,
            FontFamily::UiSerif => &self.serif,
            FontFamily::UiSansSerif => &self.sans_serif,
            FontFamily::UiMonospace => &self.monospace,
            FontFamily::UiRounded => &self.serif,
            FontFamily::Math => &self.serif,
            FontFamily::Emoji => &self.serif,
            FontFamily::Fangsong => &self.serif,
            FontFamily::Custom(s) => {
                if let None = self.by_name(&s) {
                    log::warn!("could not find system font '{s}'");
                    return &self.fallback_font;
                }
                &self.cached_font.as_ref().unwrap().1
            }
        }
    }

    pub fn glyph_metrics(&mut self, glyph: char, px: f32, family: FontFamily) -> Metrics {
        self.get_font(family).metrics(glyph, px)
    }
}
