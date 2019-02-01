// Rust doesn't allow using keywords as module names so we have to call this `use_`.
// With https://github.com/rust-lang/rfcs/blob/master/text/2151-raw-identifiers.md we
// could consider something like `r#use` instead.

use notion_core::error::ErrorDetails;
use notion_core::session::Session;
use notion_fail::Fallible;

use command::Command;
use Notion;

#[derive(Debug, Deserialize)]
pub(crate) struct Args;

pub(crate) enum Use {
    Help,
    Use,
}

impl Command for Use {
    type Args = Args;

    const USAGE: &'static str = "
To install a tool in your user toolchain, use 'notion install'
To pin a tool in a project toolchain, use 'notion pin'

See 'notion --help' for more information.
";

    fn help() -> Self {
        Use::Help
    }

    fn parse(_: Notion, _: Args) -> Fallible<Self> {
        Ok(Use::Use)
    }

    fn run(self, _: &mut Session) -> Fallible<()> {
        Ok(())
    }

    fn go(_: Notion, _: &mut Session) -> Fallible<()> {
        throw!(ErrorDetails::CliParseError {
            usage: None,
            error: format!("no such command: `use`\n{}", Use::USAGE)
        });
    }
}
