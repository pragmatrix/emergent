use crate::test_runner::{TestEnvironment, TestRunRequest, TestRunResult};
use clap::ArgMatches;
use crossbeam_channel;
use crossbeam_channel::Sender;
use std::path::PathBuf;
use std::sync::{atomic, mpsc, Arc, Mutex};
use std::{fs, mem, thread};
use watchexec::cli::Args;
use watchexec::pathop;
use watchexec::run;

#[derive(Debug)]
pub enum Notification {
    /// Watcher stopped because of an error.
    WatcherStopped(failure::Error),
    /// A test run has been completed.
    TestRunCompleted(Result<TestRunResult, failure::Error>),
}

pub struct TestWatcher {
    full_path: PathBuf,
    environment: Arc<Mutex<TestEnvironment>>,
    notify: mpsc::Sender<notify::RawEvent>,
    shutdown: Box<dyn FnOnce() + Send>,
}

impl Drop for TestWatcher {
    fn drop(&mut self) {
        debug!("shutting down the test watcher...");
        mem::replace(&mut self.shutdown, Box::new(|| ()))();
        debug!("test watcher down");
    }
}
impl TestWatcher {
    /// Begin watching and running tests and send out test captures to the channel given.
    /// Returns a Sender to re-trigger the testcase.
    pub fn begin_watching(
        req: TestRunRequest,
        environment: TestEnvironment,
        notifier: Sender<Notification>,
    ) -> Result<TestWatcher, failure::Error> {
        // parse arguments:
        let mut args = cargo_watch::get_options(false, &ArgMatches::default());
        args.paths.push(req.project_directory.clone());

        let full_path = fs::canonicalize(req.project_directory.clone()).unwrap();

        let (tx, rx) = mpsc::channel();
        let notify = tx.clone();
        let shutdown_bool = Arc::new(atomic::AtomicBool::new(false));
        let environment = Arc::new(Mutex::new(environment));

        let watcher = TestWatcherHandler {
            request: req,
            shutdown: shutdown_bool.clone(),
            notifier: notifier.clone(),
            environment: environment.clone(),
        };

        let thread = thread::spawn(move || {
            if let Err(e) = run::watch_with_handler(args, (tx, rx), watcher) {
                notifier
                    .send(Notification::WatcherStopped(e.into()))
                    // if sending did not work, noone knows that the watcher ended, so panic.
                    .unwrap();
            }
        });

        let shutdown = {
            let full_path = full_path.clone();
            let notify = notify.clone();
            move || {
                // indicate shutdown.
                shutdown_bool.store(true, atomic::Ordering::SeqCst);
                // force an update.
                notify
                    .send(notify::RawEvent {
                        path: Some(full_path),
                        op: Ok(notify::Op::CHMOD),
                        cookie: None,
                    })
                    .expect("failed to shutdown test watcher");
                // join the thread.
                thread.join().unwrap()
            }
        };

        Ok(TestWatcher {
            full_path,
            notify,
            environment,
            shutdown: Box::new(shutdown),
        })
    }

    pub fn update_environment(&mut self, environment: TestEnvironment) {
        *self.environment.lock().unwrap() = environment;
        self.notify
            .send(notify::RawEvent {
                path: Some(self.full_path.clone()),
                op: Ok(notify::Op::CHMOD),
                cookie: None,
            })
            .expect("failed to notify the test watcher");
    }
}

#[derive(Clone, Debug)]
struct TestWatcherHandler {
    request: TestRunRequest,
    shutdown: Arc<atomic::AtomicBool>,
    notifier: Sender<Notification>,
    environment: Arc<Mutex<TestEnvironment>>,
}

impl TestWatcherHandler {
    fn capture_tests(&self) {
        let environment = self.environment.lock().unwrap().clone();
        let result = self.request.capture_tests(environment);
        self.notifier
            .send(Notification::TestRunCompleted(result))
            .unwrap();
    }
}

impl run::Handler for TestWatcherHandler {
    fn new(_args: Args) -> watchexec::error::Result<Self>
    where
        Self: Sized,
    {
        panic!("internal error, unexpected TestWatcher::new() invocation");
    }

    fn on_manual(&mut self) -> watchexec::error::Result<bool> {
        self.capture_tests();
        Ok(true)
    }

    fn on_update(&mut self, _ops: &[pathop::PathOp]) -> watchexec::error::Result<bool> {
        if self.shutdown.load(atomic::Ordering::SeqCst) {
            return Ok(false);
        }
        self.capture_tests();
        Ok(true)
    }
}
