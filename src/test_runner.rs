use crate::capture::Capture;
use crate::libtest::TestCaptures;
use cargo::core::compiler;
use cargo::ops;
use cargo::ops::{FilterRule, LibRule};
use std::io::Cursor;
use std::path::{Path, PathBuf};

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct TestRunRequest {
    pub project_directory: PathBuf,
}

impl TestRunRequest {
    /// Creates a new TestRunRequest for the library
    /// in the given cargo project directory.
    pub fn new_lib(project_directory: &Path) -> TestRunRequest {
        TestRunRequest {
            project_directory: project_directory.to_owned(),
        }
    }

    pub fn capture_tests(&self) -> Result<TestCaptures, failure::Error> {
        let manifest_path = self.project_directory.join("Cargo.toml");
        let config = &cargo::Config::default()?;
        let normalized_path = cargo::util::paths::normalize_path(&manifest_path);
        dbg!(&normalized_path);
        let workspace = &cargo::core::Workspace::new(&normalized_path, config)?;

        // build library only for now.
        let compile_filter = ops::CompileFilter::Only {
            all_targets: false,
            lib: LibRule::True,
            bins: FilterRule::Just(vec![]),
            examples: FilterRule::Just(vec![]),
            tests: FilterRule::Just(vec![]),
            benches: FilterRule::Just(vec![]),
        };

        let mut compile_options = ops::CompileOptions::new(&config, compiler::CompileMode::Test)?;
        compile_options.filter = compile_filter;

        let test_options = &ops::TestOptions {
            compile_opts: compile_options,
            no_run: false,
            no_fail_fast: false,
        };

        let capture = Capture::stdout();

        // we need a very specific set of arguments to make precise capturing of the output work.
        let libtest_args: Vec<&str> = vec![
            "--test-threads",
            "1",
            "--nocapture",
            "-Z",
            "unstable-options",
            "--format",
            "json",
        ];

        let _test_error = ops::run_tests(workspace, test_options, &libtest_args)?;
        let captured = capture.end();
        println!(">>> CAPTURED BEGIN");
        println!("{}", String::from_utf8_lossy(&captured));
        println!(">>> CAPTURED END");

        let cursor = Cursor::new(captured);

        TestCaptures::from_output(cursor)
    }
}

#[cfg(test)]
pub mod tests {
    use crate::libtest::{TestCapture, TestResult};
    use crate::test_runner::TestRunRequest;
    use std::env;

    #[test]
    fn run_tests_self() {
        let request = TestRunRequest::new_lib(&env::current_dir().unwrap());
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

}
