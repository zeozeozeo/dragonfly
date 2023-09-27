#[derive(Debug, thiserror::Error)]
pub enum DfError {
    #[error("reqwest error: {0}")]
    ReqwestError(#[from] reqwest::Error),
    #[error("io error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("url parser error: {0}")]
    UrlParseError(#[from] url::ParseError),
    #[error("fontkit selection error: {0}")]
    FontSelectionError(#[from] font_kit::error::SelectionError),
    #[error("no filesystem present")]
    NoFilesystemError,
    #[error("failed to load font: {0}")]
    FontLoadingError(String),
    #[error("unknown css property: {0}")]
    UnknownStyleProperty(String),
}

pub type DfResult<T> = Result<T, DfError>;
