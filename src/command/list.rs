use structopt::StructOpt;

use crate::command::Command;
use volta_core::session::Session;
use volta_fail::{Fallible, ExitCode};

#[derive(StructOpt)]
pub(crate) struct List {}

impl Command for List {
    fn run(self, session: &mut Session) -> Fallible<ExitCode> {
        unimplemented!()
    }
}
