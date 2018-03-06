use std::string::ToString;

use notion_core::session::Session;
use notion_fail::Fallible;

use ::Notion;
use command::{Command, CommandName, Help};

#[derive(Debug, Deserialize)]
pub(crate) struct Args {
    flag_local: bool,
    flag_global: bool
}

pub(crate) enum Current {
    Help,
    Local,
    Global,
    All
}

impl Command for Current {
    type Args = Args;

    const USAGE: &'static str = "
Display the currently activated toolchain

Usage:
    notion current [options]

Options:
    -h, --help     Display this message
    -l, --local    Display local toolchain
    -g, --global   Display global toolchain
";

    fn help() -> Self { Current::Help }

    fn parse(_: Notion, Args { flag_local, flag_global }: Args) -> Fallible<Current> {
        Ok(if !flag_local && flag_global {
            Current::Local
        } else if flag_local && !flag_global {
            Current::Global
        } else {
            Current::All
        })
    }

    fn run(self) -> Fallible<bool> {
        let session = Session::new()?;

        match self {
            Current::Help => {
                Help::Command(CommandName::Current).run()
            }
            Current::Local => {
                Ok(local(&session)?
                    .map(|version| { println!("v{}", version); })
                    .is_some())
            }
            Current::Global => {
                Ok(global(&session)?
                    .map(|version| { println!("v{}", version); })
                    .is_some())
            }
            Current::All => {
                let (local, global) = (local(&session)?, global(&session)?);
                let global_active = local.is_none() && global.is_some();
                let any = local.is_some() || global.is_some();
                for version in local {
                    println!("local: v{} (active)", version);
                }
                for version in global {
                    println!("global: v{}{}", version, if global_active { " (active)" } else { "" });
                }
                Ok(any)
            }
        }
    }

}

fn local(session: &Session) -> Fallible<Option<String>> {
    let project = session.project();
    let project = match project {
        Some(ref project) => project,
        None => { return Ok(None); }
    };

    let req = &project.manifest().node;
    let catalog = session.catalog()?;
    Ok(catalog.node.resolve_local(&req).map(|v| v.to_string()))
}

fn global(session: &Session) -> Fallible<Option<String>> {
    let catalog = session.catalog()?;
    Ok(catalog.node.activated.clone().map(|v| v.to_string()))
}
