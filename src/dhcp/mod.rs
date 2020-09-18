use std::mem::transmute;
use std::mem::size_of;
use std::str::FromStr;
use std::net::IpAddr;
use std::net::SocketAddr;
use std::net::UdpSocket;
use std::io::{Error, ErrorKind};
use std::format;
use std::time::SystemTime;
use std::fs;
mod packet;
extern crate num;
extern crate num_derive;

pub struct DHCPServer{
    logging: bool,
    capture: bool,
    capture_dir: String,
    server_port: u16,
    socket: UdpSocket
}

impl DHCPServer{
    pub fn new(logging: bool, capture: bool, capture_dir: &str) -> Result <DHCPServer, Error>  {

        let local_ip4 = IpAddr::from_str("0.0.0.0").unwrap();
        let server_port: u16  = 67;
        let socket = UdpSocket::bind(&SocketAddr::new(local_ip4, server_port))?;
        socket.set_broadcast(true).expect("set_broadcast call failed");

        return Ok(DHCPServer{
            capture: capture,
            capture_dir: String::from_str(capture_dir).unwrap(),
            logging: logging,
            server_port: 67,
            socket: socket
        });
    }

    fn handle_packet(&self) ->
    std::io::Result<()>
{
    let mut packet = packet::DHCPPacket::new();
    unsafe {
        let mut buf = transmute::<
                packet::DHCPPacket,
            [u8; size_of::<packet::DHCPPacket>()]>(packet);
        let (_amt, _src) = self.socket.recv_from(&mut buf)?;
        packet = transmute::<[u8; size_of::<packet::DHCPPacket>()],
                             packet::DHCPPacket>(buf);
    }
    if self.logging {
        println!("packet received");
        packet.log();
    }

    if self.capture{
        let date_time = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap();
        let capture_file = format!("{}/packet.{:?}.in.bin", self.capture_dir, date_time);
        packet::DHCPPacket::write_to_file(&capture_file, packet);
    }
    match packet::DHCPPacket::generate_response(&packet){
        Ok(response_packet)  => {
            if self.logging {
                println!("sending packet");
                response_packet.log();
            }

            if self.capture{
                let date_time = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap();
                let capture_file = format!("{}/packet.{:?}.out.bin", self.capture_dir, date_time);
                packet::DHCPPacket::write_to_file(&capture_file, packet);
            }

            let dest = SocketAddr::from(
                (response_packet.your_ip, self.server_port));
            unsafe {
                let buf = transmute::<packet::DHCPPacket,
                                      [u8; size_of::<packet::DHCPPacket>()]>(
                    response_packet);
                self.socket.send_to(&buf, &dest)?;
            };
            Ok(())
        },
        Err(s) => Err(Error::new(ErrorKind::Other, s))
    }
}

pub fn run(&self) -> std::io::Result<()> {

    if self.capture{
        fs::create_dir_all(&self.capture_dir)?;
    }

    println!("size of Boot Packet layout  = {0}",
             size_of::<packet::DHCPPacket>());
    loop {
        self.handle_packet()?
    }
}
}
