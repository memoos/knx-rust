use crate::cemi;
use crate::cemi::Message;
use crate::dpt::DPT;
use crate::knxnet::{KnxNetIpError};
use crate::knxnet::status::StatusCode;

#[derive(Debug, PartialEq,Default)]
pub(crate) struct TunnelRequest<D: DPT+Default> {
    pub(crate) channel: u8,
    pub(crate) seq: u8,
    pub(crate) data: cemi::Message<D>,
}

#[derive(Debug, PartialEq, Default)]
pub(crate) struct TunnelAck{
    pub(crate) channel: u8,
    pub(crate) seq: u8,
    pub(crate) status: StatusCode,
}

impl<D:DPT+Default> TunnelRequest<D> {
    pub(crate) fn payload_length(&self)->u16 {
        return (4 + self.data.length()) as u16;
    }

    pub(crate) fn encode(&self, buf: &mut Vec<u8>){
        buf.push(0x4);
        buf.push(self.channel);
        buf.push(self.seq);
        buf.push(0x00); // reserved
        self.data.encode(buf);
    }

    pub(crate) fn decode(&mut self, buf: &[u8]) -> Result<(), KnxNetIpError> {
        if buf.len() < 0x6usize {
            return Err(KnxNetIpError::MessageTooShort(buf.len()))
        }
        self.channel = buf[1];
        self.seq = buf[2];
        self.data = Message::decode(&buf[4..buf.len()])?;
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