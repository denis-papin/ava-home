use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;

use log::info;
use rumqttc::v5::{AsyncClient, Event, Incoming};
use rumqttc::v5::EventLoop;
use rumqttc::v5::mqttbytes::QoS;

use crate::generic_device::GenericDevice;

pub (crate) fn build_init_list(_device_repo : &HashMap<String, Arc<RefCell<GenericDevice>>>) -> Vec<Arc<RefCell<GenericDevice>>> {
    // Nothing to init
    vec![]
}

///
/// Send an information message for all the device we want to init
/// Read the responses from mosquitto and run the init routine for the devices.
///
pub (crate) async fn process_initialization_message(client: &mut AsyncClient, eventloop: &mut EventLoop, device_to_init: &Vec<Arc<RefCell<GenericDevice>>>) -> Result<(), String> {

    info!("Initialisation stage starts");

    if !device_to_init.is_empty() {
        for dev in device_to_init {
            let borr = dev.as_ref().borrow();
            let dd = borr.deref();

            dbg!("Topic", &dd.get_topic());
            let data = dd.trigger_info();
            client.publish(&format!("{}/get", &dd.get_topic()), QoS::AtLeastOnce, false,  data).await.unwrap(); // TODO
        }

        // FIXME : if one of the device never replies, the loop never ends
        while let Ok(notification) = eventloop.poll().await {
            let mut end_loop = true;
            handle_event(notification, device_to_init).await;
            for dev in device_to_init {
                let borr = dev.as_ref().borrow();
                let dd = borr.deref();
                info!("Devices before check : topic=[{}], init =[{}]", dd.get_topic(), dd.is_init());
                if !dd.is_init() {
                    end_loop = false;
                }
            }
            if end_loop {
                break;
            }
        }
    } // device is empty

    info!("Initialisation stage finished");

    Ok(())
}

async fn handle_event(event: Event, device_to_init: &Vec<Arc<RefCell<GenericDevice>>>) {
    println!("Message reçu = {:?}", &event);
    match event {
        Event::Incoming(Incoming::Publish(publish)) => {
            // Votre logique de traitement des messages ici

            let msg = std::str::from_utf8(&publish.payload).unwrap();
            let topic = std::str::from_utf8(publish.topic.as_ref()).unwrap(); // TODO

            println!( "Message reçu sur le topic {:?}: {:?}",topic  , msg);

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
                println!("Propriétés de la réponse de connexion: {:?}", properties);
            }
        }
        Event::Incoming(Incoming::PubAck(_pub_ack)) => {
            info!("PubAck ({:?})", &_pub_ack);
        }
        _ => {}
    }
}