use std::mem::transmute;
use std::mem::size_of;
use std::net::UdpSocket;
use std::str::FromStr;
use std::net::SocketAddr;
use std::net::IpAddr;

mod bootpacket;


fn handle_packet(server_port: u16, socket: &UdpSocket) ->
    std::io::Result<()>
{
    let mut packet = bootpacket::alloc_boot_packet();

    unsafe {
        let mut buf = transmute::<
                bootpacket::BootPacket,[u8; size_of::<bootpacket::BootPacket>()]>(packet);
        let (_amt, _src) = socket.recv_from(&mut buf)?;
        
        packet = transmute::<[u8; size_of::<bootpacket::BootPacket>()],bootpacket::BootPacket>(buf);
    }
    println!("packet received");
    packet.log();

    let response_packet = bootpacket::generate_response(packet);

    println!("sending packet");
    response_packet.log();

    let dest = SocketAddr::from(
        (response_packet.your_ip, server_port));
    unsafe {
        let buf = transmute::<bootpacket::BootPacket,[u8; size_of::<bootpacket::BootPacket>()]>(
            response_packet);
        socket.send_to(&buf, &dest)?;
    };
    Ok(())
}

fn main() -> std::io::Result<()> {
    {
        println!("size of Boot Packet layout  = {0}",
                 size_of::<bootpacket::BootPacket>());
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
