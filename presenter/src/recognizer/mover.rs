use crate::recognizer::{pan, PanRecognizer};
use crate::{transaction, GestureRecognizer};
use emergent_drawing::Vector;

pub enum MoverRecognizer {}

impl MoverRecognizer {
    #[allow(clippy::new_ret_no_self)]
    pub fn new<S, GP, Msg>(pos_from: GP) -> impl GestureRecognizer<Event = Msg>
    where
        GP: Fn(&mut S) -> &mut Vector,
        GP: Clone + 'static,
        S: Clone + 'static,
    {
        PanRecognizer::new().activate(move |e, state| match e {
            pan::Event::Pressed(_p) => {
                let initial_pos = *pos_from(state);
                let pos_from = pos_from.clone();

                transaction::begin(move |e, state| match e {
                    pan::Event::Moved(_p, d) => {
                        *pos_from(state) = initial_pos + d;
                        transaction::sustain()
                    }
                    pan::Event::Released(_p, d) => {
                        *pos_from(state) = initial_pos + d;
                        transaction::commit()
                    }
                    _ => transaction::rollback(),
                })
            }
            _ => transaction::neglect(),
        })
    }
}
