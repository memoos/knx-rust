use strum_macros::FromRepr;
use crate::dpt::DPT;

pub(crate) mod l_data;
mod information;
pub mod apdu;

use crate::cemi::information::InformationType;
use crate::cemi::l_data::LData;
use crate::knxnet::KnxNetIpError;


#[derive(FromRepr, Debug, Clone, PartialEq, Default)]
#[repr(u8)]
pub enum Message<D:DPT+Default> {
    //For Knx Data Link Layer
    LDataReq(Vec<InformationType>, LData<D>) = 0x11, // transmitted frame
    LDataInd(Vec<InformationType>, LData<D>) = 0x29, // received frame
    LDataCon(Vec<InformationType>, LData<D>) = 0x2E, // confirmation

    //for raw mode
    LRawReq = 0x10,
    LRawCon = 0x2F,
    LRawInd = 0x2D,

    //Common
    MResetReq = 0xF1,

    //For busmon
    LBusmonInd = 0x2B,

    #[default]
    None,
}

impl<D:DPT+Default> Message<D> {
    pub(crate) fn length(&self)->u8 {
        return 2 + match self {
            Self::LDataReq(info, data) => info.iter().map(|i| {i.length()}).sum::<u8>() + data.length(),
            Self::LDataInd(info, data) => info.iter().map(|i| {i.length()}).sum::<u8>() + data.length(),
            Self::LDataCon(info, data) => info.iter().map(|i| {i.length()}).sum::<u8>() + data.length(),
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
        buf.push(self.identifier());
        match self {
            Self::LDataReq(info, data) => {
                buf.push(info.iter().map(|i| {i.length()}).sum());
                info.iter().for_each(|i| {i.encode(buf)});
                data.encode(buf);
            },
            Self::LDataInd(info, data) => {
                buf.push(info.iter().map(|i| {i.length()}).sum());
                info.iter().for_each(|i| {i.encode(buf)});
                data.encode(buf);
            },
            Self::LDataCon(info, data) => {
                buf.push(info.iter().map(|i| {i.length()}).sum());
                info.iter().for_each(|i| {i.encode(buf)});
                data.encode(buf);
            },
            _ => {}
        }
    }

    pub(crate) fn decode(buf: &[u8]) -> Result<Message<D>, KnxNetIpError> {
        if buf.len() < 1 {
            return Err(KnxNetIpError::MessageTooShort(buf.len()))
        }
        return match Message::from_repr(buf[0]){
            None => {Err(KnxNetIpError::NotImplemented)}
            Some(mut msg) => {
                match &mut msg {
                    Self::LDataReq(ref mut info, ref mut data) => {
                        if buf.len() < 2 || buf.len() < 2 + buf[1] as usize {
                            return Err(KnxNetIpError::MessageTooShort(buf.len()))
                        }
                        if buf[1] > 0 {/*TODO implement info vec decoding*/}
                        data.decode(&buf[(2+buf[1] as usize)..])?;
                    },
                    Self::LDataInd(ref mut info, ref mut data) => {
                        if buf.len() < 2 || buf.len() < 2 + buf[1] as usize {
                            return Err(KnxNetIpError::MessageTooShort(buf.len()))
                        }
                        if buf[1] > 0 {/*TODO implement info vec decoding*/}
                        data.decode(&buf[(2+buf[1] as usize)..])?;
                    },
                    Self::LDataCon(ref mut info, ref mut data) => {
                        if buf.len() < 2 || buf.len() < 2 + buf[1] as usize {
                            return Err(KnxNetIpError::MessageTooShort(buf.len()))
                        }
                        if buf[1] > 0 {/*TODO implement info vec decoding*/}
                        data.decode(&buf[(2+buf[1] as usize)..])?;
                    },
                    _ => {}
                };
                Ok(msg)
            }
        }

    }
}

#[cfg(test)]
mod tests {
    use crate::cemi::apdu::Apdu;
    use crate::dpt::DPT;
    use crate::cemi::l_data::{Acknowledge, AddressType, Confirmation, FrameFormat, FrameType, LData, Priority, Repetition, SystemBroadcast};
    use crate::cemi::Message;
    use crate::knxnet::connect::ConnectRequest;
    use crate::knxnet::Service;

    #[test]
    fn t_message_length() {
        assert_eq!(Service::<()>::ConnectRequest(ConnectRequest::default()).length(), 26);
    }

    #[test]
    fn t_message_encode() {
        let l_data_ind_msg1 = Message::LDataInd(vec![], LData{
            frame_type: FrameType::Standard,
            repetition: Repetition::NoRepeat,
            system_broadcast: SystemBroadcast::Broadcast,
            priority: Priority::Low,
            acknowledge: Acknowledge::NoAcknowledge,
            confirmation: Confirmation::NoError,
            destination_address_type: AddressType::Group,
            hop_count: 6,
            frame_format: FrameFormat::Standard,
            source: 0x11D5,
            destination: 0x128,
            numbered: false,
            seq: 0,
            control: false,
            data: Apdu::GroupValueWrite(vec![0x19, 0x0e])
        });
        let mut data = vec![];
        l_data_ind_msg1.encode(&mut data);

        assert_eq!(data, vec![0x29, 0x00, 0xBC, 0xE0, 0x11, 0xD5, 0x01, 0x28, 0x03, 0x00, 0x80, 0x19, 0x0e]);

        let l_data_ind_msg2 = Message::LDataInd(vec![], LData{
            frame_type: FrameType::Standard,
            repetition: Repetition::Repeat,
            system_broadcast: SystemBroadcast::SystemBroadcast,
            priority: Priority::Low,
            acknowledge: Acknowledge::NoAcknowledge,
            confirmation: Confirmation::NoError,
            destination_address_type: AddressType::Group,
            hop_count: 6,
            frame_format: FrameFormat::Standard,
            source: 0x11D5,
            destination: 0x128,
            numbered: false,
            seq: 0,
            control: false,
            data: Apdu::GroupValueWrite(true)
        });
        data.clear();
        l_data_ind_msg2.encode(&mut data);

        assert_eq!(data, vec![0x29, 0x00, 0x8C, 0xE0, 0x11, 0xD5, 0x01, 0x28, 0x01, 0x00, 0x81]);

        let l_data_req_msg1 = Message::LDataReq(vec![], LData{
            frame_type: FrameType::Standard,
            repetition: Repetition::Repeat,
            system_broadcast: SystemBroadcast::SystemBroadcast,
            priority: Priority::Low,
            acknowledge: Acknowledge::NoAcknowledge,
            confirmation: Confirmation::NoError,
            destination_address_type: AddressType::Group,
            hop_count: 6,
            frame_format: FrameFormat::Standard,
            source: 0x11D5,
            destination: 0x128,
            numbered: false,
            seq: 0,
            control: false,
            data: Apdu::GroupValueWrite(true)
        });
        data.clear();
        l_data_ind_msg2.encode(&mut data);
    }

    #[test]
    fn t_message_decode() {
    }

    #[test]
    fn t_service_decode_errors() {
    }
}
