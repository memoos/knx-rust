use crate::knxnet::hpai::HPAI;
use crate::knxnet::{KnxNetIpError};
use crate::knxnet::crd::ConnectionRespType;
use crate::knxnet::cri::ConnectionReqType;
use crate::knxnet::status::StatusCode;

#[derive(Debug, PartialEq, Default)]
pub(crate) struct DisconnectRequest {
    pub(crate) channel: u8,
    pub(crate) control: HPAI,
}

#[derive(Debug, PartialEq, Default)]
pub(crate) struct DisconnectResponse{
    pub(crate) channel: u8,
    pub(crate) status: StatusCode,
}

impl DisconnectRequest {
    pub(crate) fn payload_length(&self)->u16 {
        return 2 + HPAI::length();
    }

    pub(crate) fn encode(&self, buf: &mut Vec<u8>){
        buf.push(self.channel);
        buf.push(0x00); //reserved
        self.control.encode(buf);
    }

    pub(crate) fn decode(&mut self, buf: &[u8]) -> Result<(), KnxNetIpError> {
        if buf.len() < 2 + HPAI::length() as usize {
            return Err(KnxNetIpError::MessageTooShort(buf.len()))
        }
        self.channel = buf[0];
        self.control = HPAI::decode(&buf[2..10])?;
        return Ok(());
    }
}


impl DisconnectResponse {
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