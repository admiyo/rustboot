use std::mem::transmute;
use std::mem::size_of;
use std::str::FromStr;
use std::net::IpAddr;
use std::net::SocketAddr;
use std::net::UdpSocket;
use std::io::{Error, ErrorKind};
use std::format;
use clap::{Clap};
use std::time::SystemTime;
use std::fs;
mod dhcp;
extern crate num;
extern crate num_derive;

struct DHCPServer{
    logging: bool,
    capture: bool,
    capture_dir: String,
    server_port: u16,
    socket: UdpSocket
}

impl DHCPServer{
    fn new(logging: bool, capture: bool, capture_dir: &str) -> Result <DHCPServer, Error>  {

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
    let mut packet = dhcp::DHCPPacket::new();
    unsafe {
        let mut buf = transmute::<
                dhcp::DHCPPacket,
            [u8; size_of::<dhcp::DHCPPacket>()]>(packet);
        let (_amt, _src) = self.socket.recv_from(&mut buf)?;
        packet = transmute::<[u8; size_of::<dhcp::DHCPPacket>()],
                             dhcp::DHCPPacket>(buf);
    }
    if self.logging {
        println!("packet received");
        packet.log();
    }

    if self.capture{
        let date_time = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap();
        let capture_file = format!("{}/packet.{:?}.in.bin", self.capture_dir, date_time);
        dhcp::DHCPPacket::write_to_file(&capture_file, packet);
    }
    match dhcp::DHCPPacket::generate_response(&packet){
        Ok(response_packet)  => {
            if self.logging {
                println!("sending packet");
                response_packet.log();
            }

            if self.capture{
                let date_time = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap();
                let capture_file = format!("{}/packet.{:?}.out.bin", self.capture_dir, date_time);
                dhcp::DHCPPacket::write_to_file(&capture_file, packet);
            }

            let dest = SocketAddr::from(
                (response_packet.your_ip, self.server_port));
            unsafe {
                let buf = transmute::<dhcp::DHCPPacket,
                                      [u8; size_of::<dhcp::DHCPPacket>()]>(
                    response_packet);
                self.socket.send_to(&buf, &dest)?;
            };
            Ok(())
        },
        Err(s) => Err(Error::new(ErrorKind::Other, s))
    }
}

fn run(&self) -> std::io::Result<()> {

    if self.capture{
        fs::create_dir_all(&self.capture_dir)?;
    }

    println!("size of Boot Packet layout  = {0}",
             size_of::<dhcp::DHCPPacket>());
    loop {
        self.handle_packet()?
    }
}


}
/// This doc string acts as a help message when the user runs '--help'
/// as do all doc strings on fields
#[derive(Clap)]
#[clap(version = "1.0", author = "Adam Young <adam@younglogic.com>")]
struct Opts {
    /// Sets a custom config file. Could have been an Option<T> with no default too
    #[clap(short, long, default_value = "default.conf")]
    config: String,

    /// Directory where to write captured packets
    #[clap(short, long, default_value = "/tmp/rustboot/")]
    packet_capture_dir: String,


    /// A level of verbosity, and can be used multiple times
    #[clap(short, long, parse(from_occurrences))]
    verbose: i32,

    /// If the server should write captured packets to disk
    #[clap(short)]
    write_capture: bool,

}


fn main() -> std::io::Result<()> {

    let opts: Opts = Opts::parse();

    // Gets a value for config if supplied by user, or defaults to "default.conf"
    println!("Value for config: {}", opts.config);


    let server = DHCPServer::new( opts.verbose > 0, opts.write_capture, &opts.packet_capture_dir )?;
    server.run()?;
    Ok(())
}
