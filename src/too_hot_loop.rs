

pub (crate) const TOO_HOT_LOOP : &str = "TOO_HOT_LOOP";

// pub (crate) struct TooHotLoop {}
//
// impl DynLoop for TooHotLoop {
//     fn get_devices(&self) -> Vec<Box<dyn DynDevice>> {
//         vec![Box::new(InsideTempSensorDevice::new())/*, Box::new(OutdoorTempSensorDevice::new())*/]
//     }
//
//     fn get_name(&self) -> &'static str {
//         TOO_HOT_LOOP
//     }
// }