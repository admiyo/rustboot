use std::mem::transmute;
use std::mem::size_of;
use mac_address::MacAddress;
use std::net::Ipv4Addr;
use std::vec::Vec;

pub struct VendorData{
    code: u8,
    len: u8,
    data: Vec<u8>
}

type ParseError = &'static str;

pub fn new_vendor_data(code: u8, data: &Vec<u8>) ->Result<VendorData, ParseError>{
    let len = data.len();

    if len > 60 {
        return Err("vendor data too long")
    }else{
        return Ok(VendorData {
            code: code,
            len: data.len() as u8,
            data: data.to_vec()
        })
    }
}


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
    _vendor_magic: [u8; 4],
    _vendor_info: [u8; 60]
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

    pub fn vendor_magic(&self) -> [u8; 4]  {
        let mut retval: [u8; 4] = [0; 4];
        retval.copy_from_slice(&self._vendor_magic);
        retval
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

fn parse_vendor_data(packet: &BootPacket) -> Vec::<VendorData> {
    let mut vendor_data:Vec::<VendorData> = vec!();
    let mut vend_itr  = packet._vendor_info.iter();
    
    let vendor_data = loop {
        let next_code = vend_itr.next();

        match next_code {
            Some(code) => {
                if (*code == 0) || (*code == 255) {
                    vendor_data.push(VendorData{
                        code: *code,
                        len: 0,
                        data: vec!()
                    });
                }else{
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
                    vendor_data.push(VendorData{
                        code: *code,
                        len: *len,
                        data: vend_info
                    });
                }
            },
            None => break vendor_data
        }
    };
    vendor_data
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
        let mut buffer: [u8; size_of::<BootPacket>()] = [
            0; size_of::<BootPacket>()];
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
        assert_eq!([99,130,83,99],   packet.vendor_magic());
    }


    #[test]
    fn test_parse_vendor_data() {
        let packet = read_packet();
        let vendor_data:Vec::<VendorData> = parse_vendor_data(&packet);

         // These can all be found at:
        // https://www.iana.org/assignments/bootp-dhcp-parameters/bootp-dhcp-parameters.xhtml
        
        {  // 53 DHCP Message type
           // https://tools.ietf.org/html/rfc1533#section-9.4
            assert_eq!(53, vendor_data[0].code);
            assert_eq!(1,  vendor_data[0].len);
            assert_eq!(vec![1],  vendor_data[0].data);
        }
        {
            // 57 = Maximum DHCP Message Size
            // https://tools.ietf.org/html/rfc1533#section-9.8
            assert_eq!(57, vendor_data[1].code);
            assert_eq!(2,  vendor_data[1].len);
            assert_eq!(vec![5,192],  vendor_data[1].data);
        }
        {
            // 93 = Client System Architecture Type Option Definition 
            // https://tools.ietf.org/html/rfc4578#section-2.1
            assert_eq!(93, vendor_data[2].code);
            assert_eq!(2,  vendor_data[2].len);
            // 0 = Intel
            assert_eq!(vec![0, 0],  vendor_data[2].data);
        }
        {
            // 94 = Client Network Interface Identifier Option Definition
            // https://tools.ietf.org/html/rfc4578#section-2.2
            assert_eq!(94, vendor_data[3].code);
            assert_eq!(3,  vendor_data[3].len);
            assert_eq!(vec![1, 2, 1],  vendor_data[3].data);
        }
        {
            //https://tools.ietf.org/html/rfc2132#section-9.13
            assert_eq!(60, vendor_data[4].code);
            assert_eq!(32,  vendor_data[4].len);
            let vendor_class_id = std::str::from_utf8(
                &vendor_data[4].data).unwrap(); 
            assert_eq!(vendor_class_id.len(), usize::from(vendor_data[4].len));
            assert_eq!("PXEClient:Arch:00000:UNDI:002001", vendor_class_id);
        }        
        {
            //77 User class info
            // https://tools.ietf.org/html/rfc3004#section-4
            assert_eq!(77, vendor_data[5].code);
            assert_eq!(4,  vendor_data[5].len);

            let user_class_info = std::str::from_utf8(
                &vendor_data[5].data).unwrap(); 
            assert_eq!(user_class_info.len(), usize::from(vendor_data[5].len));
            assert_eq!("iPXE", user_class_info);
            assert_eq!(vec![105, 80, 88, 69],  vendor_data[5].data);
        }        
        {
            //The sample packet has a mangled value for Option 55.
            //55 Parameter Request List
            // https://tools.ietf.org/html/rfc2132#section-9.8
            assert_eq!(55, vendor_data[6].code);
            assert_eq!(23,  vendor_data[6].len);
            assert_eq!(0,  vendor_data[6].data.len());
        }        
    }

    #[test]
    fn test_new_vendor_data_ok() {
        match new_vendor_data(53, &vec![1]){
            Ok(vendor_data) => {
                assert_eq!(53, vendor_data.code);
                assert_eq!(1,  vendor_data.len);
                assert_eq!(vec![1],  vendor_data.data);
            },
            Err(msg) =>  assert!(false, msg)
        }   
    }
}
