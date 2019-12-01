//! State is persisting for the time the presentation is active.

use crate::{Presenter, Support};
use emergent_presentation::Presentation;
use emergent_ui::FrameLayout;

pub struct Host {
    support: Support,
    /// A copy of the most recent presentation.
    /// This is primarily used for hit testing.
    recent_presentation: Presentation,
}

impl Host {
    pub fn new(support: Support) -> Host {
        Host {
            support,
            recent_presentation: Presentation::Empty,
        }
    }

    pub fn present(&mut self, boundary: FrameLayout, f: impl FnOnce(&mut Presenter)) {
        let mut presenter = Presenter::new(self, boundary);
        f(&mut presenter);
        self.recent_presentation = presenter.into_presentation();
    }
}
