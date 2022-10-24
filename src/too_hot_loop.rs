use crate::{DynDevice, DynLoop, InsideTempSensorDevice};

pub (crate) struct TooHotLoop {}

impl DynLoop for TooHotLoop {
    fn get_devices(&self) -> Vec<Box<dyn DynDevice>> {
        vec![Box::new(InsideTempSensorDevice::new())]
    }
}