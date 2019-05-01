//! A library that is part of the tests in its role as a "library under test".
#[test]
fn test_output_capture() {
    println!("CAPTURE_ME");
}

mod mod_test {
    #[test]
    fn test_in_mod_capture() {
        println!("CAPTURE_ME_IN_MOD")
    }

    #[test]
    fn test_expect_fail() {
        assert_ne!(1, 1)
    }
}
