use std::mem::transmute;
use std::mem::size_of;
use mac_address::MacAddress;
use std::net::Ipv4Addr;


#[derive(Copy, Clone)]
#[repr(C)]
struct BootPacket{
    opcode: u8,
    _hwtype: u8,
    _hw_addr_len: u8,
    _hop_count: u8,
    _txn_id: [u8; 4],
    _num_secs: [u8; 2],
    _unused: [u8;2],
    _client_ip: [u8; 4],
    _your_ip: [u8; 4],
    _server_ip: [u8; 4],
    _gateway_ip: [u8; 4],
    _client_mac: [u8; 6],
    _client_mac_remainder: [u8; 10],
    _server_host_name: [u8; 64],
    _boot_file_name: [u8; 128],
    _vendor_info: [u8; 64]
}

fn alloc_boot_packet() -> BootPacket{
    let buf: [u8; size_of::<BootPacket>()] = [0; size_of::<BootPacket>()];
    unsafe {
         transmute::<[u8; size_of::<BootPacket>()],BootPacket>(buf)
    }
}


impl BootPacket {
    fn log(&self){
        println!("----------------------------------------------------");
        println!("opcode      = {0}", self.opcode);
        println!("hwtype      = {0}", self._hwtype);
        println!("hw addr len = {0}", self._hw_addr_len);
        println!("hop count   = {0}", self._hop_count);
        println!("txn_id      = {:x}", u32::from_be_bytes(self._txn_id));
        println!("num_secs    = {:}", u16::from_be_bytes(self._num_secs));
        println!("client_ip   = {0} ", Ipv4Addr::from(self._client_ip));
        println!("your_ip     = {0} ", Ipv4Addr::from(self._your_ip));
        println!("server_ip   = {0} ", Ipv4Addr::from(self._server_ip));
        println!("gateway_ip  = {0} ", Ipv4Addr::from(self._gateway_ip));
        println!("Mac Addr:   = {:}", MacAddress::new(self._client_mac));
    }
}

