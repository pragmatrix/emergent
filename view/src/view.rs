use emergent_presenter::Context;
use emergent_ui::WindowEvent;

pub trait View<State, Msg> {
    fn update(&mut self, state: &State);
    fn present(&mut self, presenter: &mut Context);
    fn dispatch(&mut self, _wm: WindowEvent) -> Option<Msg> {
        None
    }
}
