use std::fmt::{Display, Formatter, Result};

#[derive(Copy, Clone)]
pub struct Snapshot {
    pub instances_seen: u64,
    pub accuracy: f64,
    pub kappa: f64,
    pub ram_hours: f64,
    pub seconds: f64,
}

impl Display for Snapshot {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(
            f,
            "seen={}, acc={:.6}, kappa={:.6}, ram_h={:.6}, t={:.3}s",
            self.instances_seen, self.accuracy, self.kappa, self.ram_hours, self.seconds
        )
    }
}
