// Prints a snapshot of the load counters for each CPU

use libloadviz::system_load_macos::get_load_counters;

fn main() {
    println!("{:#?}", get_load_counters());
}
