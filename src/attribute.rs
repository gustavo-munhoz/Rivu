pub trait Attribute {
    fn name(&self) -> String;
}

pub struct NumericAttribute {
    pub name: String,
    pub values: Vec<u32>,
}

impl NumericAttribute {
    pub fn new(name: String) -> NumericAttribute {
        NumericAttribute { name, values: Vec::new() }
    }
    
    pub fn with_values(name: String, values: Vec<u32>) -> NumericAttribute {
        NumericAttribute { name, values }
    }
}

impl Attribute for NumericAttribute {
    fn name(&self) -> String {
        self.name.clone()
    }
}

pub struct NomitalAttribute {
    pub name: String,
    pub values: Vec<String>,
}

impl NomitalAttribute {
    pub fn new(name: String) -> NomitalAttribute {
        NomitalAttribute { name, values: Vec::new() }
    }
    
    pub fn with_values(name: String, values: Vec<String>) -> NomitalAttribute {
        NomitalAttribute { name, values }
    }
}

impl Attribute for NomitalAttribute {
    fn name(&self) -> String {
        self.name.clone()
    }
}