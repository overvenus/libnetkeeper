use std::net::Ipv4Addr;
use std::num::Wrapping;

use utils::integer_to_bytes;

// copy from https://github.com/xuzhipengnt/ipclient_gxnu
const USERNAME_MAX_LEN: usize = 30;
const MAC_ADDRESS_LEN: usize = 18;

#[derive(Debug)]
pub enum MACOpenErr {
    UsernameTooLong,
    MACAddressError,
}

#[derive(Debug)]
pub struct MACOpenPacket {
    username: String,
    ipaddress: Ipv4Addr,
    mac_address: String,
    isp: ISPCode,
}

#[derive(Debug)]
pub enum Configuration {
    GUET,
    GXNU,
}

#[derive(Debug, Clone, Copy)]
pub enum ISPCode {
    CChinaUnicom = 1 << 8,
    CChinaTelecom = 2 << 8,
    CChinaMobile = 3 << 8,
}

impl Configuration {
    pub fn hash_key(&self) -> u32 {
        match *self {
            _ => 0x4E67C6A7,
        }
    }
}

impl MACOpenPacket {
    pub fn new(username: &str, ipaddress: Ipv4Addr, mac_address: &str, isp: ISPCode) -> Self {
        MACOpenPacket {
            username: username.to_string(),
            ipaddress: ipaddress,
            mac_address: mac_address.to_string(),
            isp: isp,
        }
    }

    pub fn as_bytes(&self, hash_key: u32) -> Result<Box<Vec<u8>>, MACOpenErr> {
        try!(self.validate());

        let mut macopen_packet: Box<Vec<u8>> = Box::new(Vec::with_capacity(60));
        {
            let mut username_bytes = [0; USERNAME_MAX_LEN];
            let mut mac_address_bytes = [0; MAC_ADDRESS_LEN];
            username_bytes[..self.username.len()].clone_from_slice(self.username.as_bytes());
            mac_address_bytes[..self.mac_address.len()]
                .clone_from_slice(self.mac_address.as_bytes());

            let isp_be = (self.isp.clone() as u32).to_be();
            let isp_bytes = integer_to_bytes(&isp_be);

            macopen_packet.extend(&username_bytes);
            macopen_packet.extend(&self.ipaddress.octets());
            macopen_packet.extend(&mac_address_bytes);
            macopen_packet.extend(isp_bytes);

            let hash_bytes = Self::hash_bytes(&macopen_packet, hash_key);
            macopen_packet.extend(&hash_bytes);
        }

        Ok(macopen_packet)
    }

    fn validate(&self) -> Result<(), MACOpenErr> {
        if self.username.len() > USERNAME_MAX_LEN - 1 {
            return Err(MACOpenErr::UsernameTooLong);
        }
        if self.mac_address.len() != MAC_ADDRESS_LEN - 1 {
            return Err(MACOpenErr::MACAddressError);
        }
        Ok(())
    }

    fn hash_bytes(bytes: &[u8], hash_key: u32) -> [u8; 4] {
        let mut hash = Wrapping(hash_key as i32);
        for c in bytes.iter() {
            hash ^= (hash << 5) + (hash >> 2) + Wrapping(*c as i32);
        }
        hash &= Wrapping(0x7fffffff);

        let mut hash_bytes = [0; 4];
        hash_bytes.clone_from_slice(integer_to_bytes(&hash.0));
        hash_bytes
    }
}


#[test]
fn test_mac_opener_hash_bytes() {
    let bytes1 = [1, 2, 3, 4, 5, 6, 7, 0];
    let hash_bytes1 = MACOpenPacket::hash_bytes(&bytes1, Configuration::GUET.hash_key());

    let bytes2 = [97, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                  0, 0, 0, 0, 172, 16, 1, 1, 52, 48, 58, 54, 49, 58, 56, 54, 58, 56, 55, 58, 57,
                  70, 58, 70, 49, 0, 0, 0, 1, 0];
    let hash_bytes2 = MACOpenPacket::hash_bytes(&bytes2, Configuration::GUET.hash_key());

    assert_eq!(hash_bytes1, [0x9c, 0x89, 0xf8, 0x3d]);
    assert_eq!(hash_bytes2, [255, 189, 40, 90]);
}