use std::env::{args_os, ArgsOs};
use std::ffi::OsStr;
use std::process::{Command, exit};
use std::path::Path;
use std::cell::{RefCell, Ref};
use std::marker::Sized;

use project::Project;
use global::{self, State};
use version::Version;
use install;
use env;
use failure;
use config::{self, Config};
use style;

pub enum Location {
    Global(State),
    Local(Project)
}

impl Location {

    pub fn current() -> Result<Location, failure::Error> {
        Ok(if let Some(project) = Project::for_current_dir()? {
            Location::Local(project)
        } else {
            Location::Global(global::state()?)
        })
    }

    pub fn version(&self) -> Result<Option<String>, failure::Error> {
        match self {
            &Location::Global(State { node: None }) => {
                Ok(None)
            }
            &Location::Global(State { node: Some(Version::Public(ref version))}) => {
                Ok(Some(version.clone()))
            }
            &Location::Local(ref project) => {
                Ok(Some(project.lockfile()?.node.version.clone()))
            }
        }
    }

}

pub trait Tool: Sized {
    fn new(exe: &OsStr, args: ArgsOs, path_var: &OsStr) -> Self;

    fn command(self) -> Command;

    fn launch(self) -> ! {
        let mut command = self.command();
        let status = command.status();
        match status {
            Ok(status) if status.success() => {
                exit(0);
            }
            Ok(status) => {
                // FIXME: if None, in unix, find out the signal
                exit(status.code().unwrap_or(1));
            }
            Err(err) => {
                style::display_error(err);
                exit(1);
            }
        }
    }
}

pub struct Script(Command);

pub struct Binary(Command);

#[cfg(windows)]
impl Tool for Script {
    fn new(exe: &OsStr, args: ArgsOs, path_var: &OsStr) -> Self {
        // See: https://github.com/rust-lang/rust/issues/42791
        let mut command = Command::new("cmd.exe");
        command.arg("/C");
        command.arg(exe);
        command.args(args);
        command.env("PATH", path_var);
        Script(command)
    }

    fn command(self) -> Command {
        let Script(command) = self;
        command
    }
}

fn command_for(exe: &OsStr, args: ArgsOs, path_var: &OsStr) -> Command {
    // FIXME: at least in unix, use exec instead
    let mut command = Command::new(exe);
    command.args(args);
    command.env("PATH", path_var);
    command
}

#[cfg(not(windows))]
impl Tool for Script {
    fn new(exe: &OsStr, args: ArgsOs, path_var: &OsStr) -> Self {
        Script(command_for(exe, args, path_var))
    }

    fn command(self) -> Command {
        let Script(command) = self;
        command
    }
}

impl Tool for Binary {
    fn new(exe: &OsStr, args: ArgsOs, path_var: &OsStr) -> Self {
        Binary(command_for(exe, args, path_var))
    }

    fn command(self) -> Command {
        let Binary(command) = self;
        command
    }
}

pub struct Context {
    config: RefCell<Option<Config>>,
    location: Location
}

impl Context {

    pub fn new() -> Result<Context, failure::Error> {
        let location = Location::current()?;
        Ok(Context {
            config: RefCell::new(None),
            location: location
        })
    }

    pub fn config(&self) -> Result<Ref<Config>, failure::Error> {
        // Create a new scope to contain the lifetime of the dynamic borrow.
        {
            let cfg: Ref<Option<Config>> = self.config.borrow();
            if cfg.is_some() {
                return Ok(Ref::map(cfg, |opt| opt.as_ref().unwrap()));
            }
        }

        // Create a new scope to contain the lifetime of the dynamic borrow.
        {
            let mut cfg = self.config.borrow_mut();
            *cfg = Some(config::config()?);
        }

        // Now try again recursively, outside the scope of the previous borrows.
        self.config()
    }

}

fn prepare<T: Tool>() -> Result<T, failure::Error> {
    let context = Context::new()?;
    let mut args = args_os();
    // FIXME: make an error kind for this case
    let exe = Path::new(&args.next().unwrap()).file_name().unwrap().to_os_string();
    // FIXME: make an error kind for this case
    let version = context.location.version()?.unwrap();
    install::by_version(&version)?;
    let path_var = env::path_for(&version);
    Ok(T::new(&exe, args, &path_var))
}

pub fn launch<T: Tool>() -> ! {
    match prepare::<T>() {
        Ok(tool) => tool.launch(),
        Err(e) => {
            style::display_error(e);
            exit(1);
        }
    }
}
