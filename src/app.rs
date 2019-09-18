use crate::test_runner::{TestRunRequest, TestRunResult};
use crate::test_watcher;
use crate::test_watcher::Notification;
use crossbeam_channel::Receiver;
use emergent::compiler_message::ToDrawing;
use emergent::skia::text::PrimitiveText;
use emergent::{FrameLayout, RenderPresentation, WindowModel};
use emergent_drawing::simple_layout::SimpleLayout;
use emergent_presentation::{Gesture, Present, Presentation};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use tears::Cmd;

#[derive(Debug, Serialize, Deserialize)]
pub enum Msg {
    #[serde(skip)]
    WatcherNotification(test_watcher::Notification),
    ToggleTestcase {
        name: String,
    },
}

pub struct App {
    notification_receiver: Receiver<test_watcher::Notification>,
    test_run_result: Option<TestRunResult>,
    latest_test_error: Option<String>,
    collapsed_tests: HashSet<String>,
}

impl App {
    pub fn new(req: TestRunRequest) -> (Self, Cmd<Msg>) {
        let (sender, receiver) = crossbeam_channel::unbounded();
        test_watcher::begin_watching(req, sender).unwrap();

        let emergent = Self {
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
            Msg::WatcherNotification(wn) => {
                self.update_watcher(wn);
                return self.receive_watcher_notifications();
            }
            Msg::ToggleTestcase { name } => {
                if self.collapsed_tests.contains(&name) {
                    self.collapsed_tests.remove(&name);
                } else {
                    self.collapsed_tests.insert(name);
                }
            }
        }
        Cmd::None
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

            Notification::WatcherStopped(r) => {
                match r {
                    Ok(()) => panic!("watcher stopped"),
                    Err(e) => self.latest_test_error = Some(e.to_string()),
                }
                Cmd::None
            }
        }
    }

    /// Returns a command that receives watcher notifications.
    fn receive_watcher_notifications(&self) -> Cmd<Msg> {
        let receiver = self.notification_receiver.clone();
        Cmd::from(move || Msg::WatcherNotification(receiver.recv().unwrap()))
    }
}

impl RenderPresentation<Msg> for App {
    fn render_presentation(&self, layout: &FrameLayout) -> Presentation<Msg> {
        let measure = PrimitiveText::new(layout.dpi);
        let test_run_presentations = {
            match &self.test_run_result {
                Some(TestRunResult::CompilationFailed(compiler_messages, _e)) => compiler_messages
                    .iter()
                    .map(|cm| cm.to_drawing().present())
                    .collect(),
                Some(TestRunResult::TestsCaptured(compiler_messages, captures)) => {
                    let mut presentations = Vec::new();
                    for cm in compiler_messages {
                        presentations.push(cm.to_drawing().present());
                    }

                    // TODO: implement Iter in TestCaptures
                    for capture in captures.0.iter() {
                        // TODO: add a nice drawing combinator.
                        // TODO: avoid the access of 0!
                        let name = &capture.name;
                        let tap_gesture = {
                            let name = name.clone();
                            Gesture::tap(move |_| Msg::ToggleTestcase { name: name.clone() })
                        };
                        let show_contents = !self.collapsed_tests.contains(name);

                        presentations.push(capture.present(
                            tap_gesture.into(),
                            show_contents,
                            &measure,
                        ))
                    }
                    presentations
                }
                _ => Vec::new(),
            }
        };

        Presentation::BackToFront(Presentation::layout_vertically(
            test_run_presentations,
            &measure,
        ))
    }
}
