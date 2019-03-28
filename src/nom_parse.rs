use nom::{le_u8, le_u16};

use ppp_packet::Packet;

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