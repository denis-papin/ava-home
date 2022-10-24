use crate::{DynDevice, DynLoop, HallLampDevice, KitchenInterDimDevice, KitchenLampDevice};

pub (crate) struct KitchenLoop {}

impl DynLoop for KitchenLoop {
    fn get_devices(&self) -> Vec<Box<dyn DynDevice>> {
        vec![Box::new(KitchenInterDimDevice::new()),
             Box::new(KitchenLampDevice::new()),
             Box::new(HallLampDevice::new())]
    }
}