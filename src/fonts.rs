use crate::{DfError, DfResult};
use font_kit::{
    family_name::FamilyName, handle::Handle, properties::Properties, source::SystemSource,
};
use fontdue::Font;
use std::io::Read;

pub const CRUFT_TTF_DATA: &[u8] = include_bytes!("./internal/cruft.ttf");

#[derive(Debug, Clone)]
pub struct FontStorage {
    pub serif: Option<Font>,
    pub sans_serif: Option<Font>,
    pub monospace: Option<Font>,
    pub cursive: Option<Font>,
    pub cache_fonts: bool,
    /// internal/cruft.ttf
    pub fallback_font: Font,
    cached_font: Option<(String, Font)>,
}

impl Default for FontStorage {
    fn default() -> Self {
        Self {
            serif: None,
            sans_serif: None,
            monospace: None,
            cursive: None,
            cache_fonts: true,
            cached_font: None,
            fallback_font: Font::from_bytes(CRUFT_TTF_DATA, fontdue::FontSettings::default())
                .unwrap(),
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

impl FontStorage {
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
        self.serif = get_font_data(FamilyName::Serif, &properties).ok();
        self.sans_serif = get_font_data(FamilyName::SansSerif, &properties).ok();
        self.monospace = get_font_data(FamilyName::Monospace, &properties).ok();
        self.cursive = get_font_data(FamilyName::Cursive, &properties).ok();
        log::info!("loaded fonts in {:?}", start.elapsed());
    }

    /// If the requested font is not available, will use the other fonts in the following priority:
    ///
    /// serif, sans serif, monospace, cursive
    pub fn serif(&self) -> &Font {
        if let Some(serif) = &self.serif {
            serif
        } else if let Some(sans_serif) = &self.sans_serif {
            sans_serif
        } else if let Some(monospace) = &self.monospace {
            monospace
        } else {
            &self.fallback_font
        }
    }

    /// If the requested font is not available, will use the other fonts in the following priority:
    ///
    /// sans serif, serif, monospace, cursive
    pub fn sans_serif(&self) -> &Font {
        if let Some(sans_serif) = &self.sans_serif {
            sans_serif
        } else if let Some(serif) = &self.serif {
            serif
        } else if let Some(monospace) = &self.monospace {
            monospace
        } else {
            &self.fallback_font
        }
    }

    /// If the requested font is not available, will use the other fonts in the following priority:
    ///
    /// monospace, sans serif, serif, cursive
    pub fn monospace(&self) -> &Font {
        if let Some(monospace) = &self.monospace {
            monospace
        } else if let Some(sans_serif) = &self.sans_serif {
            sans_serif
        } else if let Some(serif) = &self.serif {
            serif
        } else {
            &self.fallback_font
        }
    }

    /// If the requested font is not available, will use the other fonts in the following priority:
    ///
    /// cursive, sans serif, serif, monospace
    pub fn cursive(&self) -> &Font {
        if let Some(cursive) = &self.cursive {
            cursive
        } else if let Some(sans_serif) = &self.sans_serif {
            sans_serif
        } else if let Some(serif) = &self.serif {
            serif
        } else {
            &self.fallback_font
        }
    }

    /// Get font by name. If the font is already present in the font cache, no font lookup is made.
    pub async fn by_name(&mut self, name: &str) -> Option<Font> {
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
}
