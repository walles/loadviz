use std::time::Instant;

use crate::cpuload::{diff, CpuLoad, LoadCounters};

static SECONDS_BETWEEN_MEASUREMENTS: u64 = 1;

struct LoadState {
    last_update_done: Instant,
    older_sample: Vec<LoadCounters>,
    newer_sample: Vec<LoadCounters>,
}

pub(crate) struct LoadReader {
    last_result: Vec<CpuLoad>,
    state: LoadState,
    get_load_counters: fn() -> Vec<LoadCounters>,

    /// Experiment hook: when true, every core is reported as fully loaded so
    /// the renderer always hits its worst-case path. Enabled by setting the
    /// LOADVIZ_FORCE_MAX environment variable at launch.
    force_max: bool,
}

impl LoadReader {
    pub(crate) fn new(get_load_counters: fn() -> Vec<LoadCounters>) -> LoadReader {
        let mut return_me = LoadReader {
            last_result: vec![],
            state: LoadState {
                last_update_done: Instant::now(),
                older_sample: vec![],
                newer_sample: vec![],
            },
            get_load_counters,
            force_max: std::env::var("LOADVIZ_FORCE_MAX").is_ok(),
        };

        return_me.measure_cpu_loads();

        return return_me;
    }

    pub(crate) fn get_loads(&mut self) -> Vec<CpuLoad> {
        if self.state.last_update_done.elapsed().as_secs() > SECONDS_BETWEEN_MEASUREMENTS {
            self.measure_cpu_loads();
        }

        let mut loads = self.last_result.clone();
        if !self.force_max {
            return loads;
        }

        for load in loads.iter_mut() {
            load.user_0_to_1 = 1.0;
            load.system_0_to_1 = 0.0;
        }

        return loads;
    }

    fn measure_cpu_loads(&mut self) {
        self.state.older_sample = self.state.newer_sample.clone();
        self.state.newer_sample = (self.get_load_counters)();

        self.last_result = diff(&self.state.older_sample, &self.state.newer_sample);
        self.state.last_update_done = Instant::now();
    }
}
