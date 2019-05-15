//! The framebuilder receives frame requests a specific window size and sends them
//! back when they are produced.

use crossbeam_channel::{Receiver, SendError, Sender};
use emergent::ThreadJoiner;
use emergent_drawing::{scalar, Circle, DrawingTarget, Paint, Shape};
use std::thread;

/// A frame represents a sized and layouted, ready to be drawn
/// frame that renders to a DrawingTarget.
pub trait Frame: Send {
    fn size(&self) -> (u32, u32);
    fn draw(&self, target: &mut DrawingTarget);
}

// A request to produce and send back a frame.
struct BuildFrameRequest {
    pub size: (u32, u32),
    pub notify: Box<Fn(Box<Frame>) + Send>,
}

struct FnFrame<F: Fn(&mut DrawingTarget)> {
    size: (u32, u32),
    draw_fn: F,
}

impl<F: Fn(&mut DrawingTarget)> Frame for FnFrame<F>
where
    F: Send,
{
    fn size(&self) -> (u32, u32) {
        self.size
    }

    fn draw(&self, target: &mut DrawingTarget) {
        (self.draw_fn)(target);
    }
}

pub struct FrameBuilder {
    // note: order is significant, sender has to be dropped first.
    sender: Sender<BuildFrameRequest>,
    #[allow(dead_code)]
    thread: ThreadJoiner,
}

impl FrameBuilder {
    pub fn new() -> Self {
        let (sender, receiver) = crossbeam_channel::unbounded();

        FrameBuilder {
            sender,
            thread: ThreadJoiner::from_join_handle(thread::spawn(|| Self::thread(receiver))),
        }
    }

    /// Request a new frame.
    pub fn request(&self, size: (u32, u32), receiver: impl Fn(Box<Frame>) + Send + 'static) {
        self.sender
            .send(BuildFrameRequest {
                size,
                notify: Box::new(receiver),
            })
            .expect("can't request a new frame, is the FrameBuilder gone?");
    }

    /// The thread of the frame builder.
    fn thread(receiver: Receiver<BuildFrameRequest>) {
        loop {
            match receiver.recv() {
                Ok(request) => {
                    let frame = Self::build(request.size);
                    (request.notify)(Box::new(frame));
                }
                Err(e) => {
                    println!("receiver error {:?}, ending thread", e);
                    break;
                }
            }
        }
    }

    fn build(size: (u32, u32)) -> impl Frame {
        FnFrame {
            size,
            draw_fn: move |drawing_target| {
                let (w, h): (scalar, scalar) = {
                    let (w, h) = size;
                    (w as _, h as _)
                };

                let (x, y) = (w / 2.0, h / 2.0);
                let r = w.min(h) / 2.0;

                let paint = Paint::default();
                drawing_target.draw(Circle((x, y).into(), r.into()).into(), &paint)
            },
        }
    }
}
