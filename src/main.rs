#[macro_use]
extern crate serde_derive;
extern crate docopt;
extern crate mosquitto_client as mosq;
extern crate regex;

pub mod parse;
pub mod decode;
pub mod ppp_packet;

use docopt::Docopt;
use std::error::Error;
use std::io::prelude::*;
use std::fs::File;
use std::path::Path;
use std::collections::HashMap;
use mosq::Mosquitto;

use parse::parse;
use decode::decode;

const USAGE: &'static str = "
si2mqtt: Read data off the SMA Sunny Island RS485 display bus
         and send it to an MQTT broker.

Usage:
  si2mqtt -p <tty> -m <broker> [-t <topic>]
  si2mqtt -h | --help

Options:
  -p, --port=<port>     The serial port from which to read RS485 data
  -m, --mqtt=<broker>   The MQTT broker to which to connect
  -t, --topic=<topic>   The MQTT topic under which to publish our data
  -h, --help            Show this documentation
";

#[derive(Deserialize)]
struct Args {
	flag_port: String,
	flag_mqtt: String,
	flag_topic: String,
}

fn main() {
	let mut status: HashMap<&str, String> = HashMap::new();
	/* 
		flow: charge|discharge,
		load: 0.0,
		genset/engaged: false,
		genset/requested: false,
		genset/output: 0.0,
		battery/fan: false,
		battery/charge: 100,
		battery/health: 100,
	*/
	let args: Args = Docopt::new(USAGE)
		.and_then(|d| d.deserialize())
		.unwrap_or_else(|e| e.exit());

	let m = Mosquitto::new("test");
	m.connect(&args.flag_mqtt, 1883).expect("Cannot connect");

	let path = Path::new(args.flag_port.as_str());
	let mut port = match File::open(&path) {
		Err(e) => panic!("Couldn't open {} for reading: {}",
			path.display(), e.description()),
		Ok(f) => f,
	};
	let mut bytes = 1;
	let mut buf = Vec::new();
	while bytes != 0 {
		let mut chunk = vec![0u8; 40];
		bytes = match port.read(&mut chunk) {
			Err(_) => panic!("Unable to read"),
			Ok(n)  => n,
		};
		if bytes > 0 {
			chunk.truncate(bytes);
			buf.append(&mut chunk);
			for packet in parse(&mut buf) {
				decode(packet, &mut status);
			}
		}
		mqtt_publish(&m, &args.flag_topic, &status);
	}
}

fn mqtt_publish(m: &Mosquitto, topic: &String, status: &HashMap<&str, String>) {
	println!("STATUS {:?}", status);
	for (k,v) in status {
		let t = format!("{}/{}", topic, k);
		let _mid = m.publish(t.as_str(), v.as_bytes(), 2, false);
	}
}
