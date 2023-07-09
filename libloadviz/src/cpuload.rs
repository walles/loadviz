use std::cmp::Ordering;

#[derive(PartialEq, Copy, Clone, Debug)]
pub struct CpuLoad {
    // NOTE: Maybe we should do "nice" here as well?
    pub user_0_to_1: f32,
    pub system_0_to_1: f32,
}

#[derive(Debug, Clone)]
pub struct LoadCounters {
    pub user: usize,
    pub system: usize,

    /// CPU time not used for anything
    pub idle: usize,
    //
    // NOTE: Maybe track nice in here as well?
}

impl PartialOrd for CpuLoad {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        return (self.user_0_to_1 + self.system_0_to_1)
            .partial_cmp(&(other.user_0_to_1 + other.system_0_to_1));
    }
}

/// Based on two CPU counter snapshots, compute the load for each CPU.
///
/// If the number of cores have changed, return the right number of cores, but
/// all with a load of 0. This usually happens on startup.
#[cfg(any(not(debug_assertions), test))]
pub fn diff(older: &[LoadCounters], newer: &[LoadCounters]) -> Vec<CpuLoad> {
    let mut result: Vec<CpuLoad> = vec![];
    if older.len() != newer.len() {
        // Return the correct number of cores, but with zero load. This enables
        // us to draw a not-empty image on startup.
        for _ in newer.iter() {
            result.push(CpuLoad {
                user_0_to_1: 0.0,
                system_0_to_1: 0.0,
            });
        }
        return result;
    }

    for (older, newer) in older.iter().zip(newer.iter()) {
        let user = newer.user.wrapping_sub(older.user);
        let system = newer.system.wrapping_sub(older.system);
        let idle = newer.idle.wrapping_sub(older.idle);
        let total = user + system + idle;
        result.push(CpuLoad {
            user_0_to_1: user as f32 / total as f32,
            system_0_to_1: system as f32 / total as f32,
        });
    }
    return result;
}

/// Hard code load in debug builds to simplify testing the visualization
#[cfg(all(debug_assertions, not(test)))]
pub fn diff(_: &[LoadCounters], _: &[LoadCounters]) -> Vec<CpuLoad> {
    return vec![CpuLoad {
        user_0_to_1: 0.5,
        system_0_to_1: 0.5,
    }];
}

#[cfg(test)]
mod tests {
    use crate::cpuload::LoadCounters;

    /// Test that diff() can handle one counter wrapping around to zero
    #[test]
    fn test_diff_wrap() {
        let older = vec![LoadCounters {
            user: usize::MAX,
            system: usize::MAX,
            idle: usize::MAX,
        }];
        let newer = vec![LoadCounters {
            user: 0,   // 1 up since last
            system: 1, // 2 up since last
            idle: 2,   // 3 up since last
        }];
        let result = super::diff(&older, &newer);
        assert_eq!(1, result.len());
        assert_eq!(1.0 / (1.0 + 2.0 + 3.0), result[0].user_0_to_1);
        assert_eq!(2.0 / (1.0 + 2.0 + 3.0), result[0].system_0_to_1);
    }
}
