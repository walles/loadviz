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
        };

        return_me.measure_cpu_loads();

        return return_me;
    }

    pub(crate) fn get_loads(&mut self) -> Vec<CpuLoad> {
        if self.state.last_update_done.elapsed().as_secs() > SECONDS_BETWEEN_MEASUREMENTS {
            self.measure_cpu_loads();
        }

        return self.last_result.clone();
    }

    fn measure_cpu_loads(&mut self) {
        self.state.older_sample = self.state.newer_sample.clone();
        self.state.newer_sample = (self.get_load_counters)();

        self.last_result = diff(&self.state.older_sample, &self.state.newer_sample);
        self.state.last_update_done = Instant::now();
    }
}
