use std::mem::transmute;
use std::mem::size_of;
use std::net::UdpSocket;
use std::net::IpAddr;
use std::str::FromStr;
use std::net::SocketAddr;
use mac_address::MacAddress;
use std::net::Ipv4Addr;

#[derive(Copy, Clone)]
#[repr(C)]
struct BootPacket{
    _opcode: u8,
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
    BootPacket{
        _opcode: 0,
        _hwtype: 0,
        _hw_addr_len: 0,
        _hop_count: 0,
        _txn_id: [0; 4],
        _num_secs: [0; 2],
        _unused: [0;2],
        _client_ip: [0; 4],
        _your_ip: [0; 4],
        _server_ip: [0; 4],
        _gateway_ip: [0; 4],
        _client_mac: [0; 6],
        _client_mac_remainder: [0; 10],
        _server_host_name: [0; 64],
        _boot_file_name: [0; 128],
        _vendor_info: [0; 64]
    }
}


impl BootPacket {
    fn log(&self){
        println!("packet received");
        println!("opcode      = {0}", self._opcode);
        println!("hwtype      = {0}", self._hwtype);
        println!("hw addr len = {0}", self._hw_addr_len);
        println!("hop count   = {0}", self._hop_count);
        println!("txn_id      = {:x}", u32::from_be_bytes(self._txn_id));
        println!("num_secs    = {:}", u16::from_be_bytes(self._num_secs));
        println!("ips {0} {1} {2} {3}",
                 Ipv4Addr::from(self._client_ip),
                 Ipv4Addr::from(self._your_ip),
                 Ipv4Addr::from(self._server_ip),
                 Ipv4Addr::from(self._gateway_ip));
        println!("Mac Addr:   = {:}", MacAddress::new(self._client_mac));
    }
}



fn main() -> std::io::Result<()> {
    {

        println!("size of Boot Packet layout  = {0}",
                 size_of::<BootPacket>());

        let mut packet = alloc_boot_packet(); 
        let local_ip4 = IpAddr::from_str("0.0.0.0").unwrap();
        let listen4_port: u16  = 67;
        let socket = UdpSocket::bind(&SocketAddr::new(local_ip4, listen4_port))?;
        socket.set_broadcast(true).expect("set_broadcast call failed");
        // Receives a single datagram message on the socket. If `buf` is too small to hold
        // the message, it will be cut off.
        unsafe {
            let mut buf = transmute::<BootPacket,
                                      [u8; size_of::<BootPacket>()]>(packet);
            let (_amt, _src) = socket.recv_from(&mut buf)?;
            packet = transmute::<[u8; size_of::<BootPacket>()],
                                      BootPacket>(buf);
        }
        packet.log();

        //let mut response_packet = boot_packet;
        //response_packet.client_ip =  Ipv4Addr::new(192,168,144,100);
        //response_packet.opcode = 2;
        //response_packet.log_packet();

        //response_packet.to_bytes(&mut buf);
        //socket.send_to(&buf, &src)?;
    }

    Ok(())
}
