//Simply formatted with `cargo fmt`

use regex::bytes::Regex;
use std::collections::HashMap;

use ppp_packet::Packet;

pub fn decode(packet: Packet, status: &mut HashMap<&str, String>) -> () {
    // println!("{:?}", packet);
    // let payload = OsString::from_vec(packet.payload.clone());
    // let payload = payload.to_string_lossy();
    match packet.header.row {
        1 => {
            status.insert(
                "genset/engaged",
                match packet.payload[0] == 0x03 && packet.payload[4] != 0xa4 {
                    true => "1".to_string(),
                    false => "0".to_string(),
                },
            );
        }
        2 => {
            let re = Regex::new(r"^(\d+\.\d)kW\s+(\x01|\x02)\s+(-?\d+\.\d+)kW.+?(o|\x06)(o|\x06)")
                .unwrap();
            let caps: Vec<String> = re
                .captures(packet.payload.as_slice())
                .unwrap()
                .iter()
                .map(|c| String::from_utf8(c.unwrap().as_bytes().to_vec()).unwrap())
                .collect();
            status.insert("genset/output", caps[1].parse().unwrap());
            status.insert(
                "flow",
                match caps[2].as_str() {
                    "\u{1}" => "charge".to_string(),
                    "\u{2}" => "discharge".to_string(),
                    _ => "unknown".to_string(),
                },
            );
            status.insert("load", caps[3].parse().unwrap());
            status.insert(
                "battery/fan",
                match caps[4] != "o" {
                    true => "1".to_string(),
                    false => "0".to_string(),
                },
            );
            status.insert(
                "genset/requested",
                match caps[5] != "o" {
                    true => "1".to_string(),
                    false => "0".to_string(),
                },
            );
        }
        3 => {
            let re = Regex::new(r"^[\*!\?]").unwrap();
            let engaged = match re.captures(packet.payload.as_slice()) {
                Some(c) => {
                    let s = status.clone();
                    let output = match s.get(&"genset/output") {
                        Some(o) => o,
                        None => "",
                    };
                    match c[0][0] {
                        33 if output == "0.0" => "0",
                        _ => "1",
                    }
                }
                None => "0",
            };
            status.insert("genset/engaged", engaged.to_string());
        }
        4 => {
            let re = Regex::new(r"\s+(\d+)%\s+\d{2}:\d{2}:\d{2}").unwrap();
            let caps = re.captures(packet.payload.as_slice()).unwrap();
            status.insert(
                "battery/charge",
                String::from_utf8(caps[1].to_vec())
                    .unwrap()
                    .parse()
                    .unwrap(),
            );
        }
        _ => {}
    }
    // println!("{:?}", payload);
}
