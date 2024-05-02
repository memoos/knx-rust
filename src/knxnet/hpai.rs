use byteorder::{BigEndian, ByteOrder};
use strum_macros::FromRepr;
use crate::knxnet::KnxNetIpError;

#[derive(FromRepr, Debug, Copy, Clone, PartialEq, Default)]
#[repr(u8)]
pub enum Protocol {
    #[default]
    Udp4Protocol = 0x01,
    Tcp4Protocol = 0x02,
}

#[derive(Debug, Copy, Clone, PartialEq, Default)]
pub struct HPAI {
    pub(crate) protocol: Protocol,
    pub(crate) address: [u8;4],
    pub(crate) port: u16,
}

impl HPAI {
    /// Create an TunnelConnReq
    pub fn new(protocol: Protocol, address: [u8;4], port: u16) -> HPAI {
        HPAI {
            port: port,
            address: address,
            protocol: protocol,
        }
    }

    pub(crate) fn length() -> u16{
        8
    }

    pub(crate) fn encode(&self, buf: &mut Vec<u8>){
        buf.push(0x8); // length
        buf.push(self.protocol as u8);
        buf.extend_from_slice(&self.address[..]);
        buf.extend_from_slice(&self.port.to_be_bytes());
    }

    pub(crate) fn decode(buf: &[u8]) -> Result<HPAI, KnxNetIpError> {
        if buf.len() < 8 || buf[0] != 8{
            return Err(KnxNetIpError::InvalidSize)
        }
        Ok(HPAI{
            protocol: match Protocol::from_repr(buf[1]) {
                Some(t) => t,
                None => return Err(KnxNetIpError::UnknownProtocol(buf[1]))
            },
            address: [buf[2],buf[3],buf[4],buf[5]],
            port: BigEndian::read_u16(&buf[6..8])
        })
    }
}
