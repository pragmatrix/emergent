use crate::test_runner::TestRunRequest;
use clap::ArgMatches;
use watchexec::cli::Args;
use watchexec::pathop;
use watchexec::run;

/// Asynchronous test watcher / runner.

pub fn watch(req: &TestRunRequest) -> Result<(), failure::Error> {
    // ignoring the directory for now.
    // parse arguments:

    let mut args = cargo_watch::get_options(false, &ArgMatches::default());
    args.paths.push(req.project_directory.clone());
    Ok(run::watch::<TestWatcher>(args)?)
}

struct TestWatcher {}

impl run::Handler for TestWatcher {
    fn new(args: Args) -> watchexec::error::Result<Self>
    where
        Self: Sized,
    {
        dbg!("watcher initialized");
        Ok(TestWatcher {})
    }

    fn on_manual(&mut self) -> watchexec::error::Result<bool> {
        dbg!("on manual");
        Ok(true)
    }

    fn on_update(&mut self, ops: &[pathop::PathOp]) -> watchexec::error::Result<bool> {
        dbg!("on update");

        Ok(true)
    }
}
