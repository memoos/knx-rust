use byteorder::{BigEndian, ByteOrder};
use strum_macros::FromRepr;
use crate::knxnet::KnxNetIpError;


#[derive(FromRepr, Debug, Copy, Clone, PartialEq)]
#[repr(u8)]
pub enum ConnectionRespType {
    DeviceMgmtConnection = 0x03,
    TunnelConnection{
        address: u16 // TODO use individual address type
    } = 0x04,
    RemlogConnection = 0x06,
    RemconfConnection = 0x07,
    ObjsvrConnection = 0x08,
}

impl Default for ConnectionRespType{
    fn default() -> Self {
        ConnectionRespType::TunnelConnection {address: 0}
    }
}

impl ConnectionRespType {
    pub(crate) fn length(&self) -> u16{
        match self {
            ConnectionRespType::TunnelConnection{address}  => 4, // tunnel CRD has length, type and an u16 address
            _ => 2 // others are not yet implemented
        }
    }

    fn identifier(&self) -> u8 {
        // SAFETY: Because `Self` is marked `repr(u8)`, its layout is a `repr(C)` `union`
        // between `repr(C)` structs, each of which has the `u8` discriminant as its first
        // field, so we can read the discriminant without offsetting the pointer.
        unsafe { *<*const _>::from(self).cast::<u8>() }
    }

    pub(crate) fn encode(&self, buf: &mut Vec<u8>){
        buf.push(self.length() as u8); // length
        buf.push(self.identifier());
        match self {
            ConnectionRespType::TunnelConnection{address} => {
                buf.extend(address.to_be_bytes());
            }
            _ => {panic!("Not yet implemented")}
        }
    }

    pub(crate) fn decode(buf: &[u8]) -> Result<ConnectionRespType, KnxNetIpError> {
        if buf[1] != 4 {
            return Err(KnxNetIpError::NotImplemented)
        }
        if buf[0] != 4 {
            return Err(KnxNetIpError::InvalidSize)
        }
        return Ok(ConnectionRespType::TunnelConnection{
            address: BigEndian::read_u16(&buf[2..4])
        });
    }
}
