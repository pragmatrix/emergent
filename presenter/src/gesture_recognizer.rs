use emergent_ui::WindowMessage;

/// A trait to define gesture recognizers.
///
/// Gesture recognizers are persisting and are updated with
/// each WindowMessage.
///
/// Their lifetime is bound to the context scope they are rendered at first.

pub trait GestureRecognizer {
    type Event;
    fn update(&mut self, event: WindowMessage) -> Option<Self::Event>;

    fn map<F, To>(self, f: F) -> MappingGestureRecognizer<Self, F>
    where
        F: Fn(Self::Event) -> To,
        Self: Sized,
    {
        MappingGestureRecognizer::new(self, f)
    }
}

pub struct MappingGestureRecognizer<R, F> {
    recognizer: R,
    map_event: F,
}

impl<R, F> MappingGestureRecognizer<R, F> {
    pub fn new(recognizer: R, map_event: F) -> Self {
        Self {
            recognizer,
            map_event,
        }
    }
}

impl<From, To, R, F> GestureRecognizer for MappingGestureRecognizer<R, F>
where
    R: GestureRecognizer<Event = From>,
    F: Fn(From) -> To,
{
    type Event = To;

    fn update(&mut self, msg: WindowMessage) -> Option<Self::Event> {
        self.recognizer.update(msg).map(&self.map_event)
    }
}

/*
pub trait GestureRecognizerMap {
    type Event;

    fn map<F, To>(self, f: F) -> MappingGestureRecognizer<Self, F>
    where
        F: Fn(Self::Event) -> To,
        Self: Sized;
}

impl<Event, T> GestureRecognizerMap for T
where
    T: GestureRecognizer<Event = Event>,
{
    type Event = Event;

    fn map<F, To>(self, f: F) -> MappingGestureRecognizer<Self, F>
    where
        F: Fn(Self::Event) -> To,
    {
        MappingGestureRecognizer::new(self, f)
    }
}

*/
