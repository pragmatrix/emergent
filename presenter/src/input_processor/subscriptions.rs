use emergent_ui::{ElementState, MouseButton, WindowEvent};
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::iter::FromIterator;

pub trait Subscriber {
    fn subscriptions(&self) -> Subscriptions;
}

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
            Vacant(_) => error!("unaccountable unsubscribe for {:?}", subscription),
        }
    }

    pub fn subscribes(&self, subscription: Subscription) -> bool {
        self.0.contains_key(&subscription)
    }

    pub fn wants_event(&self, event: &WindowEvent) -> bool {
        self.0.keys().any(|s| s.wants_event(event))
    }
}

impl FromIterator<Subscription> for Subscriptions {
    fn from_iter<T: IntoIterator<Item = Subscription>>(iter: T) -> Self {
        Subscriptions(HashMap::from_iter(iter.into_iter().map(|i| (i, 1))))
    }
}

impl<'a> FromIterator<&'a Subscription> for Subscriptions {
    fn from_iter<T: IntoIterator<Item = &'a Subscription>>(iter: T) -> Self {
        Subscriptions(HashMap::from_iter(iter.into_iter().map(|i| (i.clone(), 1))))
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
