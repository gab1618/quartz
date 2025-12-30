use crate::cli::EndpointPatchArg;


#[derive(clap::Args, Debug)]
pub struct CreateArgs {
    pub handle: String,

    #[command(flatten)]
    pub patch: EndpointPatchArg,

    /// Immediatly switches to this handle after creating it
    #[arg(name = "use", long)]
    pub switch: bool,
}

#[derive(clap::Args, Debug)]
pub struct SwitchArgs {
    pub handle: Option<String>,
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
    pub src: String,
    pub dest: String,
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
