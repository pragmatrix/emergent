use crate::libtest::TestCaptures;
use crate::test_runner::TestRunRequest;
use clap::ArgMatches;
use crossbeam_channel;
use crossbeam_channel::Sender;
use std::thread;
use watchexec::cli::Args;
use watchexec::pathop;
use watchexec::run;

#[derive(Debug)]
pub enum Notification {
    /// Stopped
    WatcherStopped(Result<(), failure::Error>),
    /// A test run has been completed.
    TestRunCompleted(Result<TestCaptures, failure::Error>),
}

/// Begin watching and running tests and send out test captures to the channel given.
/// The asynchronous thread cannot be interrupted (yet).
pub fn begin_watching(
    req: TestRunRequest,
    notifier: Sender<Notification>,
) -> Result<(), failure::Error> {
    // parse arguments:
    let mut args = cargo_watch::get_options(false, &ArgMatches::default());
    args.paths.push(req.project_directory.clone());

    thread::spawn(move || {
        let watcher = TestWatcher {
            args: args.clone(),
            request: req.clone(),
            notifier: notifier.clone(),
        };

        let r = run::watch_with_handler(args, watcher);
        notifier
            .send(Notification::WatcherStopped(r.map_err(|e| e.into())))
            // if sending does not work, noone knows that the watcher ended, so panic.
            .unwrap();
    });

    // note: errors can not happen ATM, but we may decide to return one in the future.
    Ok(())
}

struct TestWatcher {
    args: Args,
    request: TestRunRequest,
    notifier: Sender<Notification>,
}

impl TestWatcher {
    fn capture_tests(&self) {
        let result = self.request.capture_tests();
        self.notifier
            .send(Notification::TestRunCompleted(result))
            .unwrap();
    }
}

impl run::Handler for TestWatcher {
    fn new(args: Args) -> watchexec::error::Result<Self>
    where
        Self: Sized,
    {
        panic!("internal error, unexpected TestWatcher::new() invocation");
    }

    fn on_manual(&mut self) -> watchexec::error::Result<bool> {
        self.capture_tests();
        Ok(true)
    }

    fn on_update(&mut self, ops: &[pathop::PathOp]) -> watchexec::error::Result<bool> {
        self.capture_tests();
        Ok(true)
    }
}
