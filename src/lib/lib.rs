//! A library that is part of the tests in its role as a "library under test".
#[test]
fn test_output_capture() {
    for i in 0..1000 {
        println!("CAPTURE_ME: {}", i);
    }
}
