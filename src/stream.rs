use std::io::Write;
use std::net::TcpStream;
use std::thread;
use std::time::{Duration, Instant};

use mqtt::packet::*;

use mqtt::{Decodable, Encodable};
use crate::Params;

pub (crate) fn wait_subpack(mut stream : &mut TcpStream) ->Result<(), String>   {
    loop {
        let packet = match VariablePacket::decode(&mut stream) {
            Ok(pk) => pk,
            Err(err) => {
                error!("Error in receiving packet {:?}", err);
                continue;
            }
        };
        trace!("PACKET {:?}", packet);

        if let VariablePacket::SubackPacket(ref ack) = packet {
            if ack.packet_identifier() != 10 {
                panic!("SUBACK packet identifier not match");
            }

            info!("Subscribed!");
            break;
        }
    }
    Ok(())
}

pub (crate) fn ping_broker(stream : &mut TcpStream, params : &Params) -> Result<(), String> {
    let params_clone = (*params).clone();
    let mut stream_clone = stream.try_clone().unwrap();
    thread::spawn(move || {
        let mut last_ping_time = Instant::now();
        let mut next_ping_time = last_ping_time + Duration::from_secs((params_clone.keep_alive as f32 * 0.9) as u64);
        loop {
            let current_timestamp = Instant::now();
            if params_clone.keep_alive > 0 && current_timestamp >= next_ping_time {
                info!("Sending PINGREQ to broker");

                let pingreq_packet = PingreqPacket::new();

                let mut buf = Vec::new();
                pingreq_packet.encode(&mut buf).unwrap();
                stream_clone.write_all(&buf[..]).unwrap();

                last_ping_time = current_timestamp;
                next_ping_time = last_ping_time + Duration::from_secs((params_clone.keep_alive as f32 * 0.9) as u64);
                thread::sleep(Duration::new((params_clone.keep_alive / 2) as u64, 0));
            }
        }
    });
    Ok(())
}
