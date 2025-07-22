use crate::attribute::Attribute;

pub struct InstanceHeader {
    pub relation_name: String,
    pub attributes: Vec<Box<dyn Attribute>>,
    pub class_index: usize,
}

impl InstanceHeader {
    pub fn new(relation_name: String, attributes: Vec<Box<dyn Attribute>>, class_index: usize) -> InstanceHeader {
        InstanceHeader {
            relation_name,
            attributes,
            class_index,
        }
    }

    pub fn class_attribute(&self, index: usize) -> &dyn Attribute {
        self.attributes[index].as_ref()
    }
    
    pub fn number_of_attributes(&self) -> usize {
        self.attributes.len()
    }
}