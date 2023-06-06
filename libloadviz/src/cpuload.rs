use std::cmp::Ordering;

#[derive(PartialEq, Copy, Clone, Debug)]
pub struct CpuLoad {
    // NOTE: Maybe we should do "idle" here as well?
    pub user_0_to_1: f32,
    pub system_0_to_1: f32,
}

#[derive(Debug)]
pub struct LoadCounters {
    pub user: usize,
    pub system: usize,
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

/// Based on two CPU counter snapshots, compute the load for each CPU
pub fn diff(older: &[LoadCounters], newer: &[LoadCounters]) -> Vec<CpuLoad> {
    if older.len() != newer.len() {
        return vec![];
    }

    let mut result: Vec<CpuLoad> = vec![];
    for (older, newer) in older.iter().zip(newer.iter()) {
        let user = newer.user - older.user;
        let system = newer.system - older.system;
        let idle = newer.idle - older.idle;
        let total = user + system + idle;
        result.push(CpuLoad {
            user_0_to_1: user as f32 / total as f32,
            system_0_to_1: system as f32 / total as f32,
        });
    }
    return result;
}
