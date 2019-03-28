#[cfg(test)]
mod tests {
    use ppp_packet::Packet;

    use parse::parse as old_parse;
    use nom_parse::parse as new_parse;

    fn generic_test(
        parse: &Fn(&mut Vec<u8>)->Vec<Packet>,
        buffer: &mut Vec<u8>,
        output_packets: usize,
        unconsumed_bytes: usize
    ) {
        let packets = parse(buffer);
        assert_eq!(packets.len(), output_packets);
        assert_eq!(buffer.len(), unconsumed_bytes);
    }

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
    fn empty_buffer_yields_no_packets_old() {
        let mut buffer = Vec::<u8>::new();
        generic_test(&old_parse, &mut buffer, 0, 0);
    }

    #[test]
    fn empty_buffer_yields_no_packets_new() {
        let mut buffer = Vec::<u8>::new();
        generic_test(&new_parse, &mut buffer, 0, 0);
    }

    #[test]
    fn payloadless_packet_parses_correctly_old() {
        let mut buffer: Vec<u8> = SIMPLE_PACKET.to_vec();
        generic_test(&old_parse, &mut buffer, 1, 0);
    }

    #[test]
    fn payloadless_packet_parses_correctly_new() {
        let mut buffer: Vec<u8> = SIMPLE_PACKET.to_vec();
        generic_test(&new_parse, &mut buffer, 1, 0);
    }

    #[test]
    fn packet_missing_first_bytes_handled_correctly_old() {
        let mut buffer: Vec<u8> = SIMPLE_PACKET[2..].to_vec();
        let blen = buffer.len();
        generic_test(&old_parse, &mut buffer, 0, blen);
    }

    #[test]
    fn packet_missing_first_bytes_handled_correctly_new() {
        let mut buffer: Vec<u8> = SIMPLE_PACKET[2..].to_vec();
        let blen = buffer.len();
        generic_test(&new_parse, &mut buffer, 0, blen);
    }

    #[test]
    fn packet_not_complete_handled_correctly_old() {
        let mut buffer: Vec<u8> = SIMPLE_PACKET[..7].to_vec();
        let blen = buffer.len();
        generic_test(&old_parse, &mut buffer, 0, blen)
    }

    #[test]
    fn packet_not_complete_handled_correctly_new() {
        let mut buffer: Vec<u8> = SIMPLE_PACKET[..7].to_vec();
        let blen = buffer.len();
        generic_test(&new_parse, &mut buffer, 0, blen)
    }

    #[test]
    fn bytes_before_packet_get_dropped_old() {
        let mut buffer: Vec<u8> = [0, 0, 0, 0].to_vec();
        buffer.extend(SIMPLE_PACKET.to_vec());
        generic_test(&old_parse, &mut buffer, 1, 0);
    }

    #[test]
    fn bytes_before_packet_get_dropped_new() {
        let mut buffer: Vec<u8> = [0, 0, 0, 0].to_vec();
        buffer.extend(SIMPLE_PACKET.to_vec());
        generic_test(&new_parse, &mut buffer, 1, 0);
    }

    #[test]
    fn non_0x0b_mode_packets_are_dropped_old() {
        let mut buffer: Vec<u8> = SIMPLE_PACKET.to_vec();
        buffer[6] = 0x0c;
        generic_test(&old_parse, &mut buffer, 0, 0);
    }

    #[test]
    fn non_0x0b_mode_packets_are_dropped_new() {
        let mut buffer: Vec<u8> = SIMPLE_PACKET.to_vec();
        buffer[6] = 0x0c;
        generic_test(&new_parse, &mut buffer, 0, 0);
    }
}
