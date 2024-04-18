use crate::knxnet::hpai::HPAI;
use crate::knxnet::{KnxNetIpError};
use crate::knxnet::crd::ConnectionRespType;
use crate::knxnet::cri::ConnectionReqType;
use crate::knxnet::status::StatusCode;

#[derive(Debug, Copy, Clone)]
#[repr(u8)]
pub(crate) enum Layer {
    TunnelLayerData = 0x02,
    TunnelLayerRaw = 0x04,
    TunnelLayerBusmon = 0x80,
}


#[derive(Debug, PartialEq, Default)]
pub(crate) struct ConnectRequest {
    pub(crate) data: HPAI,
    pub(crate) control: HPAI,
    pub(crate) connection_type: ConnectionReqType
}

#[derive(Debug, PartialEq, Default)]
pub(crate) struct ConnectResponse{
    pub(crate) channel: u8,
    pub(crate) status: StatusCode,
    pub(crate) data: HPAI,
    pub(crate) connection_type: ConnectionRespType,
}

impl ConnectRequest {
    pub(crate) fn payload_length(&self)->u16 {
        return self.connection_type.length() + 2 * HPAI::length();
    }

    pub(crate) fn encode(&self, buf: &mut Vec<u8>){
        self.control.encode(buf);
        self.data.encode(buf);
        self.connection_type.encode(buf);
    }

    pub(crate) fn decode(&mut self, buf: &[u8]) -> Result<(), KnxNetIpError> {
        if buf.len() < 2 * HPAI::length() as usize {
            return Err(KnxNetIpError::MessageTooShort(buf.len()))
        }
        self.control = HPAI::decode(&buf[0..8])?;
        self.data = HPAI::decode(&buf[8..16])?;
        self.connection_type = ConnectionReqType::decode(&buf[16..buf.len()])?;
        return Ok(());
    }
}


impl ConnectResponse {
    pub(crate) fn payload_length(&self)->u16 {
        return 2 + if self.status == StatusCode::NoError {HPAI::length() + self.connection_type.length()} else {0};
    }

    pub(crate) fn encode(&self, buf: &mut Vec<u8>){
        buf.push(self.channel);
        buf.push(self.status as u8);
        if self.status == StatusCode::NoError {
            self.data.encode(buf);
            self.connection_type.encode(buf);
        }
    }

    pub(crate) fn decode(&mut self, buf: &[u8]) -> Result<(), KnxNetIpError> {
        if buf.len() < 2{
            return Err(KnxNetIpError::MessageTooShort(buf.len()))
        }
        self.channel = buf[0];
        self.status = match StatusCode::from_repr(buf[1]) {
            Some(status) => status,
            None => return Err(KnxNetIpError::UnknownStatus(buf[1]))
        };

        if self.status == StatusCode::NoError{
            if buf.len() < 2 + HPAI::length() as usize{
                return Err(KnxNetIpError::MessageTooShort(buf.len()))
            }
            self.data = HPAI::decode(&buf[2..10])?;
            self.connection_type = ConnectionRespType::decode(&buf[10..buf.len()])?;
        }
        return Ok(());
    }
}