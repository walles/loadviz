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

    return load_counters;
}

#[cfg(target_os = "linux")]
pub fn get_load_counters() -> Vec<LoadCounters> {
    todo!();
}
