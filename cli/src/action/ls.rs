#[derive(clap::Args, Debug)]
pub struct Args {
    /// See the children of a specific handle
    pub handle: Option<String>,

    /// Set a limit for how deep the listing goes in sub-handles
    #[arg(long, value_name = "N")]
    pub depth: Option<usize>,
}

