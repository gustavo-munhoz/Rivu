use crate::core::attributes::{AttributeRef, NominalAttribute};
use crate::core::instance_header::InstanceHeader;
use std::collections::HashMap;
use std::sync::Arc;

pub fn header_binary() -> Arc<InstanceHeader> {
    let vals = vec!["A".to_string(), "B".to_string()];
    let mut map = HashMap::new();
    map.insert("A".to_string(), 0);
    map.insert("B".to_string(), 1);
    let class_attribute =
        Arc::new(NominalAttribute::with_values("class".into(), vals, map)) as AttributeRef;

    Arc::new(InstanceHeader::new("bin".into(), vec![class_attribute], 0))
}
