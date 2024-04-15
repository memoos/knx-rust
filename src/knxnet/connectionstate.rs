use crate::knxnet::hpai::HPAI;
use crate::knxnet::{KnxNetIpError};
use crate::knxnet::status::StatusCode;

#[derive(Debug, PartialEq, Default)]
pub(crate) struct ConnectionstateRequest {
    pub(crate) channel: u8,
    pub(crate) control: HPAI,
}

#[derive(Debug, PartialEq, Default)]
pub(crate) struct ConnectionstateResponse{
    pub(crate) channel: u8,
    pub(crate) status: StatusCode,
}

impl ConnectionstateRequest {
    pub(crate) fn payload_length(&self)->u16 {
        return 2 + HPAI::length();
    }

    pub(crate) fn encode(&self, buf: &mut Vec<u8>){
        buf.push(self.channel);
        buf.push(0); // reserved
        self.control.encode(buf);
    }

    pub(crate) fn decode(&mut self, buf: &[u8]) -> Result<(), KnxNetIpError> {
        if buf.len() < self.payload_length() as usize {
            return Err(KnxNetIpError::MessageTooShort(buf.len()))
        }
        self.channel = buf[0];
        self.control = HPAI::decode(&buf[2..10])?;
        return Ok(());
    }
}


impl ConnectionstateResponse {
    pub(crate) fn payload_length(&self)->u16 {
        return 2;
    }

    pub(crate) fn encode(&self, buf: &mut Vec<u8>){
        buf.push(self.channel);
        buf.push(self.status as u8);
    }

    pub(crate) fn decode(&mut self, buf: &[u8]) -> Result<(), KnxNetIpError> {
        if buf.len() < 2 {
            return Err(KnxNetIpError::MessageTooShort(buf.len()))
        }
        self.channel = buf[0];
        self.status = match StatusCode::from_repr(buf[1]) {
            Some(status) => status,
            None => return Err(KnxNetIpError::UnknownStatus(buf[1]))
        };
        return Ok(());
    }
}