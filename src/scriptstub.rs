extern crate nodeup_core;

use nodeup_core::stub;

fn main() {
    let path_var = stub::prepare();
    let status = stub::script(&path_var)
        .status()
        .unwrap();
    println!("process exited with {}", status);
    // FIXME: exit with the same status code
}
