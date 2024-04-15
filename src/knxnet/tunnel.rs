use crate::knxnet::{KnxNetIpError};
use crate::knxnet::cemi::Message;
use crate::knxnet::status::StatusCode;

#[derive(Debug, PartialEq, Default)]
pub(crate) struct TunnelRequest {
    pub(crate) channel: u8,
    pub(crate) seq: u8,
    pub(crate) message_code: Message,
    pub(crate) payload: Vec<u8>,
}

#[derive(Debug, PartialEq, Default)]
pub(crate) struct TunnelAck{
    pub(crate) channel: u8,
    pub(crate) seq: u8,
    pub(crate) status: StatusCode,
}

impl TunnelRequest {
    pub(crate) fn payload_length(&self)->u16 {
        return 6;
    }

    pub(crate) fn encode(&self, buf: &mut Vec<u8>){
        buf.push(0x6);
        buf.push(self.channel);
        buf.push(self.seq);
        buf.push(0x00); // reserved
        buf.push(self.message_code as u8);
        buf.push(0x00); // reserved
    }

    pub(crate) fn decode(&mut self, buf: &[u8]) -> Result<(), KnxNetIpError> {
        if buf.len() < 0x6usize {
            return Err(KnxNetIpError::MessageTooShort(buf.len()))
        }
        self.channel = buf[1];
        self.seq = buf[2];
        self.message_code = match Message::from_repr(buf[4]) {
            Some(message) => message,
            None => return Err(KnxNetIpError::Unknown)
        };
        return Ok(())
    }
}


impl TunnelAck {
    pub(crate) fn payload_length(&self)->u16 {
        return 4;
    }

    pub(crate) fn encode(&self, buf: &mut Vec<u8>){
        buf.push(0x4);
        buf.push(self.channel);
        buf.push(self.seq);
        buf.push(self.status as u8);
    }

    pub(crate) fn decode(&mut self, buf: &[u8]) -> Result<(), KnxNetIpError> {
        if buf.len() < 0x4{
            return Err(KnxNetIpError::MessageTooShort(buf.len()))
        }
        self.channel = buf[1];
        self.seq = buf[2];
        self.status = match StatusCode::from_repr(buf[3]) {
            Some(status) => status,
            None => return Err(KnxNetIpError::UnknownStatus(buf[1]))
        };
        return Ok(());
    }
}