use crate::libtest::TestCapture;
use crate::test_runner::{TestRunRequest, TestRunResult};
use crate::test_watcher;
use crate::test_watcher::Notification;
use crossbeam_channel::Receiver;
use emergent::compiler_message::ToDrawing;
use emergent::skia::text::SimpleText;
use emergent::{AreaLayout, Frame};
use emergent_drawing::functions::{paint, text};
use emergent_drawing::simple_layout::SimpleLayout;
use emergent_drawing::{font, BackToFront, Drawing, DrawingTarget, Font, MeasureText};
use emergent_presentation::{Area, Present, Presentation};
use tears::{Cmd, Model, View};

#[derive(Debug)]
pub enum Event {
    AreaLayoutChanged(AreaLayout),
    WatcherNotification(test_watcher::Notification),
    Refresh,
}

pub struct App {
    area_layout: AreaLayout,
    notification_receiver: Receiver<test_watcher::Notification>,
    test_run_result: Option<TestRunResult>,
    latest_test_error: Option<String>,
}

impl App {
    pub fn new(area_layout: AreaLayout, req: TestRunRequest) -> (Self, Cmd<Event>) {
        let (sender, receiver) = crossbeam_channel::unbounded();
        test_watcher::begin_watching(req, sender).unwrap();

        let emergent = Self {
            area_layout,
            notification_receiver: receiver.clone(),
            test_run_result: None,
            latest_test_error: None,
        };

        let cmd = emergent.receive_watcher_notifications();
        (emergent, cmd)
    }
}

impl Model<Event> for App {
    fn update(&mut self, event: Event) -> Cmd<Event> {
        debug!("{:?}", &event);
        match event {
            Event::AreaLayoutChanged(area_layout) => self.area_layout = area_layout,
            Event::WatcherNotification(wn) => {
                self.update_watcher(wn);
                return self.receive_watcher_notifications();
            }
            Event::Refresh => {}
        }
        Cmd::None
    }
}

impl App {
    fn update_watcher(&mut self, notification: test_watcher::Notification) -> Cmd<Event> {
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
    fn receive_watcher_notifications(&self) -> Cmd<Event> {
        let receiver = self.notification_receiver.clone();
        Cmd::from(move || Event::WatcherNotification(receiver.recv().unwrap()))
    }
}

impl View<Frame> for App {
    fn render(&self) -> Frame {
        let measure = SimpleText::new(self.area_layout.dpi);
        let test_run_presentations = {
            match &self.test_run_result {
                Some(TestRunResult::CompilationFailed(compiler_messages, e)) => compiler_messages
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
                        presentations.push(capture.present(&measure))
                    }
                    presentations
                }
                _ => Vec::new(),
            }
        };

        let presentation = Presentation::BackToFront(Presentation::layout_vertically(
            test_run_presentations,
            &measure,
        ));

        Frame {
            area: self.area_layout,
            presentation,
        }
    }
}

impl TestCapture {
    fn present(&self, measure: &dyn MeasureText) -> Presentation {
        let header = self.present_header();
        let output = self.draw_output().present();
        Presentation::layout_vertically(vec![header, output], measure)
            .back_to_front()
            .scoped(&self.name)
    }

    pub const HEADER_AREA: Area = Area::new("header");

    fn present_header(&self) -> Presentation {
        // TODO: const fn? once_cell, the empty string is converted to a String which
        // is not const_fn.
        let header_font = &Font::new("", font::Style::NORMAL, font::Size::new(20.0));
        let mut drawing = Drawing::new();
        let text = text(&self.name, header_font, None);
        drawing.draw_shape(&text.into(), paint());
        drawing.present().in_area(Self::HEADER_AREA)
    }

    fn draw_output(&self) -> Drawing {
        // TODO: render invalid output as text and mark it appropriately
        if !self.output.starts_with("> ") {
            return Drawing::new();
        };

        // TODO: handle parse errors:
        serde_json::from_str(&self.output[2..]).unwrap()
    }
}
