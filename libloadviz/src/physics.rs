use std::time::Duration;

use crate::cpuload::CpuLoad;

// Maybe keep these higher than SECONDS_BETWEEN_MEASUREMENTS in load_reader.rs?
// By moving the values around until we're happy!
//
// The idea with moving up fast is that we want to react when something happens.
//
// The idea with moving down slow is that we want to have some brief history.
static SECONDS_0_TO_100_UP: f32 = 5.0;
static SECONDS_0_TO_100_DOWN: f32 = 20.0;

pub(crate) fn update_currently_displayed_loads(
    current: &mut Vec<CpuLoad>,
    target: &Vec<CpuLoad>,
    dt: Duration,
) {
    if current.len() != target.len() {
        // current = target;
        current.clone_from(target);
        return;
    }

    // Sort both arrays so we compare the right heights with each other
    //
    // Sorts are the same as in mirror_sort() in renderer.rs
    current.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let mut cpu_loads = target.clone();
    cpu_loads.sort_by(|a, b| a.partial_cmp(b).unwrap());

    for (current, actual) in current.iter_mut().zip(cpu_loads.iter_mut()) {
        current.user_0_to_1 += compute_step(dt, current.user_0_to_1, actual.user_0_to_1);

        current.system_0_to_1 += compute_step(dt, current.system_0_to_1, actual.system_0_to_1);
    }
}

/// How far should we step towards the goal value?
///
/// `dt` is the time since the last update
fn compute_step(dt: Duration, current: f32, goal: f32) -> f32 {
    let direction = if goal > current { 1.0 } else { -1.0 };

    let how_far_we_can_go = dt.as_secs_f32()
        / (if direction > 0.0 {
            SECONDS_0_TO_100_UP
        } else {
            SECONDS_0_TO_100_DOWN
        });

    let how_far_we_are_allowed_to_go = (goal - current).abs();

    return how_far_we_can_go.min(how_far_we_are_allowed_to_go) * direction;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_step() {
        let dt = Duration::from_secs_f32(SECONDS_0_TO_100_UP / 2.0);
        assert_eq!(compute_step(dt, 0.0, 1.0), 0.5);
        assert_eq!(compute_step(dt, 0.0, 0.1), 0.1);

        let dt = Duration::from_secs_f32(SECONDS_0_TO_100_DOWN / 2.0);
        assert_eq!(compute_step(dt, 1.0, 0.0), -0.5);
        assert_eq!(compute_step(dt, 1.0, 0.7), -0.3);
    }
}
