use std::cell::Cell;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Once;

static SMOKE_TEST_DIR: &str = "smoke_test";
static NEXT_ID: AtomicUsize = AtomicUsize::new(0);

thread_local!(static TASK_ID: usize = NEXT_ID.fetch_add(1, Ordering::SeqCst));

// creates the root directory for the tests (once), and
// initializes the root and home directories for the current task
fn init() {
    static GLOBAL_INIT: Once = Once::new();
    thread_local!(static LOCAL_INIT: Cell<bool> = Cell::new(false));
    GLOBAL_INIT.call_once(|| {
        global_root().mkdir_p();
    });
    LOCAL_INIT.with(|i| {
        if i.get() {
            return;
        }
        i.set(true);
        root().rm_rf();
        home().mkdir_p();
    })
}

// the root directory for the smoke tests, in `target/smoke_test`
fn global_root() -> PathBuf {
    let mut path = ok_or_panic! { env::current_exe() };
    path.pop(); // chop off exe name
    path.pop(); // chop off 'debug'

    // If `cargo test` is run manually then our path looks like
    // `target/debug/foo`, in which case our `path` is already pointing at
    // `target`. If, however, `cargo test --target $target` is used then the
    // output is `target/$target/debug/foo`, so our path is pointing at
    // `target/$target`. Here we conditionally pop the `$target` name.
    if path.file_name().and_then(|s| s.to_str()) != Some("target") {
        path.pop();
    }

    path.join(SMOKE_TEST_DIR)
}

pub fn root() -> PathBuf {
    init();
    global_root().join(TASK_ID.with(|my_id| format!("t{}", my_id)))
}

pub fn home() -> PathBuf {
    root().join("home")
}

enum Remove {
    File,
    Dir,
}
impl Remove {
    fn to_str(&self) -> &'static str {
        match *self {
            Remove::File => "remove file",
            Remove::Dir => "remove dir",
        }
    }

    fn at(&self, path: &Path) {
        if cfg!(windows) {
            let mut p = ok_or_panic!(path.metadata()).permissions();
            // This lint rule is not applicable: this is in a `cfg!(windows)` block.
            #[allow(clippy::permissions_set_readonly_false)]
            p.set_readonly(false);
            ok_or_panic! { fs::set_permissions(path, p) };
        }
        match *self {
            Remove::File => fs::remove_file(path),
            Remove::Dir => fs::remove_dir_all(path), // ensure all dir contents are removed
        }
        .unwrap_or_else(|e| {
            panic!("failed to {} {}: {}", self.to_str(), path.display(), e);
        })
    }
}

pub trait PathExt {
    fn rm(&self);
    fn rm_rf(&self);
    fn rm_contents(&self);
    fn ensure_empty(&self);
    fn mkdir_p(&self);
}

impl PathExt for Path {
    // delete a file if it exists
    fn rm(&self) {
        if !self.exists() {
            return;
        }
        // On windows we can't remove a readonly file, and git will
        // often clone files as readonly. As a result, we have some
        // special logic to remove readonly files on windows.
        Remove::File.at(self);
    }

    /* Technically there is a potential race condition, but we don't
     * care all that much for our tests
     */
    fn rm_rf(&self) {
        if !self.exists() {
            return;
        }
        self.rm_contents();
        Remove::Dir.at(self);
    }

    // remove directory contents but not the directory itself
    fn rm_contents(&self) {
        for file in ok_or_panic! { fs::read_dir(self) } {
            let file = ok_or_panic! { file };
            if file.file_type().map(|m| m.is_dir()).unwrap_or(false) {
                file.path().rm_rf();
            } else {
                file.path().rm();
            }
        }
    }

    // ensure the directory is created and empty
    fn ensure_empty(&self) {
        self.mkdir_p();
        self.rm_contents();
    }

    // create all paths up to the input path
    fn mkdir_p(&self) {
        fs::create_dir_all(self)
            .unwrap_or_else(|e| panic!("failed to mkdir_p {}: {}", self.display(), e))
    }
}
