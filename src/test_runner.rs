use crate::capture::Capture;
use cargo::core::compiler;
use cargo::ops;
use std::path::{Path, PathBuf};

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct TestRunRequest {
    directory: PathBuf,
    command: String,
    args: Vec<String>,
    libtest_args: Vec<String>,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct TestName(Vec<String>);

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct TestRun {
    name: TestName,
    output: String,
}

#[derive(Debug)]
pub struct TestRunResult {}

impl TestRunRequest {
    /// Creates a new TestRunRequest.
    pub fn new(project_directory: &Path) -> TestRunRequest {
        let command = "cargo test";
        let args = vec![];
        let libtest_args = vec![
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
            command: command.to_string(),
            args,
            libtest_args,
        }
    }

    pub fn run(self) -> Result<TestRunResult, failure::Error> {
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
        /*
        let compile_filter = ops::CompileFilter::Only {
            all_targets: false,
            lib: true,
            bins: ops::FilterRule::Just(vec![]),
            examples: ops::FilterRule::Just(vec![]),
            benches: ops::FilterRule::Just(vec![]),
            tests: ops::FilterRule::Just(vec![]),
        };
        */

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
        println!(
            ">>>OUT: \n{}\n<<<END OUT",
            String::from_utf8(captured).unwrap()
        );

        Ok(TestRunResult {})
    }
}

#[cfg(test)]
use std::env;

#[test]
fn run_tests_self() {
    let request = TestRunRequest::new(&env::current_dir().unwrap());
    println!(">>> IN");

    request.run().unwrap();

    println!("<<< OUT");
}
