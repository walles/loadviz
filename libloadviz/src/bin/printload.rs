// Prints a snapshot of the current load of each CPU

use std::thread;

use libloadviz::system_load_macos::get_load_counters;

fn main() {
    let first = get_load_counters();
    thread::sleep(std::time::Duration::from_secs(1));
    let second = get_load_counters();

    let diff = libloadviz::cpuload::diff(&first, &second);
    println!("{:#?}", diff);
}
