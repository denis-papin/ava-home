use log::info;
use rumqttc::v5::AsyncClient;
use rumqttc::v5::mqttbytes::QoS;

use crate::message_enum::MessageEnum;

#[derive(Debug)]
pub(crate) struct GenericDevice {
    pub name: String,
}

impl GenericDevice {

    pub(crate) fn new(name : &str) -> Self {
        info!("🌟 New Generic Device, topic = [{}]", &name);
        Self {
            name: name.to_string(),
        }
    }


    // better use the attribute directly
    pub(crate) fn get_topic(&self) -> String {
        format!("regulator/{}", self.name)
    }



    pub (crate) async fn publish_message_topic(&self, client: &mut AsyncClient, object_message : &MessageEnum) {
        let message = object_message.raw_message();
        info!("prepare to send :  [{}]", &message);
        let data = message.as_bytes().to_vec();
        info!("Publish on [{}]", &self.get_topic());
        client.publish(&self.get_topic(), QoS::AtLeastOnce, false, data).await.unwrap(); // TODO
    }

}
