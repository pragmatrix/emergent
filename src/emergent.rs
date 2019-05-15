use crate::libtest::TestCaptures;
use crate::test_runner::TestRunRequest;
use crate::test_watcher;
use crate::test_watcher::Notification;
use crossbeam_channel::Receiver;
use tea_rs::{Cmd, Model};

#[derive(Debug)]
pub enum Event {
    WindowResized((usize, usize)),
    WatcherNotification(test_watcher::Notification),
}

pub struct Emergent {
    notification_receiver: Receiver<test_watcher::Notification>,
    test_captures: TestCaptures,
    latest_test_error: Option<String>,
}

impl Emergent {
    pub fn new(req: TestRunRequest) -> (Self, Cmd<Event>) {
        let (sender, receiver) = crossbeam_channel::unbounded();
        test_watcher::begin_watching(req, sender).unwrap();

        let emergent = Self {
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
        println!("{:?}", event);
        match event {
            Event::WindowResized(_) => {}
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
                let receiver = self.notification_receiver.clone();
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
