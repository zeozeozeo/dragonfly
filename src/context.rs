use html5ever::tree_builder::QuirksMode;
use scraper::Html;
use std::time::{Duration, Instant};
use url::Url;

use crate::*;

/// Page loading timers
#[derive(Debug, Copy, Clone, Default)]
pub struct Timers {
    /// Time it took to pull the page (download/get from cache)
    pub pull: Duration,
    /// Time it took to parse the page
    pub parse: Duration,
    /// Time it took to compute the last layout
    pub layout: Duration,
    /// Total time elapsed
    pub total: Duration,
}

#[derive(Debug, Clone)]
pub struct WebContext {
    /// Page URL
    url: Url,
    /// Page loading timers
    pub timers: Timers,
    /// Parsed page
    pub document: Option<Html>,
    /// Computed page layout tree. This can be used for rendering
    pub layout: Layout,
    /// Retrieves files and manages the file cache
    pub puller: Puller,
    /// Handles font storage and lookup
    pub font_store: FontStorage,
}

impl WebContext {
    pub async fn new(url: &str, font_store: FontStorage) -> DfResult<Self> {
        Ok(Self {
            url: Url::parse(url)?,
            timers: Timers::default(),
            document: None,
            layout: Layout::default(),
            puller: Puller::default(),
            font_store,
        })
    }

    pub async fn load(&mut self) -> DfResult<()> {
        // pull page, measure time
        let start = Instant::now();

        let data = self.puller.pull_str(self.url.clone()).await?;

        self.timers.pull = start.elapsed();
        log::info!("pulled in {:?}", self.timers.pull);

        // parse page, measure time
        log::info!("parsing page at '{}'", self.url);
        let parse_start = Instant::now();

        self.document = Some(Html::parse_document(&data));

        self.timers.parse = parse_start.elapsed();
        log::info!("parsed in {:?}", self.timers.parse);

        // log quirks mode
        match self.document().quirks_mode {
            QuirksMode::Quirks => log::warn!("using quirks mode"),
            QuirksMode::LimitedQuirks => log::warn!("using limited quirks mode"),
            QuirksMode::NoQuirks => log::info!("using standard mode"),
        }
        // log parser errors
        for err in &self.document().errors {
            log::warn!("HTML parser error: {:?}", err);
        }

        // compute page layout
        log::info!("computing layout for the first time");
        self.recompute_layout().await;

        // measure page load time
        self.timers.total = start.elapsed();
        log::info!("loaded page in {:?}", self.timers.total);

        Ok(())
    }

    pub async fn recompute_layout(&mut self) {
        log::info!("recomputing layout...");
        let start = Instant::now();

        let mut doc = self.document().clone();
        self.layout = Layout::compute(&mut doc, &self.font_store);

        self.timers.layout = start.elapsed();
        log::info!("computed layout in {:?}", self.timers.layout);
    }

    #[inline]
    pub fn document(&mut self) -> &mut Html {
        self.document.as_mut().unwrap()
    }
}
