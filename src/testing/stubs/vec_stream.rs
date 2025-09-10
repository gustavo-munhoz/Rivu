use crate::core::instance_header::InstanceHeader;
use crate::core::instances::{DenseInstance, Instance};
use crate::streams::Stream;
use crate::testing::dummies::header_binary;
use std::io::Error;
use std::sync::Arc;

pub struct VecStream {
    pub header: Arc<InstanceHeader>,
    pub labels: Vec<usize>,
    idx: usize,
}

impl VecStream {
    pub fn new(labels: Vec<usize>) -> Self {
        Self {
            header: header_binary(),
            labels,
            idx: 0,
        }
    }
}

impl Stream for VecStream {
    fn header(&self) -> &InstanceHeader {
        &self.header
    }

    fn has_more_instances(&self) -> bool {
        self.idx < self.labels.len()
    }

    fn next_instance(&mut self) -> Option<Box<dyn Instance>> {
        if !self.has_more_instances() {
            return None;
        }

        let y = self.labels[self.idx];
        self.idx += 1;
        Some(Box::new(DenseInstance::new(
            Arc::clone(&self.header),
            vec![y as f64],
            1.0,
        )))
    }

    fn restart(&mut self) -> Result<(), Error> {
        self.idx = 0;
        Ok(())
    }
}
