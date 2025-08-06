use crate::attribute::Attribute;

pub struct InstanceHeader {
    pub relation_name: String,
    pub attributes: Vec<Box<dyn Attribute>>,
    pub class_index: usize,
}

impl InstanceHeader {
    pub fn new(
        relation_name: String,
        attributes: Vec<Box<dyn Attribute>>,
        class_index: usize,
    ) -> InstanceHeader {
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

    pub fn relation_name(&self) -> &str {
        &self.relation_name
    }

    pub fn attribute_at_index(&self, index: usize) -> Option<&dyn Attribute> {
        if index < self.attributes.len() {
            Some(self.attributes[index].as_ref())
        } else {
            None
        }
    }

    pub fn index_of_attribute(&self, name: &str) -> Option<usize> {
        for (i, attr) in self.attributes.iter().enumerate() {
            if attr.name() == name {
                return Some(i);
            }
        }
        None
    }

    fn class_index(&self) -> usize {
        self.class_index
    }

    fn number_of_classes(&self) -> usize {
        if self.class_index < self.attributes.len() {
            if let Some(nominal_attr) = self.attributes[self.class_index]
                .as_any()
                .downcast_ref::<crate::attribute::NominalAttribute>()
            {
                return nominal_attr.values.len();
            }
        }
        0
    }
}
