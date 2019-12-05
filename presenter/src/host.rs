//! State is persisting for the time the presentation is active.

use crate::{Presenter, Support};
use emergent_drawing::ReplaceWith;
use emergent_presentation::Presentation;
use emergent_ui::FrameLayout;

pub struct Host {
    pub support: Support,
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

    pub fn present(
        &mut self,
        boundary: FrameLayout,
        f: impl FnOnce(&mut Presenter),
    ) -> &Presentation {
        self.replace_with(|h| {
            let mut presenter = Presenter::new(h, boundary);
            f(&mut presenter);
            let (mut host, presentation) = presenter.into_host_and_presentation();
            host.recent_presentation = presentation;
            host
        });
        &self.recent_presentation
    }
}
