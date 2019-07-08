use crate::frame::Frame;
use crate::libtest::{TestCapture, TestCaptures};
use crate::test_runner::TestRunRequest;
use crate::test_watcher;
use crate::test_watcher::Notification;
use crossbeam_channel::Receiver;
use emergent_drawing::{Drawing, DrawingTarget, Paint, Shape};
use tears::{Cmd, Model, View};

#[derive(Debug)]
pub enum Event {
    WindowResized((u32, u32)),
    WatcherNotification(test_watcher::Notification),
}

pub struct Emergent {
    window_size: (u32, u32),
    notification_receiver: Receiver<test_watcher::Notification>,
    test_captures: TestCaptures,
    latest_test_error: Option<String>,
}

impl Emergent {
    pub fn new(window_size: (u32, u32), req: TestRunRequest) -> (Self, Cmd<Event>) {
        let (sender, receiver) = crossbeam_channel::unbounded();
        test_watcher::begin_watching(req, sender).unwrap();

        let emergent = Self {
            window_size,
            notification_receiver: receiver.clone(),
            test_captures: TestCaptures::default(),
            latest_test_error: None,
        };

        let cmd = emergent.receive_watcher_notifications();
        (emergent, cmd)
    }
}

impl Model<Event> for Emergent {
    fn update(&mut self, event: Event) -> Cmd<Event> {
        dbg!(&event);
        match event {
            Event::WindowResized(new_size) => self.window_size = new_size,
            Event::WatcherNotification(wn) => {
                self.update_watcher(wn);
                return self.receive_watcher_notifications();
            }
        }
        Cmd::None
    }
}

impl Emergent {
    fn update_watcher(&mut self, notification: test_watcher::Notification) -> Cmd<Event> {
        match notification {
            Notification::TestRunCompleted(r) => {
                match r {
                    Ok(captures) => {
                        self.test_captures = captures;
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

    /// A command to receive watcher notifications.
    fn receive_watcher_notifications(&self) -> Cmd<Event> {
        let receiver = self.notification_receiver.clone();
        Cmd::from(move || Event::WatcherNotification(receiver.recv().unwrap()))
    }
}

impl View<Frame> for Emergent {
    fn render(&self) -> Frame {
        let mut drawing = Drawing::new();

        // TODO: implement Iter in TestCaptures
        for capture in self.test_captures.0.iter() {
            // TODO: add a nice paintings combinator.
            drawing.0.extend(render_capture(capture).0)
        }

        Frame {
            size: self.window_size,
            drawing,
        }
    }
}

fn render_capture(capture: &TestCapture) -> Drawing {
    if !capture.output.starts_with("> ") {
        return Drawing::new();
    };

    // TODO: handle parse errors:
    serde_json::from_str(&capture.output[2..]).unwrap()
}
