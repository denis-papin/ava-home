
pub (crate) const KITCHEN_LOOP : &str = "KITCHEN_LOOP";

// pub (crate) struct KitchenLoop {}
//
// impl KitchenLoop {
// }
//
// impl DynLoop for KitchenLoop {
//     fn get_devices(&self) -> Vec<Box<dyn DynDevice>> {
//         /**
//             KITCHEN_INTER_DIM_DEVICE = KitchenInterDimDevice::new(name, props)
//
//             vec![
//                 Box::new(KITCHEN_INTER_DIM_DEVICE.get(),
//                 Box::new(KitchenLampDevice::new()),
//                 Box::new(HallLampDevice::new())]
//         */
//
//         vec![
//             Box::new(KitchenInterDimDevice::new()),
//             // Box::new(KitchenSwitchDevice::new()),
//              Box::new(KitchenLampDevice::new()),
//              Box::new(HallLampDevice::new())]
//     }
//
//     fn get_name(&self) -> &'static str {
//         KITCHEN_LOOP
//     }
// }