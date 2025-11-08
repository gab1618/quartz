use std::{io::Write, process::Stdio};

use quartz_core::{Quartz, QuartzError, editor::Editor, pager::Pager};

#[derive(Default)]
pub struct CliPager {}

impl Pager for CliPager {
    fn paginate<E: Editor, P: Pager>(
        &self,
        quartz: &Quartz<E, P>,
        content: &[u8],
    ) -> quartz_core::QuartzResult {
        let pager = quartz.ctx.config.parse().preferences.pager();

        let mut child = std::process::Command::new(&pager)
            .stdin(Stdio::piped())
            .spawn()
            .unwrap_or_else(|err| {
                panic!("failed to open pager: {}\n\n{}", pager, err);
            });

        child
            .stdin
            .as_mut()
            .unwrap()
            .write_all(content)
            .map_err(|_| QuartzError::Internal)?;
        child.wait().map_err(|_| QuartzError::Internal)?;

        Ok(())
    }
}
