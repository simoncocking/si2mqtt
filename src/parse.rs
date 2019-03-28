use std::mem::size_of;

use ppp_packet::{Packet, Header};

pub fn parse(chunk: &mut Vec<u8>) -> Vec<Packet> {
	println!("Decoding {} bytes", chunk.len());
	for b in chunk.iter() {
		print!("{:00x} ", b);
	}
	println!("");
	let mut packets: Vec<Packet> = Vec::new();
	let mut iter: usize = 0;
	let mut packet_start: usize = 0;
	while chunk.len() > 0 && iter < chunk.len() - size_of::<Header>() {
		if chunk[iter] == 0x7e && chunk[iter+1] == 0xff {
			let header: *const u8 = chunk[iter..].as_ptr();
			let header: *const Header = header as *const Header;
			let header: &Header = unsafe { &*header };
			let payload = iter + size_of::<Header>();
			if header.mode == 0x0b {
				// This is a display update
				for null in payload .. chunk.len() - 3 { // 3 bytes allowance for checksum/terminator
					if chunk[null] == 0x00 {
						// End of payload
						packets.push(Packet {
							header: header.clone(),
							payload: chunk[payload .. null].to_vec(),
							_checksum: ((chunk[null+1] as u16) << 8) + (chunk[null+2] as u16),
						});
						iter = null + 3;
						packet_start = iter + 1;
						break;
					}
				}
			} else {
				// println!("{:?}", header);
			}
		}
		iter += 1;
	}
	println!("Returning {} unprocessed bytes", chunk.len() - packet_start);
	*chunk = chunk[packet_start..].to_vec();
	return packets;
}
