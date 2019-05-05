use crate::capture::Capture;
use crate::libtest::{TestCapture, TestCaptures, TestResult};
use cargo::core::compiler;
use cargo::ops;
use std::io::Cursor;
use std::path::{Path, PathBuf};

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct TestRunRequest {
    directory: PathBuf,
    libtest_args: Vec<String>,
}

impl TestRunRequest {
    /// Creates a new TestRunRequest.
    pub fn new(project_directory: &Path) -> TestRunRequest {
        let libtest_args = [
            "--test-threads",
            "1",
            "--nocapture",
            "-Z",
            "unstable-options",
            "--format",
            "json",
        ]
        .iter()
        .map(|x| x.to_string())
        .collect();

        TestRunRequest {
            directory: project_directory.to_owned(),
            libtest_args,
        }
    }

    pub fn capture_tests(self) -> Result<TestCaptures, failure::Error> {
        let manifest_path = self.directory.join("Cargo.toml");
        let config = &cargo::Config::default()?;
        let workspace = &cargo::core::Workspace::new(&manifest_path, config)?;

        // WTF?
        let compile_filter = ops::CompileFilter::new(
            true,
            vec![],
            false,
            vec![],
            false,
            vec![],
            false,
            vec![],
            false,
            false,
        );

        let mut compile_options = ops::CompileOptions::new(&config, compiler::CompileMode::Test)?;
        compile_options.filter = compile_filter;

        let test_options = &ops::TestOptions {
            compile_opts: compile_options,
            no_run: false,
            no_fail_fast: false,
        };

        let capture = Capture::stdout();

        let test_error = ops::run_tests(workspace, test_options, &self.libtest_args)?;
        let captured = capture.end();
        println!(">>> CAPTURED BEGIN");
        println!("{}", String::from_utf8_lossy(&captured));
        println!(">>> CAPTURED END");

        let cursor = Cursor::new(captured);

        TestCaptures::from_output(cursor)
    }
}

#[cfg(test)]
use std::env;

#[test]
fn run_tests_self() {
    let request = TestRunRequest::new(&env::current_dir().unwrap());
    let captures = request.capture_tests().unwrap();
    println!("captures:\n{:?}", captures);

    let captures = captures.0;

    assert!(captures.contains(&TestCapture {
        name: "test_output_capture".into(),
        result: TestResult::Ok(),
        output: "CAPTURE_ME\n".into()
    }));

    assert!(captures.contains(&TestCapture {
        name: "mod_test::test_in_mod_capture".into(),
        result: TestResult::Ok(),
        output: "CAPTURE_ME_IN_MOD\n".into()
    }));

    assert!(captures.contains(&TestCapture {
        name: "test_output_capture_multiline".into(),
        result: TestResult::Ok(),
        output: "CAPTURE_ME_LINE1\nCAPTURE_ME_LINE2\n".into()
    }));
}
