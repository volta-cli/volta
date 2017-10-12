extern crate nodeup_core;

use nodeup_core::launch;

fn main() {
    let path_var = launch::prepare();
    let status = launch::script(&path_var)
        .status()
        .unwrap();
    println!("process exited with {}", status);
    // FIXME: exit with the same status code
}
