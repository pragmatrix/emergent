use crate::InputProcessor;
use emergent_drawing::Vector;
use emergent_ui::WindowMessage;

pub struct BounceBack<FR> {
    get_resistance: FR,
}

pub enum Source {
    Begin(Vector),
    Moved(Vector),
    End(Vector),
}

pub trait WithBounceBack {
    fn with_bounce_back<SourceEvent, State, FR>(self, get_resistance: FR) -> BounceBack<FR>
    where
        FR: Fn(&State) -> Vector,
        Self: Sized,
        Self: InputProcessor<In = WindowMessage, Out = SourceEvent>,
        SourceEvent: Into<Source>,
    {
        BounceBack { get_resistance }
    }
}
