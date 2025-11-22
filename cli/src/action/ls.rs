use quartz_core::{Quartz, endpoint::EndpointHandle, error::QuartzResult, state::StateField};

use crate::ctx::Ctx;

#[derive(clap::Args, Debug)]
pub struct Args {
    /// See the children of a specific handle
    pub handle: Option<String>,

    /// Set a limit for how deep the listing goes in sub-handles
    #[arg(long, value_name = "N")]
    pub depth: Option<usize>,
}

pub fn cmd(ctx: Ctx, args: Args) -> QuartzResult {
    let parsed_arg_handle = args.handle.map(|handle| EndpointHandle::from(handle));
    let base_handle = parsed_arg_handle.unwrap_or(Quartz::root_handle());
    let current_handle = StateField::Endpoint.get(&ctx.quartz).ok();
    output_tree(&ctx, base_handle, current_handle, 0);

    Ok(())
}

fn output_tree(ctx: &Ctx, base: EndpointHandle, current_handle: Option<String>, padding: usize) {
    let is_root = base.handle().is_empty();
    let handle_str = base.handle();
    let is_in_use = current_handle
        .clone()
        .map(|inner| inner.eq(&handle_str))
        .unwrap_or(false);
    // Dont apply padding to direct children of the root
    let aditional_padding = if is_root { 0 } else { 2 };

    let handle_str = base.handle();
    let padding_str = " ".repeat(padding);

    let handle_marker = if is_in_use { "*" } else { "" };

    println!("{}{}{}", padding_str, handle_marker, handle_str);
    for child in base.children(&ctx.quartz).unwrap() {
        output_tree(
            ctx,
            child,
            current_handle.clone(),
            padding + aditional_padding,
        );
    }
}
