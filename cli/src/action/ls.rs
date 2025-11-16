use quartz_core::{
    Quartz, QuartzResult, endpoint::EndpointHandle, state::StateField, tree::Node,
};

#[derive(clap::Args, Debug)]
pub struct Args {
    /// See the children of a specific handle
    pub handle: Option<String>,

    /// Set a limit for how deep the listing goes in sub-handles
    #[arg(long, value_name = "N")]
    pub depth: Option<usize>,
}

pub fn cmd(args: Args, quartz: Quartz) -> QuartzResult {
    let tree_root = quartz.handle_tree(args.handle);
    let current_handle = StateField::Endpoint.get(&quartz).ok();
    output_tree(tree_root.root, current_handle, 0);

    Ok(())
}

fn output_tree(
    tree: Node<EndpointHandle>,
    current_handle: Option<String>,
    padding: usize,
) {
    let is_root = tree.value.handle().is_empty();
    let handle_str = tree.value.handle();
    let is_in_use = current_handle
        .clone()
        .map(|inner| inner.eq(&handle_str))
        .unwrap_or(false);
    // Dont apply padding to direct children of the root
    let aditional_padding = if is_root { 0 } else { 2 };

    let handle_str = tree.value.handle();
    let padding_str = " ".repeat(padding);

    let handle_marker = if is_in_use { "*" } else { "" };

    println!("{}{}{}", padding_str, handle_marker, handle_str);
    for child in tree.children {
        output_tree(
            child,
            current_handle.clone(),
            padding + aditional_padding,
        );
    }
}
