use std::ops::Deref;

use log::{error, info};
use rumqttc::v5::{AsyncClient, Event, EventLoop, Incoming};

use crate::loops::{find_loops, HardLoop};

///
///
///
pub async fn process_incoming_message(mut client: &mut AsyncClient, eventloop: &mut EventLoop, mut all_loops: &mut Vec<HardLoop>, args: &[String])  {
    info!("Process incoming message");
    while let Ok(notification) = eventloop.poll().await {
        info!("New notification");
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
                        
                        // Change the msg into the DeviceMessage box of the ad hoc device (the original device)
                        let original_message = match dd.message_type.json_to_local(msg) {
                            Ok(om) => {om}
                            Err(e) => {
                                error!("💀 Cannot parse the message locally for device {}, msg=<{}>, \n e={}", &dd.get_topic().to_uppercase(), msg, e);
                                continue
                            }
                        };

                        // Here we can compute anything that will be used later on in the message consumption
                        let ext_data = original_message.compute().await;

                        for lp in loops {
                            info!("Before Looping");
                            if dd.process_and_continue(&original_message, &args).await {
                                lp.loop_devices(&topic, &original_message, &ext_data, &mut client).await;
                            }
                        }
                    }
                }
            }
            Event::Incoming(Incoming::ConnAck(_connack)) => {

            }
            Event::Incoming(Incoming::PubAck(_pub_ack)) => {

            }
            _ => {}
        }
    }
}