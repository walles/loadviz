use std::time::{Duration, Instant};

use crate::LoadViz;

// Maybe keep this higher than SECONDS_BETWEEN_MEASUREMENTS in load_reader.rs?
// By moving both values around until we're happy!
static SECONDS_TO_100_PERCENT: f32 = 40.0;

impl LoadViz {
    pub(crate) fn update_currently_displayed_loads(&mut self) {
        if self.currently_displayed_loads.len() != self.load_reader.get_loads().len() {
            self.currently_displayed_loads = self.load_reader.get_loads();
            return;
        }

        // Sort both arrays so we compare the right heights with each other
        //
        // Sorts are the same as in mirror_sort() in renderer.rs
        self.currently_displayed_loads
            .sort_by(|a, b| a.partial_cmp(b).unwrap());
        let mut cpu_loads = self.load_reader.get_loads();
        cpu_loads.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let dt = Instant::now().duration_since(self.currently_displayed_loads_updated);
        for (current, actual) in self
            .currently_displayed_loads
            .iter_mut()
            .zip(cpu_loads.iter_mut())
        {
            current.user_0_to_1 += compute_step(dt, current.user_0_to_1, actual.user_0_to_1);

            current.system_0_to_1 += compute_step(dt, current.system_0_to_1, actual.system_0_to_1);
        }
    }
}

/// How far should we step towards the goal value?
///
/// `dt` is the time since the last update
fn compute_step(dt: Duration, current: f32, goal: f32) -> f32 {
    let max_step = dt.as_secs_f32() / SECONDS_TO_100_PERCENT;

    let max_diff = (goal - current).abs();
    let direction = if goal > current { 1.0 } else { -1.0 };

    return max_step.min(max_diff) * direction;
}
