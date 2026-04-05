#[derive(Debug, thiserror::Error)]
pub enum EndpointError {
    #[error("No handle in use")]
    NoHandleInUse,
    #[error("Could not access handle body")]
    AccessHandleBody(#[source] std::io::Error),
    #[error("Handle is empty")]
    EmptyHandle,
    #[error("Handle name is empty")]
    EmptyHandleName,
    #[error("Handle already existing")]
    AlreadyExistingHandle,
    #[error("Not found handle: {0}")]
    HandleNotFound(String),
    #[error("Could not save handle: {0}")]
    SaveHandle(#[source] std::io::Error),
    #[error("Could not get handle children: {0}")]
    GetHandleChildren(#[source] std::io::Error),
    #[error("Could not parse handle spec")]
    ParseHandleSpec,
    #[error("Could not read spec file: {0}")]
    ReadHandleSpec(#[source] std::io::Error),
    #[error("Could not serialize endpoint: {0}")]
    SerializeEndpoint(#[source] toml::ser::Error),
    #[error("Could not save endpoint: {0}")]
    SaveEndpoint(#[source] std::io::Error),
    #[error("Could not get handle in entrybuilder")]
    GetEntryBuilderHandle,
    #[error("Must use recursive mode to remove handles that have children")]
    RemoveChildrenOnNonRecursiveMode,
    #[error("Could not remove handle files: {0}")]
    RemoveHandleFiles(#[source] std::io::Error),
    #[error("No handle parent directory")]
    NoHandleParentDir,
    #[error("Could not modify body: {0}")]
    ModifyBody(#[source] std::io::Error),
    #[error("Could not set request body")]
    SetRequestBody,
    #[error("Could not serialize uri")]
    SerializeUri,
}
