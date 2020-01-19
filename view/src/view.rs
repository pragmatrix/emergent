use emergent_presenter::Context;
use emergent_ui::WindowMsg;

pub trait View<State, Msg> {
    fn update(&mut self, state: &State);
    fn present(&mut self, presenter: &mut Context<Msg>);
    fn dispatch(&mut self, _wm: WindowMsg) -> Option<Msg> {
        None
    }
}
