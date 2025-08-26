use crate::core::attributes::{AttributeRef, NominalAttribute};
use crate::core::instance_header::InstanceHeader;
use std::collections::HashMap;
use std::sync::Arc;

pub const COLOR: [&str; 8] = [
    "black", "blue", "cyan", "brown", "red", "green", "yellow", "magenta",
];
pub const PRICE: [&str; 8] = [
    "veryLow",
    "low",
    "normal",
    "high",
    "veryHigh",
    "quiteHigh",
    "enormous",
    "non_salable",
];
pub const PAYMENT: [&str; 9] = ["0", "30", "60", "90", "120", "150", "180", "210", "240"];
pub const AMOUNT: [&str; 8] = [
    "veryLow",
    "low",
    "normal",
    "high",
    "veryHigh",
    "quiteHigh",
    "enormous",
    "non_ensured",
];
pub const DELAY: [&str; 5] = ["veryLow", "low", "normal", "high", "veryHigh"];
pub const CLASS: [&str; 2] = ["interested", "notInterested"];

#[inline]
pub fn idx(domain: &[&str], label: &str) -> usize {
    domain
        .iter()
        .position(|&s| s == label)
        .expect("label not in domain")
}

/// Builds the fixed `InstanceHeader` for the Asset Negotiation stream:
///
/// Attributes (all nominal):
/// 0. color (8 values)
/// 1. price (8 values)
/// 2. payment (9 values)
/// 3. amount (8 values)
/// 4. deliveryDelay (5 values)
/// 5. class (2 values: "interested", "notInterested")
///
/// The class index is 5. The header is immutable and shared across
/// instances via Arc.
pub fn build_header() -> InstanceHeader {
    let mut attrs: Vec<AttributeRef> = Vec::new();

    let mut push_nominal = |name: &str, labels: &[&str]| {
        let vals: Vec<String> = labels.iter().map(|s| s.to_string()).collect();
        let mut map = HashMap::new();
        for (i, lab) in vals.iter().enumerate() {
            map.insert(lab.clone(), i);
        }
        attrs.push(Arc::new(NominalAttribute::with_values(name.into(), vals, map)) as AttributeRef);
    };

    push_nominal("color", &COLOR);
    push_nominal("price", &PRICE);
    push_nominal("payment", &PAYMENT);
    push_nominal("amount", &AMOUNT);
    push_nominal("deliveryDelay", &DELAY);
    push_nominal("class", &CLASS);

    InstanceHeader::new("asset_negotiation".into(), attrs, 5)
}
