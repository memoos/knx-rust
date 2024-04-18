use strum_macros::FromRepr;
use crate::knxnet::KnxNetIpError;

#[derive(FromRepr, Debug, Copy, Clone, PartialEq)]
#[repr(u8)]
pub enum InformationType{
    PlMediumInformation = 0x01,
    RfMediumInformation = 0x02,
    BusmonitorStatusInfo = 0x03,
    TimestampRelative = 0x04,
    TimeDelayUntilSending = 0x05,
    ExtendedRelativeTimestamp = 0x06,
    BiBatInformation = 0x07,
    RfMultiInformation = 0x08,
    PreambleAndPostamble = 0x09,
    RfFastAckInformation = 0x0A,
    ManufacturerSpecific = 0xFE,
    Reserved = 0xFF
}


impl InformationType {
    pub(crate) fn length(&self)->u8 {
        return match self {
            //TODO implement
            _ => 0
        }
    }

    fn identifier(&self) -> u8 {
        // SAFETY: Because `Self` is marked `repr(u16)`, its layout is a `repr(C)` `union`
        // between `repr(C)` structs, each of which has the `u16` discriminant as its first
        // field, so we can read the discriminant without offsetting the pointer.
        unsafe { *<*const _>::from(self).cast::<u8>() }
    }

    pub(crate) fn encode(&self, buf: &mut Vec<u8>){
        match self {
            _ => {panic!("not implemented")}
        }
    }

    pub(crate) fn decode(buf: &[u8]) -> Result<InformationType, KnxNetIpError> {
        let type_id = InformationType::from_repr(buf[0]);
        return match type_id{
            None => Err(KnxNetIpError::Unknown),
            Some(mut info) => {
                match info {
                    // TODO add implementations
                    _ => Ok(info)
                }
            }
        };
    }

}
