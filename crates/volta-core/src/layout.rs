use std::env;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use crate::error::ErrorDetails;
use volta_layout::v1::VoltaLayout;

use ref_thread_local::{ref_thread_local, RefThreadLocal};

ref_thread_local! {
    static managed LAYOUT: Result<Rc<VoltaLayout>, ErrorDetails> = {
        volta_home().map(|home| {
            // ISSUE (#333): on Windows, this should be %ProgramFiles%\Notion
            let install = home.clone();
            Rc::new(VoltaLayout::new(install, home))
        })
    };
}

fn volta_home() -> Result<PathBuf, ErrorDetails> {
    if let Some(home) = env::var_os("VOLTA_HOME") {
        Ok(Path::new(&home).to_path_buf())
    } else if cfg!(target_os = "windows") {
        let home = dirs::data_local_dir().ok_or(ErrorDetails::NoLocalDataDir)?;
        Ok(home.join("Volta"))
    } else {
        let home = dirs::home_dir().ok_or(ErrorDetails::NoHomeEnvironmentVar)?;
        Ok(home.join(".volta"))
    }
}

pub fn layout() -> Result<Rc<VoltaLayout>, ErrorDetails> {
    match *LAYOUT.borrow() {
        Ok(ref path) => Ok(path.clone()),
        Err(ref err) => Err(err.clone()),
    }
}
