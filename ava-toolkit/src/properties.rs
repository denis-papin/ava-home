// use std::collections::HashMap;
// use std::ops::DerefMut;
// use std::sync::RwLock;
//
// use anyhow::anyhow;
// use lazy_static::*;
//
// lazy_static! {
//     static ref PROPS : RwLock<HashMap<u32, &'static mut HashMap<String,String>> > = RwLock::new(
//         {
//             let mut m = HashMap::new();
//             let props : HashMap<String,String> = HashMap::new();
//             m.insert(0, Box::leak(Box::new( props )));
//             m
//         });
// }
//
// // "app.customerfile"
// pub fn _get_prop_value(prop_name : &str) -> anyhow::Result<String> {
//     // https://doc.rust-lang.org/std/sync/struct.RwLock.html
//     let v = PROPS.read().unwrap().deref().get(&0).ok_or(anyhow!("Shared map not found: [{}]", prop_name))?.deref()
//         .get(prop_name).ok_or(anyhow!("Prop not found: [{}]", prop_name))?.trim().to_owned();
//     Ok(v)
// }
//
// //
// pub fn _set_prop_value(prop_name : &str, value : &str ) {
//     if let Ok(write_guard) = PROPS.write().as_mut() {
//         // the returned write_guard implements `Deref` giving us easy access to the target value
//
//         let map = write_guard.deref_mut();
//         if  let Some( item ) = map.get_mut(&0) {
//             item.insert(prop_name.to_string(), value.to_string());
//         }
//     }
// }
