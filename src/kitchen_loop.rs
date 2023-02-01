use crate::{DynDevice, DynLoop, HallLampDevice, KitchenInterDimDevice, KitchenLampDevice, KitchenSwitchDevice};
use crate::inside_temp_sensor::INSIDE_TEMP_SENSOR;

pub (crate) const KITCHEN_LOOP : &str = "KITCHEN_LOOP";

pub (crate) struct KitchenLoop {}

impl KitchenLoop {
}

impl DynLoop for KitchenLoop {
    fn get_devices(&self) -> Vec<Box<dyn DynDevice>> {
        vec![Box::new(KitchenInterDimDevice::new()),
             Box::new(KitchenSwitchDevice::new()),
             Box::new(KitchenLampDevice::new()),
             Box::new(HallLampDevice::new())]
    }

    fn get_name(&self) -> &'static str {
        KITCHEN_LOOP
    }
}