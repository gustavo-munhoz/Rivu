use crate::core::attributes::Attribute;
use std::any::Any;
use std::collections::HashMap;

#[derive(Clone)]
pub struct NominalAttribute {
    pub name: String,
    pub values: Vec<String>,
    pub label_to_index: HashMap<String, usize>,
}

impl NominalAttribute {
    pub fn new(name: String) -> NominalAttribute {
        NominalAttribute {
            name,
            values: Vec::new(),
            label_to_index: HashMap::new(),
        }
    }

    pub fn with_values(
        name: String,
        values: Vec<String>,
        label_to_index: HashMap<String, usize>,
    ) -> NominalAttribute {
        NominalAttribute {
            name,
            values,
            label_to_index,
        }
    }

    pub fn get_attribute_values(&self) -> Vec<String> {
        self.values.clone()
    }

    pub fn index_of_value_mut(&mut self, v: &str) -> Option<usize> {
        self.values.iter().position(|x| x == v)
    }

    pub fn enumerate_values(&self) -> impl Iterator<Item = (usize, &String)> {
        self.values.iter().enumerate()
    }
}

impl Attribute for NominalAttribute {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn arff_representation(&self) -> String {
        let nominal = self.as_any().downcast_ref::<NominalAttribute>().unwrap();
        format!(
            "@attribute {} {{ {} }}",
            nominal.name(),
            nominal.values.join(", ")
        )
    }
}
