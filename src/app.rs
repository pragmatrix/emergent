use cargo_metadata::diagnostic::DiagnosticLevel;
use crossbeam_channel::Receiver;
use emergent::compiler_message::ToDrawing;
use emergent::test_runner::{TestEnvironment, TestRunRequest, TestRunResult};
use emergent::test_watcher::{Notification, TestWatcher};
use emergent::{compiler_message, WindowModel};
use emergent::{test_watcher, Msg};
use emergent_presenter::{
    scroll, tab, AsData, Context, Data, Direction, IndexAccessible, IndexMappable, Reducible, View,
    ViewRenderer,
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
        let watcher = TestWatcher::begin_watching(req, test_environment, sender).unwrap();

        let emergent = App {
            watcher,
            notification_receiver: receiver,
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
        let create = |ctx: &mut Context| match &self.test_run_result {
            Some(TestRunResult::CompilationFailed(compiler_messages, _e)) => {
                let partition = compiler_messages
                    .as_data()
                    .partition(|c| match c.message.level {
                        DiagnosticLevel::Error => true,
                        _ => false,
                    });

                let (errors, rest) = partition.result;
                let errors = errors
                    .as_data()
                    .map_view(|_, cm| cm.to_drawing().into())
                    .reduce_scoped(ctx, "errors", Direction::Column);
                let rest = rest
                    .as_data()
                    .order_by(compiler_message::diagnostic_level_ordering)
                    .map_view(|_, cm| cm.to_drawing().into())
                    .reduce_scoped(ctx, "warnings", Direction::Column);

                (errors, rest, View::new())
            }

            Some(TestRunResult::TestsCaptured(compiler_messages, captures)) => {
                let partition = compiler_messages
                    .as_data()
                    .partition(|c| match c.message.level {
                        DiagnosticLevel::Error => true,
                        _ => false,
                    });

                let (errors, rest) = partition.result;
                let errors = errors
                    .as_data()
                    .map_view(|_, cm| cm.to_drawing().into())
                    .reduce_scoped(ctx, "errors", Direction::Column);
                let rest = rest
                    .as_data()
                    .order_by(compiler_message::diagnostic_level_ordering)
                    .map_view(|_, cm| cm.to_drawing().into())
                    .reduce_scoped(ctx, "warnings", Direction::Column);

                let captures = captures.0.as_data().map_view(|c, capture| {
                    let show_contents = !self.collapsed_tests.contains(&capture.name);
                    capture.present(c, show_contents)
                });

                let captures = captures.reduce_scoped(ctx, "captures", Direction::Column);

                (errors, rest, captures)
            }
            _ => {
                // TODO: no result yet (should we display some notification... running test, etc?)
                Default::default()
            }
        };

        tab::view(ctx, |ctx| {
            let (errors, warnings, tests) = create(ctx);
            vec![errors, warnings, tests]
        })
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
                name: "tests::test_in_mod_capture".into(),
                result: TestResult::Ok(),
                output: "CAPTURE_ME_IN_MOD\n".into()
            }));

            assert!(captures.contains(&TestCapture {
                name: "test_output_capture_multiline".into(),
                result: TestResult::Ok(),
                output: "CAPTURE_ME_LINE1\nCAPTURE_ME_LINE2\n".into()
            }));
        } else {
            panic!("no test results");
        }
    }
}
