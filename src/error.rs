use thiserror::Error;

#[derive(Debug, Clone, Error)]
pub enum WorgenXError {
    #[error("Error: {0}")]
    WordlistSettingsError(String),
}

pub enum WordlistSettingsError {
    // FIXME: Add error messages
}
