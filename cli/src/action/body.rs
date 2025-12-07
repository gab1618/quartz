use crate::{cli::BodyCmd as Cmd, ctx::Ctx};
use quartz_core::error::{QuartzError, QuartzResult};

#[derive(clap::Args, Debug)]
pub struct Args {
    /// Which extension to read body as. E.g.: quartz body --format json edit
    #[arg(long, value_name = "EXT")]
    format: Option<String>,

    #[command(subcommand)]
    command: crate::cli::BodyCmd,
}

pub fn cmd(ctx: Ctx, args: Args) -> QuartzResult {
    match args.command {
        Cmd::Show => {
            print(ctx)?;
        }
        Cmd::Stdin => stdin(ctx),
        Cmd::Edit => edit(ctx, args.format)?,
    };

    Ok(())
}

pub fn print(ctx: Ctx) -> QuartzResult {
    let curr_handle = ctx.quartz.handle().ok_or(QuartzError::Internal)?;
    let mut curr_endpoint = curr_handle.endpoint()?;

    if let Some(body) = curr_endpoint.body() {
        print!("{body}");
    }

    Ok(())
}

pub fn edit(ctx: Ctx, format: Option<String>) -> QuartzResult {
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

    ctx.quartz.set_body(input).unwrap();
}
