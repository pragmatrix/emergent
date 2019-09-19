//! The library of the emergent testrunner.

#[macro_use]
extern crate log;

use std::thread::JoinHandle;

mod frame;
pub use frame::*;

mod hit_test;
pub use hit_test::*;

pub mod compiler_message;

pub mod libtest;

mod line_breaking;
pub use line_breaking::*;

mod presentations;

pub mod skia;
mod ui;

mod window;
pub use window::*;

mod window_application;
pub use window_application::*;

mod window_msg;
pub use window_msg::*;

mod winit_window;
use emergent_presentation::Presentation;
pub use winit_window::*;

#[test]
fn test_output_capture() {
    println!("CAPTURE_ME");
}

#[test]
fn test_output_capture_multiline() {
    println!("CAPTURE_ME_LINE1");
    println!("CAPTURE_ME_LINE2");
}

#[cfg(test)]
mod tests {
    use std::env;

    #[test]
    fn test_in_mod_capture() {
        println!("CAPTURE_ME_IN_MOD")
    }

    /*
    #[test]
    fn create_owned_canvas() {
        let typeface = Typeface::default();
        let font = Font::from_typeface_with_size(&typeface, 20.0);
        let measured = font.measure_str("Hello World", None);
        println!("measured: {:?}", measured);
    }*/

    #[test]
    fn env() {
        println!("{}", env::var("CARGO_MANIFEST_DIR").unwrap());
    }
}

pub struct ThreadJoiner(Option<JoinHandle<()>>);

impl Drop for ThreadJoiner {
    fn drop(&mut self) {
        self.0.take().unwrap().join().unwrap();
    }
}

impl ThreadJoiner {
    pub fn from_join_handle(handle: JoinHandle<()>) -> Self {
        ThreadJoiner(Some(handle))
    }
}

/// A function to produce a presentation with the given layout.
pub trait RenderPresentation<Msg> {
    fn render_presentation(&self, layout: &FrameLayout) -> Presentation<Msg>;
}
