use std::cell::RefCell;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;
use log::info;
use rumqttc::v5::{AsyncClient, Event, EventLoop, Incoming};
use rumqttc::v5::mqttbytes::QoS;
use serde::de::DeserializeOwned;
use crate::generic_device::{GenericDevice, Locality};

/// Send an information request for all devices that need initialization.
/// Each device publishes its `trigger_info` on the `<topic>/get` channel.
/// Then we listen to responses from Mosquitto and run the initialization routine
/// for each device until all are marked as initialized.
///
/// # Warning
///
/// ⚠️ If one of the devices never replies, this loop will **never terminate**.
/// Consider adding a timeout or a safeguard to avoid blocking forever.
pub async fn process_initialization_message<T>(
    client: &mut AsyncClient,
    eventloop: &mut EventLoop,
    device_to_init: &Vec<Arc<RefCell<GenericDevice<T>>>>,
) -> Result<(), String>
where
    T: Locality + DeserializeOwned,
{
    info!("Initialization stage starts");

    if !device_to_init.is_empty() {
        // Ask every device for its info
        for dev in device_to_init {
            let borr = dev.as_ref().borrow();
            let dd = borr.deref();

            let data = dd.trigger_info();
            client
                .publish(&format!("{}/get", dd.get_topic()), QoS::AtLeastOnce, false, data)
                .await
                .map_err(|e| format!("Publish failed: {}", e))?;
        }

        // Wait for all devices to acknowledge initialization
        while let Ok(notification) = eventloop.poll().await {
            let mut end_loop = true;

            handle_event(notification, device_to_init).await;

            for dev in device_to_init {
                let borr = dev.as_ref().borrow();
                let dd = borr.deref();
                info!(
                    "Devices before check : topic=[{}], init =[{}]",
                    dd.get_topic(),
                    dd.is_init()
                );
                if !dd.is_init() {
                    end_loop = false;
                }
            }

            if end_loop {
                break;
            }
        }
    } else {
        info!("No devices to initialize");
    }

    info!("Initialization stage finished");

    Ok(())
}


pub async fn handle_event<T>(event: Event, device_to_init: &Vec<Arc<RefCell<GenericDevice<T>>>>) 
where T : Locality  + DeserializeOwned {
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