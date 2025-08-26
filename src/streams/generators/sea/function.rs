#[derive(Clone, Copy, Debug)]
pub enum SeaFunction {
    F1 = 1,
    F2 = 2,
    F3 = 3,
    F4 = 4,
}

impl SeaFunction {
    pub fn threshold(self) -> f64 {
        match self {
            SeaFunction::F1 => 8.0,
            SeaFunction::F2 => 9.0,
            SeaFunction::F3 => 7.0,
            SeaFunction::F4 => 9.5,
        }
    }
}
