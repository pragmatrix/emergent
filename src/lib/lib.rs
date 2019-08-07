//! A library that is part of the tests in its role as a "library under test".
use std::thread::JoinHandle;

mod ui;

#[test]
fn test_output_capture() {
    println!("CAPTURE_ME");
}

#[test]
fn test_output_capture_multiline() {
    println!("CAPTURE_ME_LINE1");
    println!("CAPTURE_ME_LINE2");
}

mod mod_test {
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
