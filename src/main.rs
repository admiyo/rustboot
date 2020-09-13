use std::mem::transmute;
use std::mem::size_of;
use std::net::UdpSocket;
use std::str::FromStr;
use std::net::SocketAddr;
use std::net::IpAddr;

mod dhcp;


fn handle_packet(server_port: u16, socket: &UdpSocket) ->
    std::io::Result<()>
{
    let mut packet = dhcp::DHCPPacket::new();
    unsafe {
        let mut buf = transmute::<
                dhcp::DHCPPacket,
            [u8; size_of::<dhcp::DHCPPacket>()]>(packet);
        let (_amt, _src) = socket.recv_from(&mut buf)?;
        packet = transmute::<[u8; size_of::<dhcp::DHCPPacket>()],
                             dhcp::DHCPPacket>(buf);
    }
    println!("packet received");
    packet.log();
    let response_packet = dhcp::generate_response(packet);
    println!("sending packet");
    response_packet.log();
    let dest = SocketAddr::from(
        (response_packet.your_ip, server_port));
    unsafe {
        let buf = transmute::<dhcp::DHCPPacket,[u8; size_of::<dhcp::DHCPPacket>()]>(
            response_packet);
        socket.send_to(&buf, &dest)?;
    };
    Ok(())
}

fn main() -> std::io::Result<()> {
    {
        println!("size of Boot Packet layout  = {0}",
                 size_of::<dhcp::DHCPPacket>());
        let local_ip4 = IpAddr::from_str("0.0.0.0").unwrap();
        let server_port: u16  = 67;
        let socket = UdpSocket::bind(&SocketAddr::new(local_ip4, server_port))?;
        socket.set_broadcast(true).expect("set_broadcast call failed");
        loop {
            handle_packet(server_port, &socket)?
        }
    }
    //Ok(())
}
