use crate::test_runner::{TestRunRequest, TestRunResult};
use clap::ArgMatches;
use crossbeam_channel;
use crossbeam_channel::Sender;
use std::sync::{atomic, mpsc, Arc};
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
    shutdown: Box<dyn FnOnce() + Send>,
}

impl Drop for TestWatcher {
    fn drop(&mut self) {
        debug!("shutting down the test watcher...");
        mem::replace(&mut self.shutdown, Box::new(|| ()))();
        debug!("test watcher down");
    }
}

/// Begin watching and running tests and send out test captures to the channel given.
/// Returns a Sender to re-trigger the testcase.
pub fn begin_watching(
    req: TestRunRequest,
    notifier: Sender<Notification>,
) -> Result<TestWatcher, failure::Error> {
    // parse arguments:
    let mut args = cargo_watch::get_options(false, &ArgMatches::default());
    args.paths.push(req.project_directory.clone());

    let full_path = fs::canonicalize(req.project_directory.clone()).unwrap();

    let (tx, rx) = mpsc::channel();
    let tx_shutdown = tx.clone();
    let shutdown_bool = Arc::new(atomic::AtomicBool::new(false));
    let shutdown_bool2 = shutdown_bool.clone();

    let thread = thread::spawn(move || {
        let watcher = TestWatcherHandler {
            _args: args.clone(),
            request: req.clone(),
            shutdown: shutdown_bool,
            notifier: notifier.clone(),
        };

        if let Err(e) = run::watch_with_handler(args, (tx, rx), watcher) {
            notifier
                .send(Notification::WatcherStopped(e.into()))
                // if sending did not work, noone knows that the watcher ended, so panic.
                .unwrap();
        }
    });

    let shutdown = move || {
        // indicate shutdown.
        shutdown_bool2.store(true, atomic::Ordering::SeqCst);
        // force an update.
        tx_shutdown
            .send(notify::RawEvent {
                path: Some(full_path),
                op: Ok(notify::Op::CHMOD),
                cookie: None,
            })
            .expect("failed to shutdown test watcher");
        // join the thread.
        thread.join().unwrap()
    };

    Ok(TestWatcher {
        shutdown: Box::new(shutdown),
    })
}

struct TestWatcherHandler {
    _args: Args,
    request: TestRunRequest,
    shutdown: Arc<atomic::AtomicBool>,
    notifier: Sender<Notification>,
}

impl TestWatcherHandler {
    fn capture_tests(&self) {
        let result = self.request.capture_tests();
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
