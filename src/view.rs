/*
use crate::app::App;
use cargo_metadata::CompilerMessage;
use emergent::libtest::TestCapture;
use emergent::test_runner::TestRunResult;
use emergent::Msg;
use emergent_presenter::Presenter;
use emergent_ui::WindowMsg;
use emergent_view::{IndexedTarget, View};

struct ColumnView<Element> {
    data: Vec<Element>,
}

impl<Element> View<Vec<Element>, Msg> for ColumnView<Element> {
    fn update(&mut self, state: &Vec<Element>) {
        unimplemented!()
    }

    fn present(&mut self, presenter: &mut Presenter<Msg>) {
        unimplemented!()
    }
}

struct Column2 {}

impl<C1, C2> View<(C1, C2), Msg> for Column2 {
    fn update(&mut self, state: &(C1, C2)) {
        unimplemented!()
    }

    fn present(&mut self, presenter: &mut Presenter<Msg>) {
        unimplemented!()
    }
}

struct AppView {
    compiler_messages: Vec<CompilerMessage>,
    test_results: Vec<TestCapture>,
    combined: Column2,
}

impl AppView {
    pub fn new() -> AppView {
        View {}
    }
}

impl View<App, Msg> for AppView {
    fn update(&mut self, state: &App) {
        match &state.test_run_result {
            Some(TestRunResult::CompilationFailed(compiler_messages, _)) => {
                self.compiler_messages.apply(&compiler_messages);
                self.test_results.update(&Vec::new());
            }
            Some(TestRunResult::TestsCaptured(compiler_messages, test_results)) => {
                self.compiler_messages.update(&compiler_messages);
                self.test_results.update(&test_results.0);
            }
            None => {
                self.compiler_messages.update(&Vec::new());
                self.test_results.update(&Vec::new());
            }
        }
        self.combined
            .update((&self.compiler_messages, &self.test_results));
    }

    fn present(&mut self, presenter: &mut Presenter<Msg>) {
        unimplemented!()
    }

    fn dispatch(&mut self, wm: WindowMsg) -> Option<Msg> {
        unimplemented!()
    }
}

*/
