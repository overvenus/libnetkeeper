use std::str;

use rustc_serialize::hex::ToHex;
use openssl::crypto::hash::{Hasher, Type};

use dialer::Dialer;
use utils::{current_timestamp, integer_to_bytes};

// copy from https://github.com/miao1007/Openwrt-NetKeeper
#[derive(Debug)]
pub enum Configuration {
    Zhejiang,
    SingleNet,
    Enterprise,
    Chongqing,
    Chongqing2,
    Wuhan,
    Qinghai,
    Xinjiang,
    Hebei,
    Shandong,
    Shanxi,
    Gansu,
}

#[derive(Debug)]
pub struct NetkeeperDialer {
    pub share_key: String,
    pub prefix: String,
}

impl NetkeeperDialer {
    pub fn new(share_key: &str, prefix: &str) -> Self {
        NetkeeperDialer {
            share_key: share_key.to_string(),
            prefix: prefix.to_string(),
        }
    }

    pub fn encrypt_account(&self, username: &str, timestamp: Option<u32>) -> String {
        let username = username.to_uppercase();
        let timenow = match timestamp {
            Some(timestamp) => timestamp,
            None => current_timestamp(),
        };
        let time_div_by_five: u32 = timenow / 5;

        let mut pin27_byte: [u8; 6] = [0; 6];
        let pin27_str;
        {
            let mut time_hash: [u8; 4] = [0; 4];
            for i in 0..4 {
                for j in 0..8 {
                    time_hash[i] = time_hash[i] +
                                   ((((time_div_by_five >> (i + 4 * j)) & 1) << (7 - j)) &
                                    0xFF) as u8;
                }
            }

            pin27_byte[0] = (time_hash[0] >> 2) & 0x3F;
            pin27_byte[1] = ((time_hash[0] & 0x03) << 4) | ((time_hash[1] >> 4) & 0x0F);
            pin27_byte[2] = ((time_hash[1] & 0x0F) << 2) | ((time_hash[2] >> 6) & 0x03);
            pin27_byte[3] = time_hash[2] & 0x3F;
            pin27_byte[4] = (time_hash[3] >> 2) & 0x3F;
            pin27_byte[5] = (time_hash[3] & 0x03) << 4;

            for i in 0..6 {
                pin27_byte[i] = pin27_byte[i] + 0x20;
                if pin27_byte[i] > 0x40 {
                    pin27_byte[i] += 1;
                }
            }

            unsafe {
                pin27_str = str::from_utf8_unchecked(&pin27_byte);
            }
        }

        let pin89_str;
        {
            let mut md5 = Hasher::new(Type::MD5).unwrap();

            let time_div_by_five_be = time_div_by_five.to_be();
            let tdbf_bytes = integer_to_bytes(&time_div_by_five_be);

            md5.update(tdbf_bytes).unwrap();
            md5.update(username.split("@").nth(0).unwrap().as_bytes()).unwrap();
            md5.update(self.share_key.as_bytes()).unwrap();

            let hashed_bytes = md5.finish().unwrap();
            pin89_str = hashed_bytes[0..1].to_hex();
        }

        format!("{}{}{}{}", self.prefix, pin27_str, pin89_str, username)
    }
}

impl Configuration {
    pub fn share_key(&self) -> &'static str {
        match *self {
            Configuration::Zhejiang => "zjxinlisx01",
            Configuration::SingleNet => "singlenet01",
            Configuration::Enterprise => "zjxinlisx02",
            Configuration::Chongqing => "cqxinliradius002",
            Configuration::Chongqing2 => "xianxinli1radius",
            Configuration::Wuhan => "hubtxinli01",
            Configuration::Qinghai => "shd@xiaoyuan0002",
            Configuration::Xinjiang => "xinjiang@0724",
            Configuration::Hebei => "hebeicncxinli002",
            Configuration::Shandong => "shandongmobile13",
            Configuration::Shanxi => "sh_xi@xiaoyuan01",
            Configuration::Gansu => "xiaoyuanyixun001",
        }
    }

    pub fn prefix(&self) -> &'static str {
        match *self {
            _ => "\r\n",
        }
    }
}

impl Dialer for NetkeeperDialer {
    type C = Configuration;

    fn load_from_config(config: Self::C) -> Self {
        NetkeeperDialer::new(config.share_key(), config.prefix())
    }
}
