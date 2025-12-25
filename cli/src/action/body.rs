use crate::error::{Error, Result};
use crate::{cli::BodyCmd as Cmd, ctx::Ctx};

#[derive(clap::Args, Debug)]
pub struct Args {
    /// Which extension to read body as. E.g.: quartz body --format json edit
    #[arg(long, value_name = "EXT")]
    format: Option<String>,

    #[command(subcommand)]
    command: crate::cli::BodyCmd,
}

pub fn cmd(ctx: Ctx, args: Args) -> Result {
    match args.command {
        Cmd::Show => {
            print(ctx)?;
        }
        Cmd::Stdin => stdin(ctx),
        Cmd::Edit => edit(ctx, args.format)?,
    };

    Ok(())
}

pub fn print(ctx: Ctx) -> Result {
    let endpoint = ctx.quartz.endpoint();
    let curr_handle = endpoint.handle().ok_or(Error::NoHandleInUse)?;

    if let Some(body) = curr_handle.body() {
        print!("{body}");
    }

    Ok(())
}

pub fn edit(ctx: Ctx, format: Option<String>) -> Result {
    ctx.body_edit(format)?;

    Ok(())
}

pub fn stdin(ctx: Ctx) {
    let mut input = String::new();
    while let Ok(bytes) = std::io::stdin().read_line(&mut input) {
        if bytes == 0 {
            break;
        }
    }

    let endpoint = ctx.quartz.endpoint();
    if let Some(curr_handle) = endpoint.handle() {
        curr_handle.set_body(input).unwrap();
    }
}
