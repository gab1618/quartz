use crate::{action::CliQuartz, cli::BodyCmd as Cmd};
use quartz_core::QuartzResult;

#[derive(clap::Args, Debug)]
pub struct Args {
    /// Which extension to read body as. E.g.: quartz body --format json edit
    #[arg(long, value_name = "EXT")]
    format: Option<String>,

    #[command(subcommand)]
    command: crate::cli::BodyCmd,
}

pub fn cmd(quartz: CliQuartz, args: Args) -> QuartzResult {
    match args.command {
        Cmd::Show => print(quartz),
        Cmd::Stdin => stdin(quartz),
        Cmd::Edit => edit(quartz, args.format)?,
    };

    Ok(())
}

pub fn print(quartz: CliQuartz) {
    let (_, mut endpoint) = quartz.ctx.require_endpoint();

    if let Some(body) = endpoint.body() {
        print!("{body}");
    }
}

pub fn edit(quartz: CliQuartz, format: Option<String>) -> QuartzResult {
    quartz.body_edit(format)?;

    Ok(())
}

pub fn stdin(quartz: CliQuartz) {
    let mut input = String::new();
    while let Ok(bytes) = std::io::stdin().read_line(&mut input) {
        if bytes == 0 {
            break;
        }
    }

    quartz.set_body(input);
}
