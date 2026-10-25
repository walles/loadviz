use crate::cpuload::LoadCounters;

#[cfg(target_os = "macos")]
pub fn get_load_counters() -> Vec<LoadCounters> {
    let port = unsafe { libc::mach_host_self() };

    let mut num_cpu_u = 0u32;
    let mut cpu_info: *mut i32 = std::ptr::null_mut();
    let mut num_cpu_info = 0u32;
    let errno = unsafe {
        libc::host_processor_info(
            port,
            libc::PROCESSOR_CPU_LOAD_INFO,
            &mut num_cpu_u as *mut u32,
            &mut cpu_info as *mut *mut i32,
            &mut num_cpu_info as *mut u32,
        )
    };

    if errno != libc::KERN_SUCCESS {
        panic!("host_processor_info failed: {}", errno);
    }

    let mut load_counters: Vec<LoadCounters> = vec![];
    for i in 0..num_cpu_u {
        let cpu_info = unsafe { cpu_info.offset((libc::CPU_STATE_MAX * (i as i32)) as isize) };
        let nice = unsafe { *cpu_info.offset(libc::CPU_STATE_NICE as isize) as usize };
        let user = unsafe { *cpu_info.offset(libc::CPU_STATE_USER as isize) as usize };
        let system = unsafe { *cpu_info.offset(libc::CPU_STATE_SYSTEM as isize) as usize };
        let idle = unsafe { *cpu_info.offset(libc::CPU_STATE_IDLE as isize) as usize };
        load_counters.push(LoadCounters {
            user: user + nice,
            system,
            idle,
        });
    }

    unsafe {
        libc::vm_deallocate(
            libc::mach_task_self(),
            cpu_info as libc::vm_address_t,
            num_cpu_info.try_into().unwrap(),
        );
    }

    return load_counters;
}

#[cfg(target_os = "linux")]
pub fn get_load_counters() -> Vec<LoadCounters> {
    todo!();
}

/// Parse the contents of `/proc/stat` into one `LoadCounters` per CPU.
#[cfg(target_os = "linux")]
fn parse_proc_stat(_proc_stat: &str) -> Vec<LoadCounters> {
    todo!();
}

#[cfg(test)]
mod tests {
    use super::get_load_counters;

    #[test]
    fn test_get_load_counters() {
        let older = get_load_counters();
        let newer = get_load_counters();

        let num_cpus = unsafe { libc::sysconf(libc::_SC_NPROCESSORS_ONLN) };
        assert_eq!(num_cpus as usize, older.len());
        assert_eq!(older.len(), newer.len());

        for (older, newer) in older.iter().zip(newer.iter()) {
            assert!(newer.user >= older.user);
            assert!(newer.system >= older.system);
            assert!(newer.idle >= older.idle);
        }

        let sum: usize = newer.iter().map(|c| c.user + c.system + c.idle).sum();
        assert!(sum > 0);
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn test_parse_proc_stat() {
        // Columns: user nice system idle iowait irq softirq steal guest guest_nice
        let proc_stat = "\
cpu  1111 2222 3333 4444 5555 6666 7777 8888 9999 1010
cpu0 1 2 3 4 5 6 7 8 9 10
cpu1 100 200 300 400 500 600 700 800 900 1000
intr 12345 0 0 0
ctxt 67890
btime 1700000000
processes 4242
procs_running 1
procs_blocked 0
softirq 1 2 3 4 5 6 7 8 9 10 11
";
        let counters = super::parse_proc_stat(proc_stat);
        assert_eq!(2, counters.len());

        assert_eq!(1 + 2, counters[0].user);
        assert_eq!(3 + 5 + 6 + 7 + 8, counters[0].system);
        assert_eq!(4, counters[0].idle);

        assert_eq!(100 + 200, counters[1].user);
        assert_eq!(300 + 500 + 600 + 700 + 800, counters[1].system);
        assert_eq!(400, counters[1].idle);
    }
}
