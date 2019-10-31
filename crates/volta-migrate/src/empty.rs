use std::path::PathBuf;

pub struct Empty {
    pub home: PathBuf,
}

impl Empty {
    pub fn new(home: PathBuf) -> Self {
        Empty { home }
    }
}
