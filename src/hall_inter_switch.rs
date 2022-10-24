use std::cell::{RefCell};
use std::ops::Deref;
use std::sync::Arc;
use serde_derive::*;
use crate::{Locks};


#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub (crate) struct InterSwitch {
    pub state: String,
}

pub (crate) const HALL_INTER_SWITCH : &str = "hall_inter_switch";

#[derive(Debug)]
pub (crate) struct HallInterSwitchDevice {
}

impl HallInterSwitchDevice {
    pub (crate) fn new() -> Self {
        Self {}
    }

    pub fn get_name() -> &'static str {
        HALL_INTER_SWITCH
    }

    pub (crate) fn execute(&self, arc_locks : Arc<RefCell<Locks>>) {
        info!(">>>>>>>>>> execute device {}", Self::get_name());
        let borr = arc_locks.as_ref().borrow();
        let locks = borr.deref().clone();
        info!(">>>>>>>>>>> rc_locks before SWITCH {:?}", &locks);
    }
}