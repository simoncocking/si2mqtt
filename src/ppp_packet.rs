// PPP header definition
// begin addr ctrl  head  pad0 mode col row pad1        payload       cksm end
// 7e    ff   03    4243  01   0b   01  01  00 00 00 00 Some data 00  dead 7e
#[derive(Debug)]
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

// PPP packet definition
#[derive(Debug)]
pub struct Packet<'a> {
	pub header: &'a Header,
	pub payload: Vec<u8>,
	pub _checksum: u16,
}

