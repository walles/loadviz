use std::time::{Duration, Instant};

use crate::LoadViz;

// Maybe keep this higher than SECONDS_BETWEEN_MEASUREMENTS in load_reader.rs?
// By moving both values around until we're happy!
//
// FIXME: I think the maths is wrong, I don't think it takes 20 seconds.
static SECONDS_0_TO_100: f32 = 20.0;

impl LoadViz {
    pub(crate) fn update_currently_displayed_loads(&mut self) {
        self.update_currently_displayed_loads_internal();
        self.currently_displayed_loads_updated = Instant::now();
    }

    fn update_currently_displayed_loads_internal(&mut self) {
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
    let how_far_we_can_go = dt.as_secs_f32() / SECONDS_0_TO_100;

    let how_far_we_are_allowed_to_go = (goal - current).abs();
    let direction = if goal > current { 1.0 } else { -1.0 };

    return how_far_we_can_go.min(how_far_we_are_allowed_to_go) * direction;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_step() {
        let dt = Duration::from_secs_f32(SECONDS_0_TO_100 / 2.0);

        assert_eq!(compute_step(dt, 0.0, 1.0), 0.5);
        assert_eq!(compute_step(dt, 0.0, 0.1), 0.1);

        assert_eq!(compute_step(dt, 1.0, 0.0), -0.5);
        assert_eq!(compute_step(dt, 1.0, 0.7), -0.3);
    }
}
