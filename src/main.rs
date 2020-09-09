use std::net::UdpSocket;
use std::net::IpAddr;
use std::str::FromStr;
use std::net::SocketAddr;
use mac_address::MacAddress;
use std::convert::TryInto;
use std::net::Ipv4Addr;

struct BootPacket{
    opcode: u8,
    hwtype: u8,
    hw_addr_len: u8,
    hop_count: u8,
    txn_id: u32,
    num_secs: u16,
    _unused: u16,
    client_ip: Ipv4Addr,
    your_ip: Ipv4Addr,
    server_ip: Ipv4Addr,
    gateway_ip: Ipv4Addr,
    client_mac: MacAddress ,
}

fn build_boot_packet(buf: &[u8]) -> BootPacket{
    let boot_packet = BootPacket{
        opcode: buf[0],
        hwtype: buf[1],
        hw_addr_len: buf[2],
        hop_count: buf[3],
        txn_id: u32::from_be_bytes(
            buf[4  .. 8].try_into().
                expect("txn_id slice with incorrect length")),
        num_secs:   u16::from_be_bytes(
            [buf[8], buf[9]]),
        _unused: u16::from_be_bytes(
            [buf[10], buf[11]]),
        client_ip:  Ipv4Addr::new(buf[12], buf[13], buf[14], buf[15]),
        your_ip:    Ipv4Addr::new(buf[16], buf[17], buf[18], buf[19]),
        server_ip:  Ipv4Addr::new(buf[20], buf[21], buf[22], buf[23]),
        gateway_ip: Ipv4Addr::new(buf[24], buf[25], buf[26], buf[27]),
        client_mac: MacAddress::new(
            buf[28  .. 34].try_into().
                expect("MacAddress slice with incorrect length"))
    };
    boot_packet

}

impl BootPacket {
    fn log_packet(&self){
        println!("packet received");
        println!("opcode      = {0}", self.opcode);
        println!("hwtype      = {0}", self.hwtype);
        println!("hw addr len = {0}", self.hw_addr_len);
        println!("hop count   = {0}", self.hop_count);
        println!("txn_id      = {:x}", self.txn_id);
        println!("num_secs    = {:}", self.num_secs);
        println!("ips {0} {1} {2} {3}",
                 self.client_ip, self.your_ip,
                 self.server_ip,  self.gateway_ip);
        println!("Mac Addr:   = {:}", self.client_mac);
    }
}
fn main() -> std::io::Result<()> {
    {
        let local_ip4 = IpAddr::from_str("0.0.0.0").unwrap();
        let listen4_port: u16  = 67;
        let socket = UdpSocket::bind(&SocketAddr::new(local_ip4, listen4_port))?;
        socket.set_broadcast(true).expect("set_broadcast call failed");
        // Receives a single datagram message on the socket. If `buf` is too small to hold
        // the message, it will be cut off.
        let mut buf = [0; 300];

        let (_amt, _src) = socket.recv_from(&mut buf)?;
        let boot_packet = build_boot_packet(&buf);

        boot_packet.log_packet();
    }

    Ok(())
}
