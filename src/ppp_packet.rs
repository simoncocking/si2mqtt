// PPP header definition
// begin addr ctrl  head  pad0 mode col row pad1        payload       cksm end
// 7e    ff   03    4243  01   0b   01  01  00 00 00 00 Some data 00  dead 7e
#[derive(Debug, Copy, Clone)]
#[repr(C, packed)]
pub struct Header {
	begin: u8,
	addr: u8,
	ctrl: u8,
	head: u16,
	pad0: u8,
	pub mode: u8,
	col: u8,
	pub row: u8,
	pad1: u32,
}

impl Header {
	pub fn new(mode: u8, row: u8) -> Self {
		Header {
			begin: 0,
			addr: 0,
			ctrl: 0,
			head: 0,
			pad0: 0,
			mode,
			col: 0,
			row,
			pad1: 0
		}
	}
}

// PPP packet definition
#[derive(Debug)]
pub struct Packet {
	pub header: Header,
	pub payload: Vec<u8>,
	pub _checksum: u16,
}

impl Packet {
	pub fn new(mode: u8, row: u8, payload: Vec<u8>, _checksum: u16) -> Packet {
		Packet {
			header: Header::new(mode, row), payload, _checksum
		}
	}
}