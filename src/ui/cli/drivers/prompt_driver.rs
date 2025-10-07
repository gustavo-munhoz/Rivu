use anyhow::Result;

pub trait PromptDriver {
    fn ask_bool(&self, title: &str, help: &str, default: bool) -> Result<bool>;
    fn ask_string(&self, title: &str, help: &str, default: &str) -> Result<String>;
    fn ask_u64(
        &self,
        title: &str,
        help: &str,
        default: u64,
        min: Option<u64>,
        max: Option<u64>,
    ) -> Result<u64>;
    fn ask_f64(
        &self,
        title: &str,
        help: &str,
        default: f64,
        min: Option<f64>,
        max: Option<f64>,
    ) -> Result<f64>;
}
