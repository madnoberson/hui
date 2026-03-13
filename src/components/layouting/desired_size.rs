#[derive(Debug, Clone, PartialEq)]
pub enum DesiredSize {
    Fixed(f32),
    Constrained { min_value: f32, desired_value: f32 },
    Greedy { min_value: f32, weight: u32 },
}

impl DesiredSize {
    pub fn min_value(&self) -> f32 {
        match self {
            Self::Fixed(value) => *value,
            Self::Constrained { min_value, .. } => *min_value,
            Self::Greedy { min_value, .. } => *min_value,
        }
    }
}
