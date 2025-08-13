use crate::core::attributes::Attribute;
use crate::core::instance_header::InstanceHeader;
use std::io::Error;

pub trait Instance {
    fn weight(&self) -> f64;

    fn set_weight(&mut self, new_value: f64) -> Result<(), Error>;

    fn value_at_index(&self, index: usize) -> Option<f64>;

    fn set_value_at_index(&mut self, index: usize, new_value: f64) -> Result<(), Error>;

    fn is_missing_at_index(&self, index: usize) -> Result<bool, Error>;

    fn attribute_at_index(&self, index: usize) -> Option<&dyn Attribute>;

    fn index_of_attribute(&self, attribute: &dyn Attribute) -> Option<usize>;

    fn class_index(&self) -> usize;

    fn class_value(&self) -> Option<f64>;

    fn set_class_value(&mut self, new_value: f64) -> Result<(), Error>;

    fn is_class_missing(&self) -> bool;

    fn number_of_classes(&self) -> usize;

    fn to_vec(&self) -> Vec<f64>;

    fn header(&self) -> &InstanceHeader;
}
