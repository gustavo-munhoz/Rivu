pub fn normal_probability(a: f64) -> f64 {
    0.5 * (1.0 + libm::erf(a / (2.0f64).sqrt()))
}
