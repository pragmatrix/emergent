use crossbeam_channel::Receiver;
use emergent::compiler_message::ToDrawing;
use emergent::test_runner::{TestEnvironment, TestRunRequest, TestRunResult};
use emergent::test_watcher::{Notification, TestWatcher};
use emergent::{test_watcher, Msg};
use emergent::{RenderPresentation, WindowApplicationMsg, WindowModel};
use emergent_presenter::{Direction, Presenter};
use emergent_ui::WindowMsg;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use tears::Cmd;

pub struct App {
    watcher: TestWatcher,
    notification_receiver: Receiver<test_watcher::Notification>,
    test_run_result: Option<TestRunResult>,
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

impl RenderPresentation<Msg> for App {
    fn render_presentation(&self, p: &mut Presenter<Msg>) {
        match &self.test_run_result {
            Some(TestRunResult::CompilationFailed(compiler_messages, _e)) => {
                p.stack_items(
                    Direction::Column,
                    compiler_messages,
                    |presenter, (_, cm)| presenter.draw(cm.to_drawing()),
                );
            }
            Some(TestRunResult::TestsCaptured(compiler_messages, captures)) => {
                debug!("{} tests captured", captures.0.len());
                p.stack_f(
                    Direction::Column,
                    &[
                        &|p| {
                            debug!("rendering {} compiler messages", compiler_messages.len());
                            p.stack_items(
                                Direction::Column,
                                compiler_messages,
                                |presenter, (_, cm)| presenter.draw(cm.to_drawing()),
                            )
                        },
                        &|p| {
                            debug!("rendering {} tests", captures.0.len());
                            p.stack_items(Direction::Column, &captures.0, |p, (_, capture)| {
                                let name = capture.name.clone();
                                let show_contents = !self.collapsed_tests.contains(&name);
                                capture.present(p, show_contents)
                            })
                        },
                    ],
                );
            }
            _ => {
                // TODO: no result yet (should we display some notification... running test, etc?)
            }
        }
    }
}
