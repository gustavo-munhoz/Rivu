use crate::instance_header::InstanceHeader;

pub trait Instance{
    fn weight(&self) -> f64;

    fn value_at_index(&self, index: usize) -> Option<f64>;

    fn class_value(&self) -> Option<f64>;

    fn header(&self) -> &InstanceHeader;
}

pub struct DenseInstance {
    pub header: &'static InstanceHeader,
    pub values: Vec<f64>,
    pub weight: f64,
}

impl DenseInstance {
    pub fn new(header: &'static InstanceHeader, values: Vec<f64>, weight: f64) -> DenseInstance {
        DenseInstance { header, values, weight }
    }
}

impl Instance for DenseInstance {
    fn weight(&self) -> f64 {
        self.weight
    }

    fn value_at_index(&self, index: usize) -> Option<f64> {
        if index < self.values.len() {
            Some(self.values[index])
        } else {
            None
        }
    }

    fn class_value(&self) -> Option<f64> {
        if self.header.class_index < self.values.len() {
            Some(self.values[self.header.class_index])
        } else {
            None
        }
    }

    fn header(&self) -> &InstanceHeader {
        self.header
    }
}