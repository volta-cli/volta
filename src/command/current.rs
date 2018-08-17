use std::string::ToString;

use notion_core::session::{ActivityKind, Session};
use notion_fail::Fallible;

use Notion;
use command::{Command, CommandName, Help};

#[derive(Debug, Deserialize)]
pub(crate) struct Args {
    flag_project: bool,
    flag_user: bool,
}

pub(crate) enum Current {
    Help,
    Project,
    User,
    All,
}

impl Command for Current {
    type Args = Args;

    const USAGE: &'static str = "
Display the currently activated Node version

Usage:
    notion current [options]

Options:
    -h, --help     Display this message
    -p, --project  Display the current project's Node version
    -u, --user     Display the user's Node version
";

    fn help() -> Self {
        Current::Help
    }

    fn parse(
        _: Notion,
        Args {
            flag_project,
            flag_user,
        }: Args,
    ) -> Fallible<Current> {
        Ok(if !flag_project && flag_user {
            Current::User
        } else if flag_project && !flag_user {
            Current::Project
        } else {
            Current::All
        })
    }

    fn run(self, session: &mut Session) -> Fallible<bool> {
        session.add_event_start(ActivityKind::Current);

        let result = match self {
            Current::Help => Help::Command(CommandName::Current).run(session),
            Current::Project => Ok(project_node_version(&session)?
                .map(|version| {
                    println!("v{}", version);
                })
                .is_some()),
            Current::User => Ok(user_node_version(session)?
                .map(|version| {
                    println!("v{}", version);
                })
                .is_some()),
            Current::All => {
                let (project, user) = (
                    project_node_version(&session)?,
                    user_node_version(&session)?,
                );

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

                Ok(any)
            }
        };
        session.add_event_end(ActivityKind::Current, 0);
        result
    }
}

fn project_node_version(session: &Session) -> Fallible<Option<String>> {
    if session.in_pinned_project() {
        let project = session.node_project().unwrap();
        let req = &project.manifest().node();
        let catalog = session.catalog()?;
        return Ok(catalog.node.resolve_local(&req).map(|v| v.to_string()));
    }
    Ok(None)
}

fn user_node_version(session: &Session) -> Fallible<Option<String>> {
    Ok(session.user_node()?.clone().map(|v| v.to_string()))
}
