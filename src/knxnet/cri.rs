use nom::number::complete::{be_u16, be_u8};
use strum_macros::FromRepr;
use crate::knxnet::KnxNetIpError;

#[derive(FromRepr, Debug, Copy, Clone, PartialEq)]
#[repr(u8)]
pub enum ConnectionReqType {
    DeviceMgmtConnection = 0x03,
    TunnelConnection{
        layer: TunnelingLayer
    } = 0x04,
    RemlogConnection = 0x06,
    RemconfConnection = 0x07,
    ObjsvrConnection = 0x08,
}

impl Default for ConnectionReqType{
    fn default() -> Self {
        ConnectionReqType::TunnelConnection{layer: TunnelingLayer::default() }
    }
}

#[derive(FromRepr, Debug, Copy, Clone, PartialEq, Default)]
#[repr(u8)]
pub enum TunnelingLayer {
    #[default]
    TunnelLinkLayer = 0x02,
    TunnelRaw = 0x04,
    TunnelBusmon = 0x80,
}


impl ConnectionReqType {
    pub(crate) fn length(&self) -> u16{
        match self {
            ConnectionReqType::TunnelConnection{layer}  => 4, // tunnel CRI has length, type, link and one reserved byte
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
            ConnectionReqType::TunnelConnection{layer} => {
                buf.push(*layer as u8);
                buf.push(0);
            }
            _ => {panic!("Not yet implemented")}
        }
    }

    pub(crate) fn decode(buf: &[u8]) -> Result<ConnectionReqType, KnxNetIpError> {
        if buf[1] != 4 {
            return Err(KnxNetIpError::NotImplemented)
        }
        if buf[0] != 4 {
            return Err(KnxNetIpError::InvalidSize)
        }
        return Ok(ConnectionReqType::TunnelConnection{
            layer: match TunnelingLayer::from_repr(buf[2]){
                Some(layer) => layer,
                None => return Err(KnxNetIpError::UnknownLayer(buf[2]))
            }
        });
    }
}
