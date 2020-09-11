use std::mem::transmute;
use std::mem::size_of;
use mac_address::MacAddress;
use std::net::Ipv4Addr;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct BootPacket{
    opcode: u8,
    _hwtype: u8,
    _hw_addr_len: u8,
    _hop_count: u8,
    _txn_id: [u8; 4],
    _num_secs: [u8; 2],
    _unused: [u8;2],
    _client_ip: [u8; 4],
    pub your_ip: [u8; 4],
    _server_ip: [u8; 4],
    _gateway_ip: [u8; 4],
    _client_mac: [u8; 6],
    _client_mac_remainder: [u8; 10],
    _server_host_name: [u8; 64],
    _boot_file_name: [u8; 128],
    _vendor_info: [u8; 64]
}

pub fn alloc_boot_packet() -> BootPacket{
    let buf: [u8; size_of::<BootPacket>()] = [0; size_of::<BootPacket>()];
    unsafe {
         transmute::<[u8; size_of::<BootPacket>()],BootPacket>(buf)
    }
}


impl BootPacket {

    pub fn client_mac(&self) ->  MacAddress{
        MacAddress::new(self._client_mac)
    }
    
    pub fn log(&self){
        println!("----------------------------------------------------");
        println!("opcode      = {0}", self.opcode);
        println!("hwtype      = {0}", self._hwtype);
        println!("hw addr len = {0}", self._hw_addr_len);
        println!("hop count   = {0}", self._hop_count);
        println!("txn_id      = {:x}", u32::from_be_bytes(self._txn_id));
        println!("num_secs    = {:}", u16::from_be_bytes(self._num_secs));
        println!("client_ip   = {0} ", Ipv4Addr::from(self._client_ip));
        println!("your_ip     = {0} ", Ipv4Addr::from(self.your_ip));
        println!("server_ip   = {0} ", Ipv4Addr::from(self._server_ip));
        println!("gateway_ip  = {0} ", Ipv4Addr::from(self._gateway_ip));
        println!("Mac Addr:   = {:}", self.client_mac());
    }
}


/*
To write a packet to a file
        {
            use std::io::prelude::*;
            use std::fs::File;
            let mut pos = 0;
            let mut buffer = File::create("/tmp/foo.txt")?;
            while pos < buf.len() {
                let bytes_written = buffer.write(&buf[pos..])?;
                pos += bytes_written;
            }
        }

*/

pub fn generate_response(packet: BootPacket) ->  BootPacket
{
    let mut response_packet = packet;

    let server_hostname = "ayoungP40";
    response_packet._server_host_name[0..server_hostname.len()].
        copy_from_slice(server_hostname.as_bytes());

    response_packet._server_ip =  Ipv4Addr::new(192,168,144,1).octets();
    response_packet.your_ip =  Ipv4Addr::new(192,168,144,100).octets();
    response_packet.opcode = 2;
    response_packet
}

#[cfg(test)]
mod tests {
    use std::env;
    use std::fs::File;
    use std::io::prelude::*;
    use super::*;
    use std::convert::TryFrom;


    fn read_packet() ->  BootPacket{
        let cargo_manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
        let filename = format!("{}/boot-packet.bin",
                               cargo_manifest_dir);
        
        let f = File::open(filename).unwrap();
        let take_size = u64::try_from(size_of::<BootPacket>()).unwrap();
        let mut handle = f.take( take_size );
        let mut buffer: [u8; size_of::<BootPacket>()] = [0; size_of::<BootPacket>()];
        handle.read(&mut buffer).unwrap();

        let packet = 
        unsafe {
                transmute::<[u8; size_of::<BootPacket>()],BootPacket>(buffer)
        };
        packet
    }
    

    #[test]
    fn test_create_packet() {
        let packet = alloc_boot_packet();
        assert_eq!(0, packet.opcode);
    }

    #[test]
    fn test_parse_packet() {
        let packet = read_packet();
        assert_eq!(1, packet.opcode);

        
        assert_eq!(1, packet.opcode);
        let test_mac = MacAddress::new([0x52,0x54,0x00,0xE6,0x08,0x031]);
        assert_eq!(test_mac, packet.client_mac());
        
    }
}

