use std::mem::transmute;
use std::mem::size_of;
use std::str::FromStr;
use std::net::IpAddr;
use std::net::SocketAddr;
use std::net::UdpSocket;
use std::io::{Error, ErrorKind};
use clap::App;
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
    //last packet gets written.  We can and will do better.
    dhcp::DHCPPacket::write_to_file("/tmp/rustboot_packet.bin", packet);

    
    match dhcp::DHCPPacket::generate_response(&packet){
        Ok(response_packet)  => { 
            let dest = SocketAddr::from(
                (response_packet.your_ip, server_port));
            unsafe {
                let buf = transmute::<dhcp::DHCPPacket,
                                      [u8; size_of::<dhcp::DHCPPacket>()]>(
                    response_packet);
                socket.send_to(&buf, &dest)?;
            };
            Ok(())
        },
        Err(s) => Err(Error::new(ErrorKind::Other, s))
    }
}



fn run_server() -> std::io::Result<()> {

    
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

fn main() -> std::io::Result<()> {
    let matches = App::new("rustboot")
        .version("1.0")
        .author("Adamn Young<adam@younglogic.com>")
        .about("DHCP server")
        .subcommand(
            App::new("server")
                .about("runs the DHCP server.")
        )
        .get_matches();

    match matches.subcommand_name() {
        Some("server") => run_server(),
        None => {
            println!("No subcommand was used");
            Ok(())
        },
        _ => unreachable!(),
    }
    
}
