use crate::core::instance_header::InstanceHeader;
use crate::core::instances::instance::Instance;
use std::io::Error;

pub trait Stream {
    fn header(&self) -> &InstanceHeader;

    fn has_more_instances(&self) -> bool;

    fn next_instance(&mut self) -> Option<Box<dyn Instance>>;

    fn restart(&mut self) -> Result<(), Error>;
}
