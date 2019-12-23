pub mod presenter {
    use emergent_presenter::Presenter;
    use std::collections::HashMap;

    pub fn from_test_environment<Msg>() -> Presenter<Msg> {
        Presenter::new(
            super::support::from_test_environment().into(),
            super::frame_layout::from_test_environment(),
            HashMap::new(),
        )
    }
}

mod host {
    use emergent_presenter::Host;

    pub fn from_test_environment<Msg: 'static>() -> Host<Msg> {
        Host::new(super::support::from_test_environment())
    }
}

mod support {
    use crate::skia::path_support::PathSupport;
    use crate::skia::text::PrimitiveText;
    use emergent_drawing::FromTestEnvironment;
    use emergent_presenter::Support;
    use emergent_ui::DPI;

    pub fn from_test_environment() -> Support {
        Support::new(
            DPI::from_test_environment(),
            PrimitiveText::from_test_environment(),
            PathSupport,
        )
    }
}

mod frame_layout {
    use emergent_drawing::FromTestEnvironment;
    use emergent_ui::{FrameLayout, DPI};

    pub fn from_test_environment() -> FrameLayout {
        FrameLayout {
            dpi: DPI::from_test_environment(),
            // TODO: is it useful to define dimensions in the test environment?
            dimensions: (std::u32::MAX, std::u32::MAX),
        }
    }
}
