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

#[cfg(test)]
mod tests {
	pub use super::parse;

	static SIMPLE_PACKET: &[u8] = &[
		0x7e,                       // begin
		0xff,                       // addr
		0x03,                       // ctrl
		0x42, 0x43,                 // head
		0x01,                       // pad0
		0x0b,                       // mode
		0x01,                       // col
		0x01,                       // row
		0x00, 0x00, 0x00, 0x00,     // pad1
		0x00,                       // payload (null)
		0xde, 0xad,                 // cksm
		0x7e                        // end
	];


    #[test]
    fn empty_vector_yields_no_packets() {
        let mut empty_vec: Vec<u8> = Vec::new();
        let packets = parse(&mut empty_vec);
        assert_eq!(packets.len(), 0);
    }

    #[test]
    fn payloadless_packet_parses_correctly() {
        let mut simple_packet: Vec<u8> = SIMPLE_PACKET.to_vec();
        
        // our vector of received data should contain data
        assert!(simple_packet.len() > 0);

        {
            let packets = parse(&mut simple_packet);
            assert_eq!(packets.len(), 1);

            let packet = &packets[0];
            assert_eq!(packet.header.row, 1);
            assert_eq!(packet.payload.len(), 0);
        }

        // our vector of received data should have been mutated by the parse
        // method, and should now be empty as the parser consumed the entire
        // chunk
        assert_eq!(simple_packet.len(), 0);
    }

	#[test]
	fn packet_missing_first_bytes_handled_correctly() {
		// Missed first 2 bytes
		let mut partial_packet: Vec<u8> = SIMPLE_PACKET[2..].to_vec();

		// our vector of received data should contain data
		assert!(partial_packet.len() > 0);

		{
			let packets = parse(&mut partial_packet);
			assert_eq!(packets.len(), 0);
		}

		// no change observed
		assert!(partial_packet.len() > 0);
	}

	#[test]
	#[ignore]
	fn packet_not_complete_handled_correctly() {
		// Missed last 10 bytes
		let mut partial_packet: Vec<u8> = SIMPLE_PACKET[..7].to_vec();

		// our vector of received data should contain data
		assert!(partial_packet.len() > 0);

		{
			let packets = parse(&mut partial_packet);
			assert_eq!(packets.len(), 0);
		}

		// no change observed
		assert!(partial_packet.len() > 0);

	}
}
