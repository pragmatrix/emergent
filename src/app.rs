use crossbeam_channel::Receiver;
use emergent::compiler_message::ToDrawing;
use emergent::test_runner::{TestEnvironment, TestRunRequest, TestRunResult};
use emergent::test_watcher::{Notification, TestWatcher};
use emergent::WindowModel;
use emergent::{test_watcher, Msg};
use emergent_presenter::{
    scroll, Context, Data, Direction, IndexMappable, Reducible, View, ViewRenderer,
};
use std::collections::HashSet;
use tears::Cmd;

pub struct App {
    watcher: TestWatcher,
    notification_receiver: Receiver<test_watcher::Notification>,

    pub(crate) test_run_result: Option<TestRunResult>,
    latest_test_error: Option<String>,
    collapsed_tests: HashSet<String>,
}

impl App {
    pub fn new(req: TestRunRequest, test_environment: TestEnvironment) -> (Self, Cmd<Msg>) {
        let (sender, receiver) = crossbeam_channel::unbounded();
        let watcher =
            TestWatcher::begin_watching(req.clone(), test_environment.clone(), sender).unwrap();

        let emergent = App {
            watcher,
            notification_receiver: receiver.clone(),
            test_run_result: None,
            latest_test_error: None,
            // TODO: this is part of the persistent state.
            collapsed_tests: HashSet::new(),
        };

        let cmd = emergent.receive_watcher_notifications();
        (emergent, cmd)
    }
}

impl WindowModel<Msg> for App {
    fn update(&mut self, event: Msg) -> Cmd<Msg> {
        debug!("{:?}", &event);
        match event {
            Msg::WatcherNotification(wn) => match wn {
                Ok(notification) => return self.update_watcher(notification),
                Err(e) => {
                    panic!("watcher notification error: {}", e.to_string());
                }
            },
            Msg::ToggleTestcase { name } => {
                if self.collapsed_tests.contains(&name) {
                    self.collapsed_tests.remove(&name);
                } else {
                    self.collapsed_tests.insert(name);
                }
            }
            Msg::RerunTestcases(environment) => {
                self.watcher.update_environment(environment);
            }
        }
        self.receive_watcher_notifications()
    }
}

impl App {
    fn update_watcher(&mut self, notification: test_watcher::Notification) -> Cmd<Msg> {
        match notification {
            Notification::TestRunCompleted(r) => {
                match r {
                    Ok(run_result) => {
                        self.test_run_result = Some(run_result);
                        self.latest_test_error = None;
                    }
                    Err(e) => {
                        self.latest_test_error = Some(e.to_string());
                    }
                }
                self.receive_watcher_notifications()
            }

            Notification::WatcherStopped(e) => {
                // TODO: restart it here?
                panic!("watcher stopped: {}", e.to_string());
                // self.latest_test_error = Some(e.to_string());
                // Cmd::None
            }
        }
    }

    /// Returns a command that receives watcher notifications.
    fn receive_watcher_notifications(&self) -> Cmd<Msg> {
        let receiver = self.notification_receiver.clone();
        Cmd::from(move || Msg::WatcherNotification(receiver.recv().map_err(|e| e.into())))
    }
}

impl ViewRenderer<Msg> for App {
    fn render_view(&self, ctx: Context) -> View<Msg> {
        let content = |ctx: Context| match &self.test_run_result {
            Some(TestRunResult::CompilationFailed(compiler_messages, _e)) => {
                Data::new(compiler_messages)
                    .map(|_, cm| cm.to_drawing().into())
                    .reduce(ctx, Direction::Column)
            }

            Some(TestRunResult::TestsCaptured(compiler_messages, captures)) => {
                debug!("{} tests captured", captures.0.len());

                // only render compiler messages if we don't have any
                // captures to show (later we can just collapse them by default).
                let compiler_messages = if captures.0.is_empty() {
                    compiler_messages.as_slice()
                } else {
                    &[]
                };

                let compiler_messages =
                    Data::new(compiler_messages).map(|_, cm| cm.to_drawing().into());

                let captures = Data::new(&captures.0).map(|c, capture| {
                    let show_contents = !self.collapsed_tests.contains(&capture.name);
                    let inner_view = capture.present(c, show_contents);
                    inner_view
                });

                let elements = compiler_messages.extend(&captures);
                let view = elements.reduce(ctx, Direction::Column);
                view
            }
            _ => {
                // TODO: no result yet (should we display some notification... running test, etc?)
                View::new()
            }
        };

        scroll::view(ctx, content)
    }
}

/// TestRunner tests.
///
/// These tests are not in the library, because they would interfere with emergent itself.
#[cfg(test)]
pub mod tests {
    use emergent::libtest::{TestCapture, TestResult};
    use emergent::test_runner::{TestEnvironment, TestRunRequest, TestRunResult};
    use emergent_drawing::FromTestEnvironment;
    use std::env;

    #[test]
    fn run_tests_self() {
        let request = TestRunRequest::new_lib(&env::current_dir().unwrap());
        if let TestRunResult::TestsCaptured(_, captures) = request
            .capture_tests(TestEnvironment::from_test_environment())
            .unwrap()
        {
            info!("captures:\n{:?}", captures);

            let captures = captures.0;

            assert!(captures.contains(&TestCapture {
                name: "test_output_capture".into(),
                result: TestResult::Ok(),
                output: "CAPTURE_ME\n".into()
            }));

            assert!(captures.contains(&TestCapture {
                name: "mod_test::test_in_mod_capture".into(),
                result: TestResult::Ok(),
                output: "CAPTURE_ME_IN_MOD\n".into()
            }));

            assert!(captures.contains(&TestCapture {
                name: "test_output_capture_multiline".into(),
                result: TestResult::Ok(),
                output: "CAPTURE_ME_LINE1\nCAPTURE_ME_LINE2\n".into()
            }));
        } else {
            assert!(false);
        }
    }
}
