use std::cell::{Cell, RefCell};
use std::ops::Deref;
use std::sync::Arc;
use serde_derive::*;
use crate::Locks;

// {"consumption":0,
// "current":0,
// "device_temperature":32,
// "energy":0,
// "illuminance":0,
// "illuminance_lux":0,
// "linkquality":162,
// "power":0,
// "power_outage_count":1,
// "state":"ON",
// "voltage":240.2}
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub (crate) struct InterSwitch {
    pub state: String,
}

#[derive(Debug)]
pub (crate) struct HallInterSwitchDevice {
}

impl HallInterSwitchDevice {
    pub (crate) fn new() -> Self {
        Self {
        }
    }

    pub (crate) fn execute(&self, arc_locks : Arc<RefCell<Locks>>) {
        info!(">>>>>>>>>> execute device SWITCH");
        // let mut locks = rc_locks.get_mut();
        let borr = arc_locks.as_ref().borrow();
        let mut locks = borr.deref().clone();
        info!(">>>>>>>>>>> rc_locks before SWITCH {:?}", &locks);
    }
}