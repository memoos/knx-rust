use strum_macros::FromRepr;
use crate::dpt::DPT;
use crate::knxnet::KnxNetIpError;

#[derive(PartialEq, FromRepr, Debug, Clone, Default)]
#[repr(u16)]
pub enum Apdu<D: DPT+Default>{
    #[default]
    GroupValueRead          = 0b0000_000000,
    GroupValueResponse(D) = 0b0001_000000,
    GroupValueWrite(D)    = 0b0010_000000,

    IndividualAddressWrite    = 0b0011_000000,
    IndividualAddressRead     = 0b0100_000000,
    IndividualAddressResponse = 0b0101_000000,

    AdcRead     = 0b0110_000000,
    AdcResponse = 0b0111_000000,

    None
}

impl<D: DPT+Default> Apdu<D> {
    pub(crate) fn length(&self)->u8 {
        match self {
            Apdu::GroupValueWrite(dpt) | Apdu::GroupValueResponse(dpt) => {
                return if dpt.bit_len() > 6 { (1 + dpt.bit_len() / 8) as u8 } else { 1 }
            }
            Apdu::GroupValueRead => 1,
            _ => 0
        }
    }

    fn identifier(&self) -> u16 {
        // SAFETY: Because `Self` is marked `repr(u16)`, its layout is a `repr(C)` `union`
        // between `repr(C)` structs, each of which has the `u16` discriminant as its first
        // field, so we can read the discriminant without offsetting the pointer.
        unsafe { *<*const _>::from(self).cast::<u16>() }
    }

    pub(crate) fn encode(&self, buf: &mut Vec<u8>, previous_byte: u8) {
        let apci = self.identifier() & 0b1111_111111;
        // first two bits of apci are encoded in tpci byte which is handed over as previous_byte
        buf.push(previous_byte | ((apci >> 8) & 0x3) as u8);
        match self {
            Apdu::GroupValueRead => {buf.push((apci & 0xff) as u8);},
            Apdu::GroupValueWrite(dpt) | Apdu::GroupValueResponse(dpt) => {
                if dpt.bit_len() <= 6 {
                    dpt.encode(buf);
                    let len = buf.len();
                    buf[len-1] |= (apci & 0xff) as u8
                } else {
                    buf.push((apci & 0xff) as u8);
                    dpt.encode(buf);
                }
            }
            _ => {
                buf.push((apci & 0xff) as u8);
                //TODO implement data encoding for other group services
            }
        }
    }

    pub(crate) fn decode(buf: &[u8]) -> Result<Apdu<D>, KnxNetIpError> {
        Ok(
            if buf[0] == 0 {
                Apdu::None
            } else {
                let apci = ((buf[1] & 0x3) as u16) << 8 | (buf[2]) as u16;
                let short_apci = apci & 0b1111_000000;

                match Apdu::from_repr(short_apci) {
                    None => return Err(KnxNetIpError::Unknown),
                    Some(mut a ) => match a{
                        Apdu::GroupValueRead => a,
                        Apdu::GroupValueResponse(ref mut dpt) | Apdu::GroupValueWrite(ref mut dpt) => {
                            dpt.decode(&buf[2..])?; a},
                        _ => return Err(KnxNetIpError::NotImplemented)
                    }
                }
            }
        )
    }

}