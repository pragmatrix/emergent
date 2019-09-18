use crate::capture::Capture;
use cargo::core::compiler;
use cargo::ops;
use cargo::ops::{FilterRule, LibRule};
use cargo_metadata::CompilerMessage;
use emergent::libtest::TestCaptures;
use emergent::DPI;
use emergent_drawing::FromTestEnvironment;
use std::env;
use std::io::Cursor;
use std::path::{Path, PathBuf};

#[derive(Clone, PartialEq, Debug)]
pub struct TestRunRequest {
    pub project_directory: PathBuf,
}

#[derive(Clone, PartialEq, Debug)]
pub struct TestEnvironment {
    pub dpi: DPI,
}

impl FromTestEnvironment for TestEnvironment {
    fn from_test_environment() -> Self {
        Self {
            dpi: DPI::from_test_environment(),
        }
    }
}

#[derive(Debug)]
pub enum TestRunResult {
    CompilationFailed(Vec<CompilerMessage>, failure::Error),
    TestsCaptured(Vec<CompilerMessage>, TestCaptures),
}

impl TestRunRequest {
    /// Creates a new TestRunRequest for the library
    /// in the given cargo project directory.
    pub fn new_lib(project_directory: &Path) -> TestRunRequest {
        TestRunRequest {
            project_directory: project_directory.to_owned(),
        }
    }

    pub fn capture_tests(
        &self,
        environment: TestEnvironment,
    ) -> Result<TestRunResult, failure::Error> {
        let manifest_path = self.project_directory.join("Cargo.toml");

        // TODO: verify if this is correct (taken from cargo::Config::new()).
        let current_dir = env::current_dir().unwrap();
        let home_dir = cargo::util::config::homedir(&current_dir).unwrap();

        let (test_result, captured) = {
            /*
            this code may be used to capture / supporess the error output (for example Fresh lines)

            let mut output = Vec::new();
            let mut output_cursor = Cursor::new(output);
            let boxed_output_cursor: Box<dyn Write> = Box::new(output_cursor);
            let shell = cargo::core::Shell::from_write(boxed_output_cursor);
            */

            // stdout can not be captured this way there is an actual println! used
            // although there are ways to capture println! calls I can remember, we need
            // also capture test output.

            let shell = cargo::core::Shell::new();
            let mut config = cargo::Config::new(shell, current_dir, home_dir);
            env::set_var("EMERGENT_TEST_DPI", environment.dpi.0.to_string());
            let normalized_path = cargo::util::paths::normalize_path(&manifest_path);
            dbg!(&normalized_path);
            let workspace = &cargo::core::Workspace::new(&normalized_path, &config)?;

            // build library only for now.
            let compile_filter = ops::CompileFilter::Only {
                all_targets: false,
                // Prefer to test the library of the package for now only.
                lib: LibRule::True,
                bins: FilterRule::Just(vec![]),
                examples: FilterRule::Just(vec![]),
                tests: FilterRule::Just(vec![]),
                benches: FilterRule::Just(vec![]),
            };

            let mut compile_options =
                ops::CompileOptions::new(&config, compiler::CompileMode::Test)?;
            compile_options.build_config.message_format = compiler::MessageFormat::Json {
                render_diagnostics: false,
                short: false,
                ansi: true,
            };
            compile_options.filter = compile_filter;

            let test_options = &ops::TestOptions {
                compile_opts: compile_options,
                no_run: false,
                no_fail_fast: false,
            };

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

            let capture = Capture::stdout();
            let test_result = ops::run_tests(workspace, test_options, &libtest_args);
            (test_result, capture.end())
        };

        // TODO: use a log for that

        println!(">>> TEST RESULT: {:?}", test_result);
        println!(">>> CAPTURED BEGIN");
        println!("{}", String::from_utf8_lossy(&captured));
        println!(">>> CAPTURED END");

        let cursor = Cursor::new(&captured);

        // parse messages from cargo:
        let mut iterator = cargo_metadata::parse_messages(cursor);
        let compiler_messages = {
            let mut messages = Vec::new();
            for msg in &mut iterator {
                match msg {
                    Ok(msg) => {
                        if let cargo_metadata::Message::CompilerMessage(compiler_message) = msg {
                            messages.push(compiler_message)
                        }
                    }
                    Err(_) => break,
                }
            }

            messages
        };

        if let Err(e) = test_result {
            return Ok(TestRunResult::CompilationFailed(compiler_messages, e));
        }

        // and interpret the rest as test captures.
        let byte_offset = iterator.byte_offset();
        let rest = &captured[byte_offset..];

        // TODO: perhaps it's better to separate the compilation of the test code and the running of it?

        Ok(TestRunResult::TestsCaptured(
            compiler_messages,
            TestCaptures::from_output(Cursor::new(rest))?,
        ))
    }
}

#[cfg(test)]
pub mod tests {
    use crate::test_runner::{TestEnvironment, TestRunRequest, TestRunResult};
    use emergent::libtest::{TestCapture, TestResult};
    use emergent_drawing::FromTestEnvironment;
    use std::env;

    #[test]
    fn run_tests_self() {
        let request = TestRunRequest::new_lib(
            &env::current_dir().unwrap(),
            TestEnvironment::from_test_environment(),
        );
        if let TestRunResult::TestsCaptured(_, captures) = request.capture_tests().unwrap() {
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
        } else {
            assert!(false);
        }
    }
}

impl TestEnvironment {
    pub fn new(dpi: DPI) -> Self {
        TestEnvironment { dpi }
    }
}
