use emergent_ui::{ElementState, MouseButton, WindowEvent};
use std::collections::hash_map::Entry;
use std::collections::HashMap;

#[derive(Clone, PartialEq, Eq, Default, Debug)]
pub struct Subscriptions(HashMap<Subscription, usize>);

impl Subscriptions {
    pub fn subscribe(&mut self, subscription: Subscription) {
        use Entry::*;
        match self.0.entry(subscription) {
            Occupied(mut e) => *e.get_mut() += 1,
            Vacant(e) => {
                e.insert(1);
            }
        }
    }

    pub fn unsubscribe(&mut self, subscription: Subscription) {
        use Entry::*;
        match self.0.entry(subscription) {
            Occupied(mut e) => {
                let cnt = e.get_mut();
                if *cnt == 1 {
                    e.remove();
                } else {
                    *cnt += 1;
                }
            }
            Vacant(_) => error!("inconsistent subscription count for {:?}", subscription),
        }
    }

    pub fn subscribes(&self, subscription: Subscription) -> bool {
        self.0.contains_key(&subscription)
    }

    pub fn wants_event(&self, event: &WindowEvent) -> bool {
        self.0.keys().any(|s| s.wants_event(event))
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum Subscription {
    /// Subscribed to continuity events after a specific mouse button was pressed.
    ///
    /// These are automatically subscribed for every input processor that receives button presses.
    ButtonContinuity(MouseButton),
    /// Subscribed to timer ticks once per frame.
    Ticks,
}

impl Subscription {
    pub fn wants_event(self, event: &WindowEvent) -> bool {
        match self {
            Subscription::ButtonContinuity(b) => match event {
                WindowEvent::CursorMoved(_) => true,
                WindowEvent::MouseInput { state, button }
                    if *state == ElementState::Released && *button == b =>
                {
                    true
                }
                _ => false,
            },
            Subscription::Ticks => match event {
                WindowEvent::Tick(_) => true,
                _ => false,
            },
        }
    }
}

pub(crate) trait AutoSubscribe {
    /// the set of subscriptions to add or to remove in response to an event.
    fn auto_subscribe(&self, subscriptions: &mut Subscriptions);
}

impl AutoSubscribe for WindowEvent {
    fn auto_subscribe(&self, subscriptions: &mut Subscriptions) {
        match self {
            WindowEvent::MouseInput { state, button } if *state == ElementState::Pressed => {
                subscriptions.subscribe(Subscription::ButtonContinuity(*button));
            }
            WindowEvent::MouseInput { state, button } if *state == ElementState::Released => {
                subscriptions.unsubscribe(Subscription::ButtonContinuity(*button));
            }
            _ => {}
        }
    }
}
