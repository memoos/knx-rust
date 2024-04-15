use nom::number::complete::{be_u16, be_u8};
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

named!(parse_host_info<&[u8], HPAI>,
    do_parse!(
    _length: be_u8 >>
    protocol: be_u8 >>
    address1: be_u8 >>
    address2: be_u8 >>
    address3: be_u8 >>
    address4: be_u8 >>
    port: be_u16 >>

    (HPAI{
        port: port,
        address: [address1, address2, address3, address4],
        protocol: Protocol::from_repr(protocol).unwrap(),
    })
    )
);

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
        match parse_host_info(&buf){
            Ok(v) => Ok(v.1),
            Err(e) => Err(KnxNetIpError::Unknown)
        }
    }
}
