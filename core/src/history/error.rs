
#[derive(Debug, thiserror::Error)]
pub enum HistoryError {
    #[error("Could not read history entries")]
    ReadEntries,
    #[error("Could not serialize history")]
    Serialize,
    #[error("Could not save history: {0}")]
    Save(#[source] std::io::Error),
    #[error("Could not get handle in entrybuilder")]
    GetEntryBuilderHandle,
    #[error("Empty history")]
    Empty,
    #[error("Could not read history entry")]
    ReadEntry,
    #[error("Could not parse history entry")]
    ParseEntry,
}

pub type HistoryResult<T = ()> = Result<T, HistoryError>;
