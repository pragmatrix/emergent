//! A library that is part of the tests in its role as a "library under test".
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
    #[test]
    fn test_in_mod_capture() {
        println!("CAPTURE_ME_IN_MOD")
    }
}
