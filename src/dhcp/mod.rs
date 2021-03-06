use std::mem::transmute;
use std::mem::size_of;
use std::str::FromStr;
use std::net::IpAddr;
use std::net::Ipv4Addr;

use std::net::SocketAddr;
use std::net::UdpSocket;
use std::io::{Error, ErrorKind};
use std::format;
use std::time::SystemTime;
use std::fs;
use mac_address::MacAddress;

mod packet;
use packet::DHCPOptCodes;
use packet::DHCPOptionCode;
use packet::DHCPMessageType;
use packet::DHCPPacket;
use packet::VENDOR_MAGIC;
use packet::VendorData;


extern crate num;
extern crate num_derive;

pub struct MachineConfig{
    pub mac_address: MacAddress
}

impl MachineConfig{

    pub fn server_ip(&self) -> Ipv4Addr{
        Ipv4Addr::new(192,168,144,1)
    }

    pub fn your_ip(&self) -> Ipv4Addr{
        Ipv4Addr::new(192,168,144,100)
    }

    pub fn subnet_mask(&self) -> Ipv4Addr {
        Ipv4Addr::new(255,255,255,0)
    }

    pub fn router(&self) -> Ipv4Addr {
        Ipv4Addr::new(192,168,144,1)
    }

    pub fn lease_time(&self) -> u32 {
         86400
    }

    pub fn dhcp_server(&self) -> Ipv4Addr {
        Ipv4Addr::new(192,168,144,1)
    }

    pub fn boot_file_name(&self) -> String {
        "pxelinux/pxelinux.0".to_string()
    }
    pub fn domain_search(&self) -> String {
        "younglogic.net".to_string()
    }


    fn dns_servers(&self) -> Vec<u8> {
        let _dns_servers : Vec<Ipv4Addr> = vec![
            Ipv4Addr::new(75,75,75,75),Ipv4Addr::new(75,75,75,76),Ipv4Addr::new(8,8,8,8)]; 
        let mut addr_buf = vec![];

        for addr in _dns_servers {
            for b in addr.octets().to_vec() {
                addr_buf.push(b)
            }
        }
        addr_buf
    }

}

pub struct DHCPServer{
    logging: bool,
    local_ipv4: IpAddr,
    capture: bool,
    capture_dir: String,
    server_port: u16,
}

impl DHCPServer{

    pub fn machine_config(&self, mac: &MacAddress ) -> MachineConfig{
        return MachineConfig{
            mac_address: *mac
        }
    }

    pub fn new(logging: bool, capture: bool, capture_dir: &str) -> Result <DHCPServer, Error>  {
        let local_ip4 = IpAddr::from_str("0.0.0.0").unwrap();
        return Ok(DHCPServer{
            capture: capture,
            capture_dir: String::from_str(capture_dir).unwrap(),
            local_ipv4: local_ip4,
            logging: logging,
            server_port: 67,
        });
    }

    fn handle_packet(&self, socket: &UdpSocket) ->
        std::io::Result<()>
    {
        let mut packet = DHCPPacket::new();
        unsafe {
            let mut buf = transmute::<
                    DHCPPacket,
                [u8; size_of::<DHCPPacket>()]>(packet);
            let (_amt, _src) = socket.recv_from(&mut buf)?;
            packet = transmute::<[u8; size_of::<DHCPPacket>()],
                                 DHCPPacket>(buf);
        }
        if self.logging {
            println!("packet received");
            packet.log();
        }

        if self.capture{
            let date_time = SystemTime::now().duration_since(
                SystemTime::UNIX_EPOCH).unwrap();
            let capture_file = format!(
                "{}/packet.{:?}.in.bin", self.capture_dir, date_time);
            DHCPPacket::write_to_file(&capture_file, packet);
        }
        match self.generate_response(&packet){
            Ok(response_packet)  => {
                if self.logging {
                    println!("sending packet");
                    response_packet.log();
                }
                if self.capture{
                    let date_time = SystemTime::now().duration_since(
                        SystemTime::UNIX_EPOCH).unwrap();
                    let capture_file = format!(
                        "{}/packet.{:?}.out.bin", self.capture_dir, date_time);
                    DHCPPacket::write_to_file(&capture_file, packet);
                }

                let dest = SocketAddr::from(
                    (response_packet.your_ip, self.server_port));
                unsafe {
                    let buf = transmute::<
                            DHCPPacket,[
                                u8; size_of::<DHCPPacket>()]>(
                        response_packet);
                    socket.send_to(&buf, &dest)?;
                };
                Ok(())
            },
            Err(s) => Err(Error::new(ErrorKind::Other, s))
        }
    }

    pub fn run(&self) -> std::io::Result<()> {
        let socket = UdpSocket::bind(&SocketAddr::new(self.local_ipv4, self.server_port))?;
        socket.set_broadcast(true).expect("set_broadcast call failed");

        if self.capture{
            fs::create_dir_all(&self.capture_dir)?;
        }
        println!("size of Boot Packet layout  = {0}",
                 size_of::<DHCPPacket>());
        loop {
            self.handle_packet(&socket)?
        }
    }
    fn set_common_fields(&self, request_packet: &DHCPPacket, response_packet:  &mut DHCPPacket){
        let server_hostname = "ayoungP40";
        response_packet._server_host_name[0..server_hostname.len()].
            copy_from_slice(server_hostname.as_bytes());
        response_packet._vendor_magic = VENDOR_MAGIC;
        response_packet.opcode = DHCPOptCodes::RESPONSE as u8;
        response_packet._hwtype = request_packet._hwtype;
        response_packet._hw_addr_len =  request_packet._hw_addr_len;
        response_packet._client_mac =  request_packet._client_mac;
        response_packet._txn_id =  request_packet._txn_id;
        let config = self.machine_config(&request_packet.client_mac());
        response_packet._server_ip =  config.server_ip().octets();
        response_packet.your_ip =  config.your_ip().octets();

    }

    fn handle_dhcprequest(&self, request_packet: &DHCPPacket) ->  Result<DHCPPacket, &'static str>{
        let mut response_packet = DHCPPacket::new();
        self.set_common_fields(request_packet, &mut response_packet);

        let config = self.machine_config(&request_packet.client_mac());


        let mut vendor_data:Vec::<VendorData> = vec!();

        vendor_data.push(VendorData::new(DHCPOptionCode::DHCPMessageType,
             &vec![DHCPMessageType::DHCPACK as u8])?);

        vendor_data.push(VendorData::new(DHCPOptionCode::SubnetMask,
            &config.subnet_mask().octets().to_vec())?);

        vendor_data.push(VendorData::new(DHCPOptionCode::Router,
            &config.router().octets().to_vec())?);

        vendor_data.push(VendorData::new(DHCPOptionCode::IPAddressLeaseTime,
            &u32::to_be_bytes(config.lease_time()).to_vec())?);

        vendor_data.push(VendorData::new(DHCPOptionCode::DHCPServer,
            &config.dhcp_server().octets().to_vec())?);

        vendor_data.push(VendorData::new(DHCPOptionCode::DNSServers,
            &config.dns_servers())?);
        vendor_data.push(VendorData::END);


        let mut offset = 0;
             for opt in vendor_data{
                 offset = opt.write(&mut response_packet._vendor_info, offset)
             }

        Ok(response_packet)
    }

    fn handle_dhcpdiscover(&self, request_packet: &DHCPPacket) ->  Result<DHCPPacket, &'static str>{
        let mut response_packet = DHCPPacket::new();

        self.set_common_fields(request_packet, &mut response_packet);

        let config = self.machine_config(&request_packet.client_mac());

        // if the string is too long for the field, this will panic.
        let _boot_file_name = config.boot_file_name();
        for i in 0.._boot_file_name.len() {
            response_packet._boot_file_name[i] =  _boot_file_name.as_bytes()[i]
        }
        let mut vendor_data:Vec::<VendorData> = vec!();

        vendor_data.push(VendorData::new(DHCPOptionCode::DHCPMessageType,
             &vec![DHCPMessageType::DHCPOFFER as u8])?);
        vendor_data.push(VendorData::new(DHCPOptionCode::DomainSearch,
            &config.domain_search().as_bytes().to_vec())?);
        vendor_data.push(VendorData::END);

        let mut offset = 0;
        for opt in vendor_data{
            offset = opt.write(&mut response_packet._vendor_info, offset)
        }

        Ok(response_packet)
    }

    pub fn generate_response(&self, request_packet: &DHCPPacket) ->
        Result<DHCPPacket, &'static str>
    {
        if request_packet.vendor_magic() != VENDOR_MAGIC{
            return Err("Bad Vendor magic value");
        }
        let options = match request_packet.parse_vendor_data() {
            Ok(options) => options,
            Err(s) => {
                println!("Bad Packet: {}", s);
                return Err(s)
            }
        };
        DHCPPacket::dump_options(&options);
        match options.get(&DHCPOptionCode::DHCPMessageType){
            Some(option) => {
                if option.len != 1 {
                    return Err("malformed DHCPMessageType option");
                }
                let message_type: DHCPMessageType = match num::FromPrimitive::from_u8(option.data[0]){
                    Some(message_type) => message_type,
                    None =>  return Err("unknown message type")
                };
                match message_type{
                    DHCPMessageType::DHCPDISCOVER => self.handle_dhcpdiscover(request_packet),
                    DHCPMessageType::DHCPREQUEST => self.handle_dhcprequest(request_packet),
                    _ => return Err("cannot handle request for type")
                }
            },
            None =>  return Err("unknown message type")
        }
    }
}


#[cfg(test)]
mod tests {
    use std::env;
    use std::fs::File;
    use std::io::prelude::*;
    use super::*;
    use std::convert::TryFrom;

    fn make_test_server() -> DHCPServer{
         DHCPServer::new(false, false, "").unwrap()
    }

    fn read_discovery_packet() ->  DHCPPacket{
        let cargo_manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
        let filename = format!("{}/boot-packet.bin",
                               cargo_manifest_dir);

        let f = File::open(filename).unwrap();
        let take_size = u64::try_from(size_of::<DHCPPacket>()).unwrap();
        let mut handle = f.take( take_size );
        let mut buffer: [u8; size_of::<DHCPPacket>()] = [
            0; size_of::<DHCPPacket>()];
        handle.read(&mut buffer).unwrap();

        let packet =
            unsafe {
                transmute::<[u8; size_of::<DHCPPacket>()],DHCPPacket>(buffer)
            };
        packet
    }


    #[test]
    fn test_handle_discover(){
        let server = make_test_server();
        let response_packet = server.handle_dhcpdiscover(&read_discovery_packet()).unwrap();
        assert_eq!(response_packet.opcode, DHCPMessageType::DHCPOFFER as u8);
        let vendor_data = response_packet.parse_vendor_data().unwrap();
        assert_eq!(vendor_data.len(), 2);
        match vendor_data.get(&DHCPOptionCode::DHCPMessageType){
            Some(option) =>  {
                assert_eq!(DHCPOptionCode::DHCPMessageType as u8, option.code);
                assert_eq!(1,  option.len);
                assert_eq!(vec![DHCPMessageType::DHCPOFFER as u8],  option.data);
            },
            None => {
                assert!(false, "Vendor data is mising Message Type Value");
            }
        }
    }

    #[test]
    fn test_handle_dhcprequest(){
        let server = make_test_server();

        let response_packet = server.handle_dhcprequest(&read_discovery_packet()).unwrap();
        assert_eq!(response_packet.opcode, DHCPOptCodes::RESPONSE as u8);
        let vendor_data = response_packet.parse_vendor_data().unwrap();
        assert_eq!(vendor_data.len(), 6);
        match vendor_data.get(&DHCPOptionCode::DHCPMessageType){
            Some(option) =>  {
                assert_eq!(DHCPOptionCode::DHCPMessageType as u8, option.code);
                assert_eq!(1,  option.len);
                assert_eq!(vec![DHCPMessageType::DHCPACK as u8],  option.data);
            },
            None => {
                assert!(false, "Vendor data is mising Message Type Value");
            }
        }
        match vendor_data.get(&DHCPOptionCode::DNSServers){
            Some(option) =>  {
                assert_eq!(DHCPOptionCode::DNSServers as u8, option.code);
                assert_eq!(12,  option.len);
                assert_eq!(vec![75, 75, 75, 75, 75, 75, 75, 76, 8, 8, 8, 8],
                  option.data);
            },
            None => {
                assert!(false, "Vendor data is mising Message Type Value");
            }
        }
        match vendor_data.get(&DHCPOptionCode::IPAddressLeaseTime){
            Some(option) =>  {
                assert_eq!(DHCPOptionCode::IPAddressLeaseTime as u8, option.code);
                assert_eq!(4,  option.len);
                assert_eq!(vec![0,1,81,128],  option.data);
            },
            None => {
                assert!(false, "Vendor data is mising Message Type Value");
            }
        }
    }
}
