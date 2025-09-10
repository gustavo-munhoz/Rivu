use crate::core::attributes::{Attribute, NominalAttribute, NumericAttribute};
use crate::core::instance_header::InstanceHeader;
use crate::core::instances::instance::Instance;
use std::io::Error;
use std::sync::Arc;

pub struct DenseInstance {
    pub header: Arc<InstanceHeader>,
    pub values: Vec<f64>,
    pub weight: f64,
}

impl DenseInstance {
    pub fn new(header: Arc<InstanceHeader>, values: Vec<f64>, weight: f64) -> DenseInstance {
        DenseInstance {
            header,
            values,
            weight,
        }
    }
}

impl Instance for DenseInstance {
    fn weight(&self) -> f64 {
        self.weight
    }

    fn set_weight(&mut self, new_value: f64) -> Result<(), Error> {
        if new_value < 0.0 {
            Err(Error::new(
                std::io::ErrorKind::InvalidInput,
                "Weight cannot be negative",
            ))
        } else {
            self.weight = new_value;
            Ok(())
        }
    }

    fn value_at_index(&self, index: usize) -> Option<f64> {
        if index < self.values.len() {
            Some(self.values[index])
        } else {
            None
        }
    }

    fn set_value_at_index(&mut self, index: usize, new_value: f64) -> Result<(), Error> {
        if index < self.values.len() {
            self.values[index] = new_value;
            Ok(())
        } else {
            Err(Error::new(
                std::io::ErrorKind::InvalidInput,
                "Index out of bounds",
            ))
        }
    }

    fn is_missing_at_index(&self, index: usize) -> Result<bool, Error> {
        if index < self.values.len() {
            Ok(self.values[index].is_nan())
        } else {
            Err(Error::new(
                std::io::ErrorKind::InvalidInput,
                "Index out of bounds",
            ))
        }
    }

    fn attribute_at_index(&self, index: usize) -> Option<&dyn Attribute> {
        if index < self.header.attributes.len() {
            Some(&*self.header.attributes[index])
        } else {
            None
        }
    }

    fn index_of_attribute(&self, attribute: &dyn Attribute) -> Option<usize> {
        self.header
            .attributes
            .iter()
            .position(|attr| attr.name() == attribute.name())
    }

    fn number_of_attributes(&self) -> usize {
        self.header.attributes.len()
    }

    fn class_index(&self) -> usize {
        self.header.class_index()
    }

    fn class_value(&self) -> Option<f64> {
        if self.header.class_index() < self.values.len() {
            Some(self.values[self.header.class_index()])
        } else {
            None
        }
    }

    fn set_class_value(&mut self, new_value: f64) -> Result<(), Error> {
        if self.header.class_index() < self.values.len() {
            self.values[self.header.class_index()] = new_value;
            Ok(())
        } else {
            Err(Error::new(
                std::io::ErrorKind::InvalidInput,
                "Class index out of bounds",
            ))
        }
    }

    fn is_class_missing(&self) -> bool {
        if self.header.class_index() < self.values.len() {
            self.values[self.header.class_index()].is_nan()
        } else {
            false
        }
    }

    fn number_of_classes(&self) -> usize {
        let attr = &*self.header.attributes[self.class_index()];
        if attr.as_any().is::<NumericAttribute>() {
            0
        } else if let Some(nominal) = attr.as_any().downcast_ref::<NominalAttribute>() {
            nominal.values.len()
        } else {
            0
        }
    }

    fn to_vec(&self) -> Vec<f64> {
        self.values.clone()
    }

    fn header(&self) -> &InstanceHeader {
        &self.header
    }
}
