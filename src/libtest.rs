// https://github.com/rust-lang/libtest/blob/master/libtest/formatters/json.rs

use serde_json::Value;
use std::convert::TryInto;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum OkOrFailed {
    Ok,
    Failed,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum ExtraData {
    Message(String),
    StdOut(String),
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum TestResult {
    Ok,
    Failed(Option<ExtraData>),
    Ignored,
    AllowedFail,
    // TODO: Bench
    Timeout,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Event {
    RunStart {
        test_count: usize,
    },
    RunFinish {
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

impl Event {
    pub fn from_json(value: &Value) -> Result<Event, String> {
        // TODO: how to reduce indent here?
        if let Value::Object(m) = value {
            let get_str = |property| {
                m.get(property)
                    .and_then(|v| v.as_str())
                    .expect(&format!("expect string property '{}'", property))
            };

            let get_usize = |property| {
                m.get(property)
                    .and_then(|v| v.as_u64())
                    .expect(&format!("expect number property '{}'", property))
                    .try_into()
                    .unwrap()
            };

            let run_finish = |r| Event::RunFinish {
                result: r,
                passed: get_usize("passed"),
                failed: get_usize("failed"),
                allowed_fail: get_usize("allowed_fail"),
                ignored: get_usize("ignored"),
                measured: get_usize("measured"),
                filtered_out: get_usize("filtered_out"),
            };

            let test_finish = |r| Event::TestFinish {
                name: get_str("name").into(),
                result: r,
            };

            let event = get_str("event");
            let ty = get_str("type");

            match (ty, event) {
                ("suite", "started") => Ok(Event::RunStart {
                    test_count: get_usize("test_count"),
                }),
                ("suite", "ok") => Ok(run_finish(OkOrFailed::Ok)),
                ("suite", "failed") => Ok(run_finish(OkOrFailed::Failed)),
                ("test", "started") => Ok(Event::TestStart {
                    name: get_str("name").into(),
                }),
                ("test", "ok") => Ok(test_finish(TestResult::Ok)),
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

                    Ok(test_finish(TestResult::Failed(extra_data)))
                }
                ("test", "ignored") => Ok(test_finish(TestResult::Ignored)),
                ("test", "allowed_fail") => Ok(test_finish(TestResult::AllowedFail)),
                ("test", "timeout") => Ok(test_finish(TestResult::Timeout)),
                (ty, event) => Result::Err(format!(
                    "unsupported combination of type '{}' and event '{}'",
                    ty, event
                )),
            }
        } else {
            Result::Err("expected an object".into())
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
        Event::RunStart { test_count: 3 }
    );
}

#[test]
fn parse_run_fninished() {
    assert_eq!(
        to_event(r#"{ "type": "suite", "event": "failed", "passed": 2, "failed": 1, "allowed_fail": 0, "ignored": 0, "measured": 0, "filtered_out": 0 }"#),
        Event::RunFinish {
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
            result: TestResult::Ok,
            name: "test_name".to_string()
        }
    );
}
