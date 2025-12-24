use std::cell::RefCell;
use std::ops::Deref;
use std::str;
use std::sync::Arc;
use log::{info, error, debug};
use rumqttc::v5::{AsyncClient, Event, EventLoop, Incoming};
use serde::de::DeserializeOwned;
use tokio::time::{sleep, Duration};
use crate::generic_device::{GenericDevice, Locality};
use crate::hard_loop::HardLoop;

pub async fn process_incoming_message<T, F>(
    mut client: &mut AsyncClient,
    eventloop: &mut EventLoop,
    args: &[String],
    find_loop_fn: F,
)
where
    T: Locality + DeserializeOwned ,
    F: Fn(&str) -> (Vec<HardLoop<T>>, Option<Arc<RefCell<GenericDevice<T>>>>),
{
    info!("Process incoming message");

    loop {
        match eventloop.poll().await {
            Ok(notification) => match notification {
            Event::Incoming(Incoming::Publish(publish)) => {
                let msg = std::str::from_utf8(&publish.payload).unwrap();
                let topic = std::str::from_utf8(publish.topic.as_ref()).unwrap();

                info!("🧶 Publish on topic: [{}], message: <{}>", topic, msg);

                let (loops, opt_device) = find_loop_fn(topic);

                match opt_device {
                    None => {
                        info!("No device to process the message");
                    }
                    Some(dev) => {
                        let device_ref = dev.as_ref().borrow();
                        let device = device_ref.deref();

                        let original_message = match device.message_type.json_to_local(msg) {
                            Ok(om) => om,
                            Err(e) => {
                                error!(
                                    "💀 Cannot parse the message locally for device {}, msg=<{}>, \n e={}",
                                    &device.get_topic().to_uppercase(),
                                    msg,
                                    e
                                );
                                continue;
                            }
                        };

                        let o_ext_data = original_message.compute().await;
                        
                        for lp in loops {
                            if device.process_and_continue(&original_message, args).await {
                                lp.loop_devices(&topic, &original_message, o_ext_data.as_ref(), &mut client).await;
                            }
                        }
                    }
                }
            }
            Event::Incoming(Incoming::ConnAck(connack)) => {
                // Accéder aux métadonnées de la réponse de connexion (Connack)
                info!("Réponse à la connection ack");
                debug!("ConnaAck ({:?})", &connack);
            }
            Event::Incoming(Incoming::PubAck(pub_ack)) => {
                debug!("PubAck ({:?})", &pub_ack);
            }
            _ => {
                debug!("Other cases!");
            }
            },
            Err(e) => {
                error!("MQTT event loop error: {}", e);
                sleep(Duration::from_millis(500)).await;
            }
        }
    }
}
