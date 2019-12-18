//! State is persisting for the time the presentation is active.

use crate::{Presenter, Support, UntypedGestureRecognizer};
use emergent_drawing::ReplaceWith;
use emergent_presentation::{Presentation, Scope};
use emergent_ui::FrameLayout;
use std::collections::HashMap;

pub struct Host {
    pub support: Support,
    /// A copy of the most recent presentation.
    /// This is primarily used for hit testing.
    presentation: Presentation,

    /// The active recognizers of the previous presentation.
    pub(crate) recognizers: HashMap<Vec<Scope>, Box<dyn UntypedGestureRecognizer>>,
}

impl Host {
    pub fn new(support: Support) -> Host {
        Host {
            support,
            presentation: Presentation::Empty,
            recognizers: HashMap::new(),
        }
    }

    pub fn present(
        &mut self,
        boundary: FrameLayout,
        present: impl FnOnce(&mut Presenter),
    ) -> &Presentation {
        self.replace_with(|h| {
            let mut presenter = Presenter::new(h, boundary);
            present(&mut presenter);
            // commit
            let mut host = presenter.host;
            host.presentation = presenter.presentation;
            host.recognizers = presenter.recognizers;
            host
        });
        &self.presentation
    }
}
