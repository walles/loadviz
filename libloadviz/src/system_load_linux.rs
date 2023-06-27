#![cfg(target_os = "linux")]

use crate::cpuload::LoadCounters;

pub fn get_load_counters() -> Vec<LoadCounters> {
    // See system_load_macos.rs for inspiration
    todo!();
}
