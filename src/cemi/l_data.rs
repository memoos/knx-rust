use std::any::Any;
use std::ops::BitOr;
use byteorder::{BigEndian, ByteOrder};
use strum_macros::FromRepr;
use crate::cemi::l_data::SystemBroadcast::Broadcast;
use crate::cemi::Message;
use crate::cemi::apdu::Apdu;
use crate::cemi::dpt::DPT;
use crate::knxnet::{KnxNetIpError, Service};


#[derive(FromRepr, Debug, Copy, Clone, PartialEq, Default)]
#[repr(u8)]
pub enum FrameType{
    Extended = 0x00,
    #[default]
    Standard = 0x80,
}

#[derive(FromRepr, Debug, Copy, Clone, PartialEq, Default)]
#[repr(u8)]
pub enum Repetition{
    Repeat = 0x00,
    #[default]
    NoRepeat = 0x20,
}

#[derive(FromRepr, Debug, Copy, Clone, PartialEq, Default)]
#[repr(u8)]
pub enum SystemBroadcast{
    SystemBroadcast = 0x00,
    #[default]
    Broadcast = 0x10,
}

#[derive(FromRepr, Debug, Copy, Clone, PartialEq, Default)]
#[repr(u8)]
pub enum Priority {
    #[default]
    Low = 0x0C,
    Normal = 0x04,
    Urgent = 0x08,
    System = 0x00,
}

#[derive(FromRepr, Debug, Copy, Clone, PartialEq, Default)]
#[repr(u8)]
pub enum Acknowledge {
    NoAcknowledge = 0x00,
    #[default]
    Acknowledge = 0x02,
}

#[derive(FromRepr, Debug, Copy, Clone, PartialEq, Default)]
#[repr(u8)]
pub enum Confirmation {
    #[default]
    NoError = 0x00,
    Error = 0x01,
}

#[derive(FromRepr, Debug, Copy, Clone, PartialEq, Default)]
#[repr(u8)]
pub enum FrameFormat {
    #[default]
    Standard = 0x00,
    LTE0 = 0x04,
    LTE1 = 0x05,
    LTE2 = 0x06,
    LTE3 = 0x07,
}

#[derive(FromRepr, Debug, Copy, Clone, PartialEq, Default)]
#[repr(u8)]
pub enum AddressType {
    Individual = 0x00,
    #[default]
    Group = 0x80,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct LData<D: DPT+Default>{
    pub(crate) frame_type: FrameType,
    pub(crate) repetition: Repetition,
    pub(crate) system_broadcast: SystemBroadcast,
    pub(crate) priority: Priority,
    pub(crate) acknowledge: Acknowledge,
    pub(crate) confirmation: Confirmation,
    pub(crate) destination_address_type: AddressType,
    pub(crate) hop_count: u8,
    pub(crate) frame_format: FrameFormat,
    pub(crate) source: u16,
    pub(crate) destination: u16,
    //TPDU fields
    pub(crate) control: bool,
    pub(crate) numbered: bool,
    pub(crate) seq: u8,

    pub(crate) data: Apdu<D>,
}

impl<D: DPT+Default> Default for LData<D>{
    fn default() -> Self {
        return LData{
            frame_type: FrameType::default(),
            repetition: Repetition::default(),
            system_broadcast: SystemBroadcast::default(),
            priority: Priority::default(),
            acknowledge: Acknowledge::default(),
            confirmation: Confirmation::default(),
            destination_address_type: AddressType::default(),
            hop_count: 6,
            frame_format: FrameFormat::default(),
            source: 0,
            destination: 0,
            control: false,
            numbered: false,
            seq: 0,
            data: Apdu::default(),
        }
    }
}



impl<D:DPT+Default> LData<D> {
    pub(crate) fn length(&self)->u8 {
        return (8 + self.data.length()) as u8
    }

    pub(crate) fn encode(&self, buf: &mut Vec<u8>) {
        buf.push(self.frame_type as u8|self.repetition as u8|self.system_broadcast as u8|self.priority as u8|self.acknowledge as u8|self.confirmation as u8);
        buf.push(self.destination_address_type as u8 | ((self.hop_count&0x7) << 4) | self.frame_format as u8);
        buf.extend(self.source.to_be_bytes());
        buf.extend(self.destination.to_be_bytes());
        buf.push(self.data.length() as u8);
        let b7 = if self.control {0x80} else {0} | if self.numbered {0x40} else {0} | ((self.seq & 0xF)<<2);
        self.data.encode(buf, b7);
    }

    pub(crate) fn decode(&mut self, buf: &[u8]) -> Result<(), KnxNetIpError> {
        if buf.len() < 8  || buf.len() < (8 + buf[6]) as usize {
            return Err(KnxNetIpError::MessageTooShort(buf.len()))
        }
        self.frame_type = FrameType::from_repr(buf[0]&0x80).unwrap();
        self.repetition = Repetition::from_repr(buf[0] & 0x20).unwrap();
        self.system_broadcast = SystemBroadcast::from_repr(buf[0] & 0x10).unwrap();
        self.priority = Priority::from_repr(buf[0] & 0x0C).unwrap();
        self.acknowledge = Acknowledge::from_repr(buf[0] & 0x02).unwrap();
        self.confirmation = Confirmation::from_repr(buf[0] & 0x01).unwrap();
        self.destination_address_type = AddressType::from_repr(buf[1] & 0x80).unwrap();
        self.hop_count = (buf[1] & 0x70) >> 4;
        self.frame_format = match FrameFormat::from_repr(buf[1] & 0x07) {Some(v) => v, None =>{ return Err(KnxNetIpError::Unknown)}};
        self.source = BigEndian::read_u16(&buf[2..4]);
        self.destination = BigEndian::read_u16(&buf[4..6]);
        self.control = (buf[7] & 0x80) != 0;
        self.numbered = (buf[7] & 0x40) != 0;
        self.seq = (buf[7] >> 2) & 0xF;
        self.data = Apdu::decode(&buf[6..(8+buf[6] as usize)])?;

        return Ok(());
    }

}