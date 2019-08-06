use crate::frame::Frame;
use crate::libtest::{TestCapture, TestCaptures};
use crate::test_runner::TestRunRequest;
use crate::test_watcher;
use crate::test_watcher::Notification;
use crossbeam_channel::Receiver;
use emergent_drawing::functions::text;
use emergent_drawing::{
    font, Drawing, DrawingFastBounds, DrawingTarget, Font, MeasureText, Paint, Point,
};
use std::borrow::Borrow;
use tears::{Cmd, Model, View};

#[derive(Debug)]
pub enum Event {
    WindowResized((u32, u32)),
    WatcherNotification(test_watcher::Notification),
}

pub struct Emergent {
    measure_text: Box<dyn MeasureText + Send>,
    window_size: (u32, u32),
    notification_receiver: Receiver<test_watcher::Notification>,
    test_captures: TestCaptures,
    latest_test_error: Option<String>,
}

impl Emergent {
    pub fn new(
        measure_text: impl MeasureText + Send + 'static,
        window_size: (u32, u32),
        req: TestRunRequest,
    ) -> (Self, Cmd<Event>) {
        let (sender, receiver) = crossbeam_channel::unbounded();
        test_watcher::begin_watching(req, sender).unwrap();

        let emergent = Self {
            measure_text: Box::new(measure_text),
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

    /// Returns a command that receives watcher notifications.
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
            // TODO: add a nice drawing combinator.
            // TODO: avoid the access of 0!
            drawing.0.extend(capture.render(&*self.measure_text).0)
        }

        // TODO: we probably need a composer for drawings.
        Frame {
            size: self.window_size,
            drawing,
        }
    }
}

impl TestCapture {
    fn render(&self, measure_text: &dyn MeasureText) -> Drawing {
        let header = self.render_header();
        dbg!(header.fast_bounds(measure_text));
        self.render_output()
    }

    fn render_header(&self) -> Drawing {
        // TODO: const fn? once_cell, the empty string is converted to a String which
        // is not const_fn.
        let header_font = &Font::new("", font::Style::NORMAL, font::Size::new(20.0));
        let mut target = Drawing::new();
        let text = text(Point::new(0.0, 20.0), &self.name, header_font);
        let paint = &Paint::default();
        target.draw(&text.into(), paint);
        target
    }

    fn render_output(&self) -> Drawing {
        // TODO: render invalid output as text and mark it appropriately
        if !self.output.starts_with("> ") {
            return Drawing::new();
        };

        // TODO: handle parse errors:
        serde_json::from_str(&self.output[2..]).unwrap()
    }
}
