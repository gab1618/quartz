use quartz_core::cookie::Cookie;

use crate::{ctx::Ctx, error::Result};

#[derive(clap::Args, Debug)]
pub struct PrintArgs {
    key: Option<String>,

    /// Filter cookies that match this domain
    #[arg(long, short = 'd')]
    domain: Option<String>,
}

pub fn print(ctx: Ctx, args: PrintArgs) -> Result {
    let curr_env = ctx.quartz.current_env()?;
    let jar = curr_env.cookie_jar();

    let iter = jar.iter().filter(|c| {
        args.domain
            .as_ref()
            .map(|domain| c.domain().matches(domain.as_str()))
            .unwrap_or(true)
    });

    if let Some(key) = args.key {
        let cookies = iter.filter(|c| c.name() == key).collect::<Vec<&Cookie>>();

        match cookies.len() {
            0 => panic!("{key}: No such cookie"),
            1 => println!("{}", cookies[0].value()),
            _ => {
                for c in cookies {
                    println!("{}: {}", **c.domain(), c.value());
                }
            }
        }
    } else {
        for cookie in iter {
            println!("{}={}", cookie.name(), cookie.value());
        }
    }
    Ok(())
}
