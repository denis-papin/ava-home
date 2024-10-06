use std::ops::Deref;
use log::{error, info};
use rumqttc::v5::{AsyncClient, Event, EventLoop, Incoming};
use crate::loops::{find_loops, HardLoop};

///
///
///
pub async fn process_incoming_message(mut client: &mut AsyncClient, mut eventloop: &mut EventLoop, mut all_loops: &mut Vec<HardLoop>)  {

    while let Ok(notification) = eventloop.poll().await {
        info!("New notification, [{:?}]", &notification);
        match notification {
            Event::Incoming(Incoming::Publish(publish)) => {
                let msg = std::str::from_utf8(&publish.payload).unwrap();
                let topic = std::str::from_utf8(publish.topic.as_ref()).unwrap(); // TODO

                info!("🧶 Publish on topic: [{}], message: <{}>", topic, msg);
                let (loops, opt_device) = find_loops(&topic, &mut all_loops);

                match opt_device {
                    None => {
                        info!("No device to process the message");
                    }
                    Some(dev) => {
                        info!("Receiver device found !");
                        let dd1 = dev.as_ref().borrow();
                        let dd = dd1.deref();
                        for lp in loops {
                            info!("Before Looping");
                            // Change the msg into the DeviceMessage box of the ad hoc device (the original device)
                            let original_message = match dd.message_type.json_to_local(msg) {
                                Ok(om) => {om}
                                Err(e) => {
                                    error!("💀 Cannot parse the message locally for device {}, msg=<{}>, \n e={}", &dd.get_topic().to_uppercase(), msg, e);
                                    continue
                                }
                            };

                            if dd.process_and_continue(&original_message) {
                                lp.loop_devices(&topic, &original_message, &mut client).await;
                            }
                        }
                    }
                }
            }
            Event::Incoming(Incoming::ConnAck(connack)) => {
                // Accéder aux métadonnées de la réponse de connexion (Connack)
                info!("ConnaAck ({:?})", &connack);
                if let Some(properties) = connack.properties {
                    println!("Propriétés de la réponse de connexion: {:?}", properties);
                }
            }
            Event::Incoming(Incoming::PubAck(pubAck)) => {
                info!("PubAck ({:?})", &pubAck);
            }
            _ => {
                info!("Other case!");
            }
        }
    }
}