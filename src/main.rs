#[macro_use]
extern crate serde_derive;
extern crate docopt;
//`failure` is the best option for error handling
// Many `unwrap`s are left there, but we can think about
// using `?` instead, and use the power of `failure`.
extern crate failure;
extern crate futures;
extern crate mosquitto_client as mosq;
extern crate regex;
extern crate tokio;

pub mod decode;
pub mod parse;
pub mod ppp_packet;

use docopt::Docopt;
use failure::Error;
use mosq::Mosquitto;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use tokio::prelude::*;
use tokio::timer::Interval;

use decode::decode;
use parse::parse;

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
    // There are two threads accessing the status,
    // one is read only (the timer) and the other is read-write (read_data).
    // Therefore we need `RwLock` for thread safety.
    //
    // Arc is also needed because neither the threads own the status.
    let status: Arc<RwLock<HashMap<&str, String>>> = Arc::new(RwLock::new(HashMap::new()));
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
    // There is no need to use Arc for other types as they
    // are not shared between threads.
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.deserialize())
        .unwrap_or_else(|e| e.exit());

    let m = Mosquitto::new("test");
    m.connect(&args.flag_mqtt, 1883)
        .expect("Cannot connect to mqtt broker");

    // `Path` is a borrowed version of `PathBuf`.
    // We need an owned version here.
    let filename = PathBuf::from(args.flag_port.clone());
    // Creating a clone for the timer, it will be moved into the closure.
    // Avoid cloning it everytime the timer ticks.
    let status_timer = status.clone();
    let timer = Interval::new(Instant::now(), Duration::from_millis(5000))
        .map_err(Error::from)
        .for_each(move |_t| {
            //When the expression is already returning a `Result<(),Error>`,
            //don't need to add `Ok(()) at the end.
            mqtt_publish(&m, &args.flag_topic, &status_timer.read().unwrap()).map_err(Error::from)
        })
        .map_err(|e| panic!("timer error {:?}", e));

    let port = tokio::fs::File::open(filename)
        .and_then(move |file| {
            tokio::spawn(timer);
            Ok(file)
        })
        .map_err(Error::from)
        .and_then(move |file| read_data(file, status))
        .map_err(|e| panic!("Unable to open file.txt {:?}", e));

    tokio::run(port);
}

// This function should not return a `Result`, as this is not the point that using `tokio`
// - it will be a blocking call.
// Instead, we should return a `Future`, so `tokio` can schedule the pending results
// at a later moment.
// When `poll_read` returns `Async::NotReady`, we cannot break the loop like receiving a zero
// sized result, as it will disconnect the connection. Instead, we delegate the not ready
// status to the `tokio` runtime, so it can call `poll` again in a later stage.
fn read_data<'a>(
    file: tokio::fs::File,
    status: Arc<RwLock<HashMap<&'a str, String>>>,
) -> impl Future<Item = (), Error = Error> + 'a {
    struct ReadData<'a> {
        file: tokio::fs::File,
        status: Arc<RwLock<HashMap<&'a str, String>>>,
        buf: Vec<u8>,
    };
    impl<'a> Future for ReadData<'a> {
        type Item = ();
        type Error = Error;
        fn poll(&mut self) -> Result<Async<()>, Error> {
            loop {
                let mut chunk = vec![0u8; 40];
                let bytes = match self.file.poll_read(&mut chunk) {
                    Ok(Async::Ready(n)) => n,
                    Ok(Async::NotReady) => return Ok(Async::NotReady),
                    Err(e) => return Err(e.into()),
                };
                if bytes == 0 {
                    break;
                }
                chunk.truncate(bytes);
                self.buf.append(&mut chunk);
                for packet in parse(&mut self.buf) {
                    decode(packet, &mut self.status.write().unwrap());
                }
            }
            Ok(Async::Ready(()))
        }
    }
    ReadData {
        file,
        status,
        buf: vec![],
    }
}

fn mqtt_publish(
    _m: &Mosquitto,
    _topic: &String,
    status: &HashMap<&str, String>,
) -> Result<(), std::io::Error> {
    println!("STATUS {:?}", status);
    // for (k,v) in status {
    // 	let t = format!("{}/{}", topic, k);
    // 	let _mid = m.publish(t.as_str(), v.as_bytes(), 2, false);
    // }
    Ok(())
}
