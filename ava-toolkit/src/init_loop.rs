use std::cell::RefCell;
use std::ops::DerefMut;
use std::sync::Arc;
use log::info;
use rumqttc::v5::{Event, Incoming};
use crate::generic_device::{GenericDevice, Locality};

pub async fn handle_event<T>(event: Event, device_to_init: &Vec<Arc<RefCell<GenericDevice<T>>>>) 
where T : Locality  {
    info!("Message reçu = {:?}", &event);
    match event {
        Event::Incoming(Incoming::Publish(publish)) => {
            // Votre logique de traitement des messages ici

            let msg = std::str::from_utf8(&publish.payload).unwrap();
            let topic = std::str::from_utf8(publish.topic.as_ref()).unwrap(); // TODO

            info!( "Message reçu sur le topic {:?}: {:?}",topic  , msg);
            info!("Publish ({}): {}", topic, msg);

            // TODO is it necessary to loop over all the devices ?
            for dev in device_to_init {
                let mut borr = dev.as_ref().borrow_mut();
                let dd = borr.deref_mut();
                dd.init(topic, msg);
            }

        }
        Event::Incoming(Incoming::ConnAck(connack)) => {
            // Accéder aux métadonnées de la réponse de connexion (Connack)
            info!("ConnaAck ({:?})", &connack);
            if let Some(properties) = connack.properties {
                info!("Propriétés de la réponse de connexion: {:?}", properties);
            }
        }
        Event::Incoming(Incoming::PubAck(pub_ack)) => {
            info!("PubAck ({:?})", &pub_ack);
        }
        _ => {}
    }
}