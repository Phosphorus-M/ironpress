/// Errors that can occur during HTML-to-PDF conversion.
#[derive(Debug, thiserror::Error)]
pub enum IronpressError {
    #[error("HTML parsing error: {0}")]
    ParseError(String),

    #[error("CSS parsing error: {0}")]
    CssError(String),

    #[error("Layout error: {0}")]
    LayoutError(String),

    #[error("PDF rendering error: {0}")]
    RenderError(String),

    #[error("Font error: {0}")]
    FontError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Security error: input rejected: {0}")]
    SecurityError(String),
}
