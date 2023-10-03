//! This binary is here to provoke memory leaks in getting the load counters

use std::process;

use libloadviz::system_load::get_load_counters;

fn main() {
    println!("PID {} now sleeping for 10s", process::id());
    std::thread::sleep(std::time::Duration::from_secs(10));
    println!("Now getting load counters");
    loop {
        get_load_counters();
    }
}
