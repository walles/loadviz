use std::cmp::Ordering;

#[derive(PartialEq, Copy, Clone, Debug)]
pub struct CpuLoad {
    // NOTE: Maybe we should do "idle" here as well?

    pub user_0_to_1: f32,
    pub system_0_to_1: f32,
}

impl PartialOrd for CpuLoad {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        return (self.user_0_to_1 + self.system_0_to_1).partial_cmp(&(other.user_0_to_1 + other.system_0_to_1))
    }
}
