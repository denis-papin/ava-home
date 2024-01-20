
use serde_derive::*;

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq)]
pub (crate) struct RegulationMap {
    pub tc_bureau: f32,
    pub tc_salon_1: f32,
    pub tc_salon_2: f32,
    pub tc_chambre_1: f32,
    pub tc_couloir: f32,
    pub mode: char, // J /"JOUR", N / "NUIT", A / "ABSENCE"
}

///
/// {
//       "tc_bureau": 23.0,
//       "tc_salon_1": 22.5,
//       "tc_salon_2": 24.0,
//       "tc_chambre_1": 21.0,
//       "tc_couloir": 22.0,
//       "mode": 'J',
//   }
///
///
impl RegulationMap {
    pub (crate) fn new() -> Self {
        Self {
            tc_bureau: 0.0,
            tc_salon_1: 0.0,
            tc_salon_2: 0.0,
            tc_chambre_1: 0.0,
            tc_couloir: 0.0,
            mode: 'J',
        }
    }

    pub (crate) fn from_json(msg: &str) -> Result<Self, String> {
        serde_json::from_str(msg).map_err(|e| e.to_string())
    }
}


