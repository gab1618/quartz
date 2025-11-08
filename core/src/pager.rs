use crate::{Quartz, QuartzResult, editor::Editor};

pub trait Pager {
    fn paginate<E: Editor, P: Pager>(&self, quartz: &Quartz<E, P>, content: &[u8]) -> QuartzResult;
}
