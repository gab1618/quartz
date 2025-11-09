use crate::{Quartz, QuartzResult, editor::Editor};

pub trait Pager {
    fn paginate<E: Editor, P: Pager>(&self, quartz: &Quartz<E, P>, content: &[u8]) -> QuartzResult;
}

#[derive(Default)]
pub struct NoPager {}
impl Pager for NoPager {
    fn paginate<E: Editor, P: Pager>(
        &self,
        _quartz: &Quartz<E, P>,
        _content: &[u8],
    ) -> QuartzResult {
        Ok(())
    }
}
