use quartz_core::endpoint::EndpointPatch;

#[derive(clap::Args, Debug)]
pub struct CreateArgs {
    pub handle: String,

    #[command(flatten)]
    pub patch: EndpointPatch,

    /// Immediatly switches to this handle after creating it
    #[arg(name = "use", long)]
    pub switch: bool,
}

#[derive(clap::Args, Debug)]
pub struct SwitchArgs {
    pub handle: Option<String>,

    #[command(flatten)]
    pub patch: EndpointPatch,

    /// Make handle empty. Using it with other editing options will write a new endpoint in
    /// place of the old one
    #[arg(long)]
    pub empty: bool,
}

#[derive(clap::Args, Debug)]
pub struct CpArgs {
    #[arg(long, short = 'r')]
    pub recursive: bool,

    pub src: String,
    pub dest: String,
}

#[derive(clap::Args, Debug)]
pub struct MvArgs {
    pub handles: Vec<String>,
}

#[derive(clap::Args, Debug)]
pub struct RmArgs {
    /// Delete child handles recursively
    #[arg(long, short = 'r')]
    pub recursive: bool,

    /// Handles to be removed
    #[arg(name = "HANDLE", required = true)]
    pub handles: Vec<String>,
}
