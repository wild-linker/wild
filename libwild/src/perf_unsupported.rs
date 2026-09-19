use crate::args::CounterKind;

pub(crate) struct CounterList {}

impl CounterList {
    pub(crate) fn from_kinds(_opts: &[CounterKind]) -> Self {
        CounterList {}
    }

    #[allow(clippy::unused_self, clippy::needless_pass_by_ref_mut)]
    pub(crate) fn read(&mut self) -> Vec<Option<crate::timing::CounterSnapshot>> {
        Vec::new()
    }
}
