use emergent_presenter::Presenter;
use emergent_ui::WindowMsg;

pub trait View<State, Msg> {
    fn update(&mut self, state: &State);
    fn present(&mut self, presenter: &mut Presenter<Msg>);
    fn dispatch(&mut self, wm: WindowMsg) -> Option<Msg> {
        None
    }
}
