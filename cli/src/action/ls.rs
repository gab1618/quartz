use colored::{Color, Colorize};
use quartz_core::{endpoint::handle::EndpointHandle, error::Result, state::field::StateField};

use crate::{color::method_color, ctx::Ctx};

#[derive(clap::Args, Debug)]
pub struct Args {
    /// See the children of a specific handle
    pub handle: Option<String>,

    /// Set a limit for how deep the listing goes in sub-handles
    #[arg(long, value_name = "N")]
    pub depth: Option<usize>,
}

pub fn cmd(ctx: Ctx, args: Args) -> Result {
    let parsed_arg_handle = args
        .handle
        .map(|handle| EndpointHandle::new(&ctx.quartz, handle.into()));
    let base_handle = parsed_arg_handle.unwrap_or(EndpointHandle::root(&ctx.quartz));
    let current_handle = ctx.quartz.state_get(StateField::Endpoint).ok();
    output_tree(base_handle, current_handle, 0)?;

    Ok(())
}

fn output_tree(base: EndpointHandle, current_handle: Option<String>, padding: usize) -> Result {
    let handle_str = base.handle();
    let is_root = handle_str.is_empty();
    // Dont apply padding to direct children of the root
    let aditional_padding = if is_root { 0 } else { 2 };

    if !is_root {
        let is_in_use = current_handle
            .as_ref()
            .map(|inner| inner.eq(&handle_str))
            .unwrap_or(false);
        let handle_str = base.handle();
        let padding_str = " ".repeat(padding);
        let handle_color = if is_in_use { Color::Blue } else { Color::White };

        let endpoint = base.endpoint().ok();
        let method = endpoint.map(|endpoint| endpoint.method);
        let method_str = method
            .map(|method| {
                let color = method_color(&method);
                format!("{} ", method.color(color))
            })
            .unwrap_or_default();

        println!(
            "{}{}{}",
            padding_str,
            method_str,
            handle_str.color(handle_color)
        );
    }
    for child in base.children()? {
        output_tree(child, current_handle.clone(), padding + aditional_padding)?;
    }
    Ok(())
}
