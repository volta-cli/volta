use std::path::PathBuf;

/// Represents an Empty (or uninitialized) Volta layout, one that has never been used by any prior version
///
/// This is the easiest to migrate from, as we simply need to create the current layout within the .volta
/// directory
pub struct Empty {
    pub home: PathBuf,
}

impl Empty {
    pub fn new(home: PathBuf) -> Self {
        Empty { home }
    }
}
