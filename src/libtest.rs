// https://github.com/rust-lang/libtest/blob/master/libtest/formatters/json.rs

use failure::Fail;
use serde_json::Value;
use std::convert::TryInto;
use std::io;
use std::io::{BufRead, BufReader, Read};

#[derive(Clone, PartialEq, Eq, Default, Debug)]
pub struct TestCaptures(pub Vec<TestCapture>);

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct TestCapture {
    pub name: String,
    pub result: TestResult,
    pub output: String,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum TestResult {
    Ok(),
    Failed(Option<ExtraData>),
    Ignored,
    AllowedFail,
    // TODO: Bench
    Timeout,
}

#[derive(Clone, PartialEq, Eq, Debug)]
enum ExtraData {
    Message(String),
    StdOut(String),
}

#[derive(Fail, Debug)]
pub enum TestResultsError {
    #[fail(display = "expected suite start event {:?}", _0)]
    ExpectedSuiteStart(Event),
    #[fail(display = "expected test start at test no {}, got: {:?}", index, event)]
    ExpectedTestStart { index: usize, event: Event },
}

trait EventReader {
    fn read_next_line(&mut self) -> Result<String, io::Error>;

    fn read_line_as_event(&mut self) -> Result<Event, failure::Error> {
        let line = self.read_next_line()?;
        parse_event(&line)
    }
}

fn parse_event(line: &String) -> Result<Event, failure::Error> {
    let value = serde_json::from_str(line)?;
    Ok(Event::from_json(&value)?)
}

impl<R: Read> EventReader for BufReader<R> {
    fn read_next_line(&mut self) -> Result<String, io::Error> {
        let mut l: String = String::new();
        if 0 == BufRead::read_line(self, &mut l)? {
            Err(io::ErrorKind::UnexpectedEof.into())
        } else {
            Ok(l)
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Event {
    SuiteStart {
        test_count: usize,
    },
    SuiteFinish {
        result: OkOrFailed,
        passed: usize,
        failed: usize,
        allowed_fail: usize,
        ignored: usize,
        measured: usize,
        filtered_out: usize,
    },
    TestStart {
        name: String,
    },
    TestFinish {
        name: String,
        result: TestResult,
    },
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum OkOrFailed {
    Ok,
    Failed,
}

#[derive(Fail, Debug)]
pub enum EventError {
    #[fail(display = "failed to get property '{}'", _0)]
    GettingProperty(String),
    #[fail(display = "unsupported type / event combination {}, {}", _0, _0)]
    UnsupportedTypeEvent(String, String),
    #[fail(display = "expected object")]
    ExpectedObject,
}

impl TestCaptures {
    /// Parses output lines from a complete libtest / suite test into
    /// a number of test captures.

    pub fn from_output<R: Read>(reader: R) -> Result<Self, failure::Error> {
        let mut reader = BufReader::new(reader);

        let num_tests = {
            match reader.read_line_as_event()? {
                Event::SuiteStart { test_count } => test_count,
                e => Err(TestResultsError::ExpectedSuiteStart(e))?,
            }
        };

        let mut captures = Vec::new();

        for i in 0..num_tests {
            let test_name = match reader.read_line_as_event()? {
                Event::TestStart { name } => name,
                e => Err(TestResultsError::ExpectedTestStart { index: i, event: e })?,
            };

            let output = &mut Vec::new();

            let result = loop {
                let line = reader.read_next_line()?;
                match parse_event(&line) {
                    Ok(Event::TestFinish {
                        ref name,
                        ref result,
                    }) if name == &test_name => {
                        break result.clone();
                    }
                    _ => {
                        output.push(line);
                    }
                }
            };

            captures.push(TestCapture {
                name: test_name,
                result,
                output: output.concat(),
            })
        }

        match reader.read_line_as_event()? {
            Event::SuiteFinish { .. } => {}
            e => Err(TestResultsError::ExpectedSuiteStart(e))?,
        }

        Ok(TestCaptures(captures))
    }
}

impl Event {
    pub fn from_json(value: &Value) -> Result<Event, failure::Error> {
        // TODO: how to reduce indent here?
        if let Value::Object(m) = value {
            let get_str = |property: &str| {
                m.get(property)
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| EventError::GettingProperty(property.to_owned()))
            };

            let get_usize = |property: &str| {
                m.get(property)
                    .and_then(|v| v.as_u64())
                    .and_then(|v| v.try_into().ok())
                    .ok_or_else(|| EventError::GettingProperty(property.to_owned()))
            };

            let run_finish = |r| {
                Ok(Event::SuiteFinish {
                    result: r,
                    passed: get_usize("passed")?,
                    failed: get_usize("failed")?,
                    allowed_fail: get_usize("allowed_fail")?,
                    ignored: get_usize("ignored")?,
                    measured: get_usize("measured")?,
                    filtered_out: get_usize("filtered_out")?,
                })
            };

            let test_finish = |r| {
                Ok(Event::TestFinish {
                    name: get_str("name")?.into(),
                    result: r,
                })
            };

            let event = get_str("event")?;
            let ty = get_str("type")?;

            match (ty, event) {
                ("suite", "started") => Ok(Event::SuiteStart {
                    test_count: get_usize("test_count")?,
                }),
                ("suite", "ok") => run_finish(OkOrFailed::Ok),
                ("suite", "failed") => run_finish(OkOrFailed::Failed),
                ("test", "started") => Ok(Event::TestStart {
                    name: get_str("name")?.into(),
                }),
                ("test", "ok") => test_finish(TestResult::Ok()),
                ("test", "failed") => {
                    let extra_data = {
                        if let Some(message) = m.get("message").and_then(|v| v.as_str()) {
                            Some(ExtraData::Message(message.into()))
                        } else if let Some(stdout) = m.get("stdout").and_then(|v| v.as_str()) {
                            Some(ExtraData::StdOut(stdout.into()))
                        } else {
                            None
                        }
                    };

                    test_finish(TestResult::Failed(extra_data))
                }
                ("test", "ignored") => test_finish(TestResult::Ignored),
                ("test", "allowed_fail") => test_finish(TestResult::AllowedFail),
                ("test", "timeout") => test_finish(TestResult::Timeout),
                (ty, event) => Err(EventError::UnsupportedTypeEvent(ty.into(), event.into()))?,
            }
        } else {
            Err(EventError::ExpectedObject)?
        }
    }
}

#[cfg(test)]
fn to_event(json: &str) -> Event {
    Event::from_json(&serde_json::from_str(json).unwrap()).unwrap()
}

#[test]
fn parse_run_start() {
    assert_eq!(
        to_event(r#"{ "type": "suite", "event": "started", "test_count": 3 }"#),
        Event::SuiteStart { test_count: 3 }
    );
}

#[test]
fn parse_run_finished() {
    assert_eq!(
        to_event(r#"{ "type": "suite", "event": "failed", "passed": 2, "failed": 1, "allowed_fail": 0, "ignored": 0, "measured": 0, "filtered_out": 0 }"#),
        Event::SuiteFinish {
            result: OkOrFailed::Failed,
            passed: 2,
            failed: 1,
            allowed_fail: 0,
            ignored: 0,
            measured: 0,
            filtered_out: 0,
        },
    );
}
#[test]
fn parse_test_start() {
    assert_eq!(
        to_event(r#"{ "type": "test", "event": "started", "name": "test_name" }"#),
        Event::TestStart {
            name: "test_name".to_string()
        }
    );
}

#[test]
fn parse_test_failed() {
    assert_eq!(
        to_event(r#"{ "type": "test", "name": "test_name", "event": "failed" }"#),
        Event::TestFinish {
            result: TestResult::Failed(None),
            name: "test_name".to_string()
        }
    );
}

#[test]
fn parse_test_ok() {
    assert_eq!(
        to_event(r#"{ "type": "test", "name": "test_name", "event": "ok" }"#),
        Event::TestFinish {
            result: TestResult::Ok(),
            name: "test_name".to_string()
        }
    );
}
