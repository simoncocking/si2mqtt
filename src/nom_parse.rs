use nom::{le_u8, le_u16};

use ppp_packet::{Packet};


pub fn not_null(chr: u8) -> bool {
    chr != 0x00u8
}


named!(nom_parse<&[u8], Packet>,
    do_parse!(
        take_until!("\x7e")                     >> // drop unrecognised data
        tag!(b"\x7e\xff")                       >> // begin packet
        take!(4)                                >> // addr, ctrl, head (all discarded)
        mode: le_u8                             >> // mode
        take!(1)                                >> // col
        row: le_u8                              >> // row
        take!(4)                                >> // pad1
        payload: take_while!(not_null)          >> // payload
        take!(1)                                >> // payload-end
        cksm: le_u16                            >> // cksm
        tag!("\x7e")                            >>
        (
            Packet::new(mode, row, payload.to_vec(), cksm)
        )
    )
);

pub fn parse(chunk: &mut Vec<u8>) -> Vec<Packet> {
    let mut packets = Vec::<Packet>::new();
    let temp = chunk.clone();
    let mut remaining = temp.as_slice();
    loop {
        let result = nom_parse(remaining);
        match result {
            Ok((i, o)) => {
                if o.header.mode == 0x0b {
                    packets.push(o);
                }
                remaining = i;
            }
            _ => {
                break;
            }
        }
    }
    *chunk = remaining.to_vec();
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
        let mut buffer: Vec<u8> = SIMPLE_PACKET.to_vec();

        // our vector of received data should contain data
        assert!(buffer.len() > 0);

        {
            let packets = parse(&mut buffer);
            assert_eq!(packets.len(), 1);

            let packet = &packets[0];
            assert_eq!(packet.header.row, 1);
            assert_eq!(packet.payload.len(), 0);
        }

        // our vector of received data should have been mutated by the parse
        // method, and should now be empty as the parser consumed the entire
        // chunk
        assert_eq!(buffer.len(), 0);
    }

    #[test]
    fn packet_missing_first_bytes_handled_correctly() {
        // Missed first 2 bytes
        let mut buffer: Vec<u8> = SIMPLE_PACKET[2..].to_vec();

        // our vector of received data should contain data
        assert!(buffer.len() > 0);

        {
            let packets = parse(&mut buffer);
            assert_eq!(packets.len(), 0);
        }

        // no change observed
        assert!(buffer.len() > 0);
    }

    #[test]
    fn packet_not_complete_handled_correctly() {
        // Missed last 10 bytes
        let mut buffer: Vec<u8> = SIMPLE_PACKET[..7].to_vec();

        // our vector of received data should contain data
        assert!(buffer.len() > 0);

        {
            let packets = parse(&mut buffer);
            assert_eq!(packets.len(), 0);
        }

        // no change observed
        assert!(buffer.len() > 0);

    }

    #[test]
    fn bytes_before_packet_get_dropped() {
        // Extraneous data + packet
        let mut buffer: Vec<u8> = [0, 0, 0, 0].to_vec();
        buffer.extend(SIMPLE_PACKET.to_vec());

        // our vector of received data should contain data
        assert_eq!(buffer.len(), 4 + SIMPLE_PACKET.len());

        {
            let packets = parse(&mut buffer);
            assert_eq!(packets.len(), 1);
        }

        // extraneous data + packet data all consumed
        assert_eq!(buffer.len(), 0);
    }

    #[test]
    fn non_0x0b_mode_packets_are_dropped() {
        let mut buffer: Vec<u8> = SIMPLE_PACKET.to_vec();
        buffer[6] = 0x0c;

        // our vector of received data should contain data
        assert!(buffer.len() > 0);

        {
            let packets = parse(&mut buffer);
            assert_eq!(packets.len(), 0);

        }

        // bytes should still have been consumed
        assert_eq!(buffer.len(), 0);
    }
}