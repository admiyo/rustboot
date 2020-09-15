use std::mem::transmute;
use std::mem::size_of;
use mac_address::MacAddress;
use std::net::Ipv4Addr;
use std::collections::HashMap;
use std::vec::Vec;
use std::cmp::Eq;
use std::cmp::PartialEq;
use std::hash::Hash;

pub struct VendorData{
    pub code: u8,
    pub len: u8,
    pub data: Vec<u8>
}

const VENDOR_MAGIC:[u8; 4] = [99,130,83,99];

impl VendorData{
    fn new(code: u8, data: &Vec<u8>) ->Result<VendorData, &'static str>{
        let len = data.len();

        //Kindof bogus, as no single field will be this long.
        if len > 312 {
            return Err("vendor data too long")
        }else{
            return Ok(VendorData {
                code: code,
                len: data.len() as u8,
                data: data.to_vec()
            })
        }
    }
}

// These can all be found at:
// https://www.iana.org/assignments/bootp-dhcp-parameters/bootp-dhcp-parameters.xhtml
#[derive(Eq, Hash, PartialEq,
         ::num_derive::FromPrimitive,::num_derive::ToPrimitive)]
#[repr(u8)]
pub enum DHCPOptionCode{
    Pad = 0,
    // https://tools.ietf.org/html/rfc1533#section-9.4
    DHCPMessageType = 53,
    // https://tools.ietf.org/html/rfc2132#section-9.8
    ParameterRequestList = 55,
    // https://tools.ietf.org/html/rfc1533#section-9.8
    MaximumDHCPMessageSize = 57,
    //https://tools.ietf.org/html/rfc2132#section-9.13
    VendorClassIdentifier = 60,
    // https://tools.ietf.org/html/rfc2132#section-9.14
    ClientIdentifier = 61,
    // https://tools.ietf.org/html/rfc3004#section-4
    UserClassInfo = 77,
    // https://tools.ietf.org/html/rfc4578#section-2.1
    ClientSystemArchitectureType = 93,
    // https://tools.ietf.org/html/rfc4578#section-2.2
    ClientNetworkInterfaceIdentifier = 94 ,
    // https://tools.ietf.org/html/rfc4578#section-2.3
    ClientMachineIdentifier = 97,
    // 175 = Etherboot.  Undocumented.
    Etherboot = 175,
    End = 255
}

#[derive(Eq, Hash, PartialEq,::num_derive::FromPrimitive,::num_derive::ToPrimitive)]
#[repr(u8)]
pub enum DHCPMessageType {
    DHCPDISCOVER = 1,
    DHCPOFFER= 2,
    DHCPREQUEST= 3,
    DHCPDECLINE= 4,
    DHCPACK= 5,
    DHCPNAK= 6,
    DHCPRELEASE= 7
}


#[derive(Copy, Clone)]
#[repr(C)]
pub struct DHCPPacket{
    opcode: u8,
    _hwtype: u8,
    _hw_addr_len: u8,
    _hop_count: u8,
    _txn_id: [u8; 4],
    _num_secs: [u8; 2],
    _flags: [u8;2],
    _client_ip: [u8; 4],
    pub your_ip: [u8; 4],
    _server_ip: [u8; 4],
    _gateway_ip: [u8; 4],
    _client_mac: [u8; 6],
    _client_mac_remainder: [u8; 10],
    _server_host_name: [u8; 64],
    _boot_file_name: [u8; 128],
    _vendor_magic: [u8; 4],
    _vendor_info: [u8; 312]
}

impl DHCPPacket {
    pub fn new() -> DHCPPacket{
        let buf: [u8; size_of::<DHCPPacket>()] = [0; size_of::<DHCPPacket>()];
        unsafe {
            transmute::<[u8; size_of::<DHCPPacket>()],DHCPPacket>(buf)
        }
    }

    pub fn client_mac(&self) ->  MacAddress{
        MacAddress::new(self._client_mac)
    }

    pub fn vendor_magic(&self) -> [u8; 4]  {
        let mut retval: [u8; 4] = [0; 4];
        retval.copy_from_slice(&self._vendor_magic);
        retval
    }

    pub fn txn_id(&self) -> u32  {
        u32::from_be_bytes(self._txn_id)
    }

    pub fn log(&self){
        println!("----------------------------------------------------");
        println!("opcode      = {0}", self.opcode);
        println!("hwtype      = {0}", self._hwtype);
        println!("hw addr len = {0}", self._hw_addr_len);
        println!("hop count   = {0}", self._hop_count);
        println!("txn_id      = {:x}", self.txn_id());
        println!("num_secs    = {:}", u16::from_be_bytes(self._num_secs));
        println!("client_ip   = {0} ", Ipv4Addr::from(self._client_ip));
        println!("your_ip     = {0} ", Ipv4Addr::from(self.your_ip));
        println!("server_ip   = {0} ", Ipv4Addr::from(self._server_ip));
        println!("gateway_ip  = {0} ", Ipv4Addr::from(self._gateway_ip));
        println!("Mac Addr:   = {:}", self.client_mac());
    }

    //The packet should be copied when calling this function, as it will
    //Be converted to a byte array for the write.
    pub fn write_to_file(filename: &str, packet: DHCPPacket) {
        use std::io::prelude::*;
        use std::fs::File;
        let mut pos = 0;
        let mut buffer = File::create(filename).unwrap();
        let buf =
            unsafe {
                transmute::<DHCPPacket,
                            [u8; size_of::<DHCPPacket>()]>(packet)
            };
        while pos < buf.len() {
            let bytes_written = buffer.write(&buf[pos..]).unwrap();
            pos += bytes_written;
        }
    }

    pub fn generate_response(request_packet: &DHCPPacket) ->
        Result<DHCPPacket, &'static str>
    {
        println!("packet received");
        request_packet.log();

        if request_packet.vendor_magic() != VENDOR_MAGIC{
            return Err("Bad Vendor magic value");
        }
        match request_packet.parse_vendor_data(){
            Ok(options) => {
                for (code, option) in options{
                    println!("option code = {} len = {}",
                             code as u8, option.len);
                }
            },
            Err(s) => {
                println!("Bad Packet: {}", s);
                return Err(s)
            }
        }

        let mut response_packet = DHCPPacket::new();
        let server_hostname = "ayoungP40";
        response_packet._server_host_name[0..server_hostname.len()].
            copy_from_slice(server_hostname.as_bytes());
        response_packet._vendor_magic = VENDOR_MAGIC;
        response_packet.opcode = 2;
        response_packet._hwtype = response_packet._hwtype;
        response_packet._hw_addr_len = response_packet._hw_addr_len;
        response_packet._client_mac = response_packet._client_mac;
        response_packet._txn_id = response_packet._txn_id;
        response_packet._server_ip =  Ipv4Addr::new(192,168,144,1).octets();
        response_packet.your_ip =  Ipv4Addr::new(192,168,144,100).octets();

        //TODO write vendor data to packet
        let mut vendor_data:Vec::<VendorData> = vec!();
        vendor_data.push(VendorData::new(0, &vec![])?);
        vendor_data.push(VendorData::new(255, &vec![])?);

        println!("sending packet");
        response_packet.log();
        Ok(response_packet)
    }

    pub fn parse_vendor_data(&self) ->
        Result<HashMap::<DHCPOptionCode, VendorData>, &'static str>
    {
        let mut vendor_data:HashMap::<DHCPOptionCode, VendorData>
            = HashMap::new();
        let mut vend_itr  = self._vendor_info.iter();
        let vendor_data = loop {
            let next_code = vend_itr.next();

            match next_code {
                Some(code) => {
                    if *code == 255{
                        break vendor_data
                    }
                    if *code == 0   {
                        continue;
                    }else{
                        //TODO error handling
                        let len = vend_itr.next().unwrap();
                        let mut vend_info:Vec::<u8> = vec!();
                        for  _i in 0..*len{
                            let val = vend_itr.next();
                            match val {
                                Some(b) => vend_info.push(*b),
                                None => {
                                    vend_info.clear();
                                    println!("invalid code = {} len = {} _i={}",
                                             *code, len, _i);
                                    break
                                }
                            }
                        };

                        match num::FromPrimitive::from_u8(*code) {
                            Some(m_t_c) => {vendor_data.insert(m_t_c, VendorData{
                                code: *code,
                                len: *len,
                                data: vend_info});
                            },
                            None  => {
                                println!("unknown type code {}", *code);
                            }
                        };
                    }
                },
                None => break vendor_data
            }
        };
        Ok(vendor_data)
    }
}

#[cfg(test)]
mod tests {
    use std::env;
    use std::fs::File;
    use std::io::prelude::*;
    use super::*;
    use std::convert::TryFrom;

    fn read_packet() ->  DHCPPacket{
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
    fn test_create_packet() {
        let packet = DHCPPacket::new();
        assert_eq!(0, packet.opcode);
    }

    #[test]
    fn test_parse_packet() {
        let packet = read_packet();
        assert_eq!(1, packet.opcode);
        assert_eq!(1, packet._hwtype);
        assert_eq!(6, packet._hw_addr_len);
        assert_eq!(0, packet._hop_count);
        assert_eq!(4286046017, packet.txn_id());


        let test_mac = MacAddress::new([0x52,0x54,0x00,0x94,0x9e,0xf2]);
        assert_eq!(test_mac, packet.client_mac());

        assert_eq!(VENDOR_MAGIC,   packet.vendor_magic());
    }

    #[test]
    fn test_parse_vendor_data() {
        let packet = read_packet();
        let vendor_data = packet.parse_vendor_data().unwrap();

        assert_eq!(10,  vendor_data.len());

        match vendor_data.get(&DHCPOptionCode::DHCPMessageType){
            Some(option) =>  {
                assert_eq!(DHCPOptionCode::DHCPMessageType as u8, option.code);
                assert_eq!(1,  option.len);
                assert_eq!(vec![1],  option.data);
            },
            None => {
                assert!(false, "Vendor data is mising option 53");
            }
        }

        match vendor_data.get(&DHCPOptionCode::MaximumDHCPMessageSize){
            Some(option) =>  {
                assert_eq!(DHCPOptionCode::MaximumDHCPMessageSize as u8, option.code);
                assert_eq!(2,  option.len);
                assert_eq!(vec![5,192],  option.data);
            },
            None => {
                assert!(false, "Vendor data is mising option ");
            }
        }

        match vendor_data.get(&DHCPOptionCode::ClientSystemArchitectureType){
            Some(option) =>  {
                assert_eq!(DHCPOptionCode::ClientSystemArchitectureType as u8,
                           option.code);
                assert_eq!(2,  option.len);
                // 0 = Intel
                assert_eq!(vec![0, 0],  option.data);
            }
            None => {
                assert!(false, "Vendor data is mising option ");
            }
        }

        match vendor_data.get(&DHCPOptionCode::ClientNetworkInterfaceIdentifier){
            Some(option) =>  {

                assert_eq!(DHCPOptionCode::ClientNetworkInterfaceIdentifier as u8, option.code);
                assert_eq!(3,  option.len);
                assert_eq!(vec![1, 2, 1],  option.data);
            } ,           None => {
                assert!(false, "Vendor data is mising option ");
            }
        }

        match vendor_data.get(&DHCPOptionCode::VendorClassIdentifier){
            Some(option) =>  {
                assert_eq!(60, option.code);
                assert_eq!(32,  option.len);
                let vendor_class_id = std::str::from_utf8(
                    &option.data).unwrap();
                assert_eq!(vendor_class_id.len(), usize::from(option.len));
                assert_eq!("PXEClient:Arch:00000:UNDI:002001", vendor_class_id);
            },           None => {
                assert!(false, "Vendor data is mising option ");
            }
        }
        match vendor_data.get(&DHCPOptionCode::UserClassInfo){
            Some(option) =>  {
                assert_eq!(77, option.code);
                assert_eq!(4,  option.len);

                let user_class_info = std::str::from_utf8(
                    &option.data).unwrap();
                assert_eq!(user_class_info.len(), usize::from(option.len));
                assert_eq!("iPXE", user_class_info);
                assert_eq!(vec![105, 80, 88, 69],  option.data);
            },           None => {
                assert!(false, "Vendor data is mising option ");
            }
        }
        match vendor_data.get(&DHCPOptionCode::ParameterRequestList){
            Some(option) =>  {

                assert_eq!(55, option.code);
                assert_eq!(23,  option.len);
                assert_eq!(23,  option.data.len());
                assert_eq!(vec![1, 3, 6, 7, 12, 15, 17, 26, 43, 60, 66, 67, 119,
                                128, 129, 130, 131, 132, 133, 134, 135, 175, 203],
                           option.data);
            },           None => {
                assert!(false, "Vendor data is mising option ");
            }
        }
        match vendor_data.get(&DHCPOptionCode::Etherboot){
            Some(option) =>  {

                assert_eq!(175, option.code);
                assert_eq!(48,  option.len);
            },           None => {
                assert!(false, "Vendor data is mising option ");
            }
        }
        match vendor_data.get(&DHCPOptionCode::ClientIdentifier){
            Some(option) =>  {
                assert_eq!(61, option.code);
                assert_eq!(7,  option.len);
                assert_eq!(vec![1, 82, 84, 0, 148, 158, 242], option.data);

            },           None => {
                assert!(false, "Vendor data is mising option ");
            }
        }

        match vendor_data.get(&DHCPOptionCode::ClientMachineIdentifier){
            Some(option) =>  {

                assert_eq!(97, option.code);
                assert_eq!(17, option.len);
                assert_eq!(vec![0, 178, 35, 76, 56, 225, 195, 173, 69, 183,
                                151, 210, 221, 34, 14, 27, 157],
                           option.data);

            },           None => {
                assert!(false, "Vendor data is mising option ");
            }
        }
    }

    #[test]
    fn test_new_vendor_data_ok() {
        match VendorData::new(53, &vec![1]){
            Ok(vendor_data) => {
                assert_eq!(53, vendor_data.code);
                assert_eq!(1,  vendor_data.len);
                assert_eq!(vec![1],  vendor_data.data);
            },
            Err(msg) =>  assert!(false, msg)
        }
    }

    #[test]
    fn test_new_vendor_data_too_long() {
        match VendorData::new(53, &vec![0; 488]){
            Ok(vendor_data) => {
                assert!(false, "vendor data {} would overun buffer",
                        vendor_data.data.len());
            },
            Err(_) =>  assert!(true)
        }
    }

    #[test]
    fn test_write_vendor_data_to_buffer() {
        match VendorData::new(53, &vec![1]){
            Ok(vendor_data) => {
                let mut buf:[u8; 4] = [9;4];
                buf[3] = 9;
                {//this code moves to a function
                    buf[0] = vendor_data.code;
                    buf[1] = vendor_data.len;

                    for i in 0 .. vendor_data.data.len() {
                        buf[i+2] = vendor_data.data[i];
                    }
                }
                assert_eq!( buf[3], 9);
            },
            Err(msg) =>  assert!(false, msg)
        }
    }
    #[test]
    fn test_generate_response(){
        assert!(!DHCPPacket::generate_response(&read_packet()).is_err());
    }
}
