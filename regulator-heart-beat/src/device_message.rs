use serde_derive::*;

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq)]
pub (crate) struct RegulationMap {
    pub tc_bureau: f32,
    pub tc_salon_1: f32,
    pub tc_salon_2: f32,
    pub tc_chambre_1: f32,
    pub tc_couloir: f32,
}






