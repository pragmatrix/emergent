use crate::libtest::TestCaptures;
use crate::test_runner::TestRunRequest;
use crate::test_watcher;
use crate::test_watcher::Notification;
use crossbeam_channel::Receiver;
use emergent_drawing::{scalar, Circle, DrawingTarget, Paint};
use tears::{Cmd, Model, View};

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

impl View<Box<Frame>> for Emergent {
    fn render(&self) -> Box<Frame> {
        let size = (256, 256);
        let frame = FnFrame {
            size,
            draw: move |target| {
                let (w, h): (scalar, scalar) = {
                    let (w, h) = size;
                    (w as _, h as _)
                };

                let (x, y) = (w / 2.0, h / 2.0);
                let r = w.min(h) / 2.0;

                let paint = Paint::default();
                target.draw(Circle((x, y).into(), r.into()).into(), &paint)
            },
        };

        Box::new(frame)
    }
}

/// A frame represents a sized and layouted, ready to be drawn
/// frame that renders to a DrawingTarget.
// TODO: do we need that at all?
pub trait Frame: Send {
    fn size(&self) -> (u32, u32);
    fn draw(&self, target: &mut DrawingTarget);
}

pub struct FnFrame<F: Fn(&mut DrawingTarget)> {
    size: (u32, u32),
    draw: F,
}

impl<F: Fn(&mut DrawingTarget)> Frame for FnFrame<F>
where
    F: Send,
{
    fn size(&self) -> (u32, u32) {
        self.size
    }

    fn draw(&self, target: &mut DrawingTarget) {
        (self.draw)(target);
    }
}
