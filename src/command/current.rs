use std::string::ToString;

use structopt::StructOpt;

use notion_core::error::ErrorDetails;
use notion_core::session::{ActivityKind, Session};
use notion_fail::{throw, ExitCode, Fallible};

use crate::command::Command;

#[derive(StructOpt)]
pub(crate) struct Current {
    /// Display the current project's Node version
    #[structopt(short = "p", long = "project")]
    project: bool,

    /// Display the user's Node version
    #[structopt(short = "u", long = "user")]
    user: bool,
}

impl Command for Current {
    fn run(self, session: &mut Session) -> Fallible<ExitCode> {
        session.add_event_start(ActivityKind::Current);

        let result = match (self.project, self.user) {
            // both or neither => "all"
            (true, true) | (false, false) => {
                let project = project_node_version(&session)?;
                let user = user_node_version(&session)?;

                let user_active = project.is_none() && user.is_some();
                let any = project.is_some() || user.is_some();

                for version in project {
                    println!("project: v{} (active)", version);
                }

                for version in user {
                    println!(
                        "user: v{}{}",
                        version,
                        if user_active { " (active)" } else { "" }
                    );
                }

                any
            }

            // Only project set
            (true, false) => match project_node_version(&session)? {
                Some(version) => {
                    println!("v{}", version);
                    true
                }
                None => false,
            },

            // Only user set
            (false, true) => match user_node_version(&session)? {
                Some(version) => {
                    println!("v{}", version);
                    true
                }
                None => false,
            },
        };

        session.add_event_end(ActivityKind::Current, ExitCode::Success);

        if !result {
            throw!(ErrorDetails::NoVersionsFound)
        }

        Ok(ExitCode::Success)
    }
}

fn project_node_version(session: &Session) -> Fallible<Option<String>> {
    Ok(session
        .project_platform()?
        .map(|platform| platform.node_runtime.to_string()))
}

fn user_node_version(session: &Session) -> Fallible<Option<String>> {
    Ok(session
        .user_platform()?
        .map(|platform| platform.node_runtime.to_string()))
}
