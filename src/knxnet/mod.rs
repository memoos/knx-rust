pub(crate) mod hpai;
pub mod status;
pub(crate) mod connectionstate;
pub(crate) mod connect;
pub(crate) mod cri;
pub(crate) mod crd;
pub(crate) mod disconnect;
pub(crate) mod tunnel;

use strum_macros::FromRepr;
use thiserror::Error;
use crate::knxnet::status::StatusCode;
use byteorder::{ByteOrder, BigEndian};
use crate::cemi::dpt::DPT;
use crate::knxnet::connect::{ConnectRequest, ConnectResponse};
use crate::knxnet::connectionstate::{ConnectionstateRequest, ConnectionstateResponse};
use crate::knxnet::disconnect::{DisconnectRequest, DisconnectResponse};
use crate::knxnet::tunnel::{TunnelRequest, TunnelAck};

const HEADER_LENGTH: u8 = 0x06;
const KNXNET_VERSION: u8 = 0x10;

/// Errors that can arise here
#[derive(Debug, Error, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum KnxNetIpError {
    /// The peer does not support receiving datagram frames
    #[error("datagram contained an error {0}")]
    ErrorStatus(StatusCode),
    #[error("unknown status code {0}")]
    UnknownStatus(u8),
    #[error("unknown connection type code {0}")]
    UnknownConnectionType(u8),
    #[error("datagram was too short (len {0})")]
    MessageTooShort(usize),
    #[error("datagram header was too short (len {0})")]
    HeaderTooShort(usize),
    #[error("unexpected header size {0}")]
    UnknownHeaderSize(u8),
    #[error("size in header {0} does not match message length {1}")]
    InvalidHeaderSize(u16, usize),
    #[error("unexpected KnxNet/IP version {0}")]
    UnknownVersion(u8),
    #[error("unknown service {0:#x}")]
    UnknownService(u16),
    #[error("unknown error")]
    Unknown,
    #[error("Not implemented")]
    NotImplemented,
    #[error("unexpected size detected")]
    InvalidSize,
    #[error("unknown layer {0:#x}")]
    UnknownLayer(u8),
}


#[derive(PartialEq, FromRepr, Debug)]
#[repr(u16)]
pub enum Service<D: DPT + Default = ()> {
    SearchRequest = 0x0201,
    SearchResponse = 0x0202,
    DescriptionRequest = 0x0203,
    DescriptionResponse = 0x0204,
    ConnectRequest(ConnectRequest) = 0x0205,
    ConnectResponse(ConnectResponse) = 0x0206,
    ConnectionstateRequest(ConnectionstateRequest) = 0x0207,
    ConnectionstateResponse(ConnectionstateResponse) = 0x0208,
    DisconnectRequest(DisconnectRequest) = 0x0209,
    DisconnectResponse(DisconnectResponse) = 0x020A,
    TunnelRequest(TunnelRequest<D>) = 0x0420,
    TunnelAck(TunnelAck) = 0x0421,
    DeviceConfigurationRequest = 0x0310,
    DeviceConfigurationAck = 0x0311,
    RoutingIndication = 0x0530,
    RoutingLostMessage = 0x0531,
}

impl<D:DPT+Default> Service<D> {
    pub(crate) fn length(&self)->u16 {
        return 6 + match self {
            Self::ConnectRequest(r) => r.payload_length(),
            Self::ConnectResponse(r) => r.payload_length(),
            Self::ConnectionstateRequest(r) => r.payload_length(),
            Self::ConnectionstateResponse(r) => r.payload_length(),
            Self::DisconnectRequest(r) => r.payload_length(),
            Self::DisconnectResponse(r) => r.payload_length(),
            Self::TunnelRequest(r) => r.payload_length(),
            Self::TunnelAck(a) => a.payload_length(),
            _ => 0
        }
    }

    fn identifier(&self) -> u16 {
        // SAFETY: Because `Self` is marked `repr(u16)`, its layout is a `repr(C)` `union`
        // between `repr(C)` structs, each of which has the `u16` discriminant as its first
        // field, so we can read the discriminant without offsetting the pointer.
        unsafe { *<*const _>::from(self).cast::<u16>() }
    }

    pub(crate) fn encoded(&self) -> Vec<u8>{
        let mut buf = Vec::<u8>::with_capacity(self.length() as usize);
        buf.push(HEADER_LENGTH); // header length
        buf.push(KNXNET_VERSION); // version
        buf.extend(self.identifier().to_be_bytes());
        buf.extend(self.length().to_be_bytes());
        match self {
            Self::ConnectRequest(r) => r.encode(&mut buf),
            Self::ConnectResponse(r) => r.encode(&mut buf),
            Self::ConnectionstateRequest(r) => r.encode(&mut buf),
            Self::ConnectionstateResponse(r) => r.encode(&mut buf),
            Self::DisconnectRequest(r) => r.encode(&mut buf),
            Self::DisconnectResponse(r) => r.encode(&mut buf),
            Self::TunnelRequest(r) => r.encode(&mut buf),
            Self::TunnelAck(a) => a.encode(&mut buf),
            _ => {}
        }
        return buf;
    }

    pub(crate) fn decoded(buf: &[u8]) -> Result<Service<D>, KnxNetIpError> {
        if buf.len() < 6 {
            return Err(KnxNetIpError::HeaderTooShort(buf.len()))
        }
        if buf[0] != HEADER_LENGTH {
            return Err(KnxNetIpError::UnknownHeaderSize(buf[0]))
        }
        if buf[1] != KNXNET_VERSION {
            return Err(KnxNetIpError::UnknownVersion(buf[1]))
        }

        let service_identifier = BigEndian::read_u16(&buf[2..4]);
        let total_size = BigEndian::read_u16(&buf[4..6]);
        if total_size as usize != buf.len() {
            return Err(KnxNetIpError::InvalidHeaderSize(total_size, buf.len()))
        }
        return match Service::from_repr(service_identifier){
            None => Err(KnxNetIpError::UnknownService(service_identifier)),
            Some(mut service) => {
                match service {
                    Self::ConnectRequest(mut r) => {
                        r.decode(&buf[6..])?;
                        Ok(Self::ConnectRequest(r))
                    },
                    Self::ConnectResponse(mut r) => {
                        r.decode(&buf[6..])?;
                        Ok(Self::ConnectResponse(r))
                    },
                    Self::ConnectionstateRequest(mut r) => {
                        r.decode(&buf[6..])?;
                        Ok(Self::ConnectionstateRequest(r))
                    },
                    Self::ConnectionstateResponse(mut r) => {
                        r.decode(&buf[6..])?;
                        Ok(Self::ConnectionstateResponse(r))
                    },
                    Self::DisconnectRequest(mut r) => {
                        r.decode(&buf[6..])?;
                        Ok(Self::DisconnectRequest(r))
                    },
                    Self::DisconnectResponse(mut r) => {
                        r.decode(&buf[6..])?;
                        Ok(Self::DisconnectResponse(r))
                    },
                    Self::TunnelRequest(ref mut r) => {
                        r.decode(&buf[6..])?;
                        Ok(service)
                    }
                    Self::TunnelAck(ref mut r) => {
                        r.decode(&buf[6..])?;
                        Ok(service)
                    }
                    _ => Ok(service)
                }
            }
        };
    }

}

#[cfg(test)]
mod tests {
    use crate::cemi::apdu::Apdu;
    use crate::cemi::l_data::{Acknowledge, AddressType, Confirmation, FrameFormat, FrameType, LData, Priority, Repetition, SystemBroadcast};
    use crate::cemi::Message;
    use crate::knxnet::connect::{ConnectRequest, ConnectResponse};
    use crate::knxnet::Service;
    use crate::knxnet::connectionstate::ConnectionstateRequest;
    use crate::knxnet::connectionstate::ConnectionstateResponse;
    use crate::knxnet::crd::ConnectionRespType;
    use crate::knxnet::cri::{ConnectionReqType, TunnelingLayer};
    use crate::knxnet::disconnect::{DisconnectRequest, DisconnectResponse};
    use crate::knxnet::hpai::{HPAI, Protocol};
    use crate::knxnet::status::StatusCode;
    use crate::knxnet::tunnel::{TunnelAck, TunnelRequest};

    #[test]
    fn t_service_length() {
        assert_eq!(Service::<()>::ConnectRequest(ConnectRequest::default()).length(), 26);
        assert_eq!(Service::<()>::ConnectResponse(ConnectResponse::default()).length(), 20);
        assert_eq!(Service::<()>::ConnectionstateRequest(ConnectionstateRequest::default()).length(), 16);
        assert_eq!(Service::<()>::ConnectionstateResponse(ConnectionstateResponse::default()).length(), 8);
        assert_eq!(Service::<()>::DisconnectRequest(DisconnectRequest::default()).length(), 16);
        assert_eq!(Service::<()>::DisconnectResponse(DisconnectResponse::default()).length(), 8);
        assert_eq!(Service::<()>::TunnelRequest(TunnelRequest::<()>::default()).length(), 12);
        assert_eq!(Service::<()>::TunnelAck(TunnelAck::default()).length(), 10);
        assert_eq!(Service::<()>::SearchRequest.length(), 6); // TODO change after implementation
    }

    #[test]
    fn t_service_encode() {
        assert_eq!(Service::<()>::ConnectRequest(ConnectRequest{
            control: HPAI{
                protocol: Protocol::Udp4Protocol,
                port: 50100,
                address: [192,168,200,12],
            },
            data: HPAI{
                protocol: Protocol::Udp4Protocol,
                port: 50100,
                address: [192,168,200,20],
            },
            connection_type: ConnectionReqType::TunnelConnection {
                layer: TunnelingLayer::TunnelLinkLayer,
            }
        }).encoded(), vec![0x06, 0x10, 0x02, 0x05, 0x00, 0x1A, 0x08, 0x01, 192, 168, 200, 12, 0xC3, 0xB4, 0x08, 0x01, 192, 168, 200, 20, 0xC3, 0xB4, 0x04, 0x04, 0x02, 0x00]);
        assert_eq!(Service::<()>::ConnectResponse(ConnectResponse{
            data: HPAI{
                protocol: Protocol::Udp4Protocol,
                port: 50100,
                address: [192,168,200,20],
            },
            connection_type: ConnectionRespType::TunnelConnection {
                address: (2<<11)|(1<<8)|(10),
            },
            status: StatusCode::NoError,
            channel: 21,
        }).encoded(), vec![0x06, 0x10, 0x02, 0x06, 0x00, 0x14, 0x15, 0x00, 0x08, 0x01, 192, 168, 200, 20, 0xC3, 0xB4, 0x04, 0x04, 0x11, 0x0A]);
        assert_eq!(Service::<()>::ConnectionstateRequest(ConnectionstateRequest{
            channel: 21,
            control: HPAI{
                protocol: Protocol::Udp4Protocol,
                port: 50100,
                address: [192,168,200,12]
            }
        }).encoded(), vec![0x06, 0x10, 0x02, 0x07, 0x00, 0x10, 0x15, 0x00, 0x08, 0x01, 192, 168, 200, 12, 0xC3, 0xB4]);
        assert_eq!(Service::<()>::ConnectionstateResponse(ConnectionstateResponse {
            channel: 21,
            status: StatusCode::NoError
        }).encoded(), vec![0x06, 0x10, 0x02, 0x08, 0x00, 0x08, 0x15, 0x00]);
        assert_eq!(Service::<()>::DisconnectRequest(DisconnectRequest{
            channel: 21,
            control: HPAI{
                protocol: Protocol::Udp4Protocol,
                port: 50100,
                address: [192,168,200,12]
            }
        }).encoded(), vec![0x06, 0x10, 0x02, 0x09, 0x00, 0x10, 0x15, 0x00, 0x08, 0x01, 192, 168, 200, 12, 0xC3, 0xB4]);
        assert_eq!(Service::TunnelRequest(TunnelRequest{
            channel:17,
            seq: 0,
            data: Message::LDataInd(vec![],LData{
               frame_type: FrameType::Standard,
               repetition: Repetition::NoRepeat,
               system_broadcast: SystemBroadcast::Broadcast,
               priority: Priority::Low,
               acknowledge: Acknowledge::NoAcknowledge,
               confirmation: Confirmation::NoError,
               destination_address_type: AddressType::Group,
               hop_count: 6,
               frame_format: FrameFormat::Standard,
               source: 0x1101, // 1.1.1
               destination: 10, // 0/10
               control: false,
               numbered: false,
               seq: 0,
               data: Apdu::GroupValueWrite(vec![0x03, 0xD4]),
           }) }).encoded(), vec![0x6, 0x10, 0x4, 0x20, 0x0, 0x17, 0x4, 0x11, 0, 0, 0x29, 0, 0xBC, 0xE0, 0x11, 0x1, 0, 0x0a, 0x3, 0x0, 0x80, 3, 0xD4]);
        assert_eq!(Service::TunnelRequest(TunnelRequest{
            channel:17,
            seq: 0,
            data: Message::LDataReq(vec![],LData{
                frame_type: FrameType::Standard,
                repetition: Repetition::NoRepeat,
                system_broadcast: SystemBroadcast::Broadcast,
                priority: Priority::Low,
                acknowledge: Acknowledge::NoAcknowledge,
                confirmation: Confirmation::NoError,
                destination_address_type: AddressType::Group,
                hop_count: 6,
                frame_format: FrameFormat::Standard,
                source: 0x0, // 1.1.1
                destination: 10, // 0/10
                control: false,
                numbered: false,
                seq: 0,
                data: Apdu::<()>::GroupValueRead,
            }) }).encoded(), vec![0x6, 0x10, 0x4, 0x20, 0x0, 0x15, 0x4, 0x11, 0, 0, 0x11, 0, 0xBC, 0xE0, 0x0, 0x0, 0, 0x0a, 0x1, 0x0, 0x0]);
        assert_eq!(Service::<()>::TunnelAck(TunnelAck{
            channel:17,
            seq: 141,
            status: StatusCode::NoError
        }).encoded(), vec![0x06, 0x10, 0x04, 0x21, 0x00, 0x0a, 0x04, 0x11, 0x8D, 0]);
    }

    #[test]
    fn t_service_decode() {
        assert_eq!(Service::<()>::decoded(&vec![0x06, 0x10, 0x02, 0x05, 0x00, 0x1A, 0x08, 0x01, 192, 168, 200, 12, 0xC3, 0xB4, 0x08, 0x01, 192, 168, 200, 20, 0xC3, 0xB4, 0x04, 0x04, 0x02, 0x00]),
                   Ok(Service::ConnectRequest(ConnectRequest{
                       control: HPAI{
                           protocol: Protocol::Udp4Protocol,
                           port: 50100,
                           address: [192,168,200,12],
                       },
                       data: HPAI{
                           protocol: Protocol::Udp4Protocol,
                           port: 50100,
                           address: [192,168,200,20],
                       },
                       connection_type: ConnectionReqType::TunnelConnection {
                           layer: TunnelingLayer::TunnelLinkLayer,
                       }
                   })));
        assert_eq!(Service::<()>::decoded(&vec![0x06, 0x10, 0x02, 0x06, 0x00, 0x14, 0x15, 0x00, 0x08, 0x01, 192, 168, 200, 20, 0xC3, 0xB4, 0x04, 0x04, 0x11, 0x0A]),
                   Ok(Service::ConnectResponse(ConnectResponse{
                       data: HPAI{
                           protocol: Protocol::Udp4Protocol,
                           port: 50100,
                           address: [192,168,200,20],
                       },
                       connection_type: ConnectionRespType::TunnelConnection {
                           address: (2<<11)|(1<<8)|(10),
                       },
                       status: StatusCode::NoError,
                       channel: 21,
                   })));
        assert_eq!(Service::<()>::decoded(&vec![0x06, 0x10, 0x02, 0x06, 0x00, 0x08, 0x00, 0x24]),
                   Ok(Service::ConnectResponse(ConnectResponse{
                       data: HPAI{
                           protocol: Protocol::Udp4Protocol,
                           port: 0,
                           address: [0,0,0,0],
                       },
                       connection_type: ConnectionRespType::TunnelConnection {
                           address: (0),
                       },
                       status: StatusCode::ErrNoMoreConnections,
                       channel: 0,
                   })));
        assert_eq!(Service::<()>::decoded(&vec![0x06, 0x10, 0x02, 0x07, 0x00, 0x10, 0x15, 0x00, 0x08, 0x01, 192, 168, 200, 12, 0xC3, 0xB4]),
                   Ok(Service::ConnectionstateRequest(ConnectionstateRequest{
                       channel: 21,
                       control: HPAI{
                           protocol: Protocol::Udp4Protocol,
                           port: 50100,
                           address: [192,168,200,12]
                       }
                   })));
        assert_eq!(Service::<()>::decoded(&vec![0x06, 0x10, 0x02, 0x08, 0x00, 0x08, 0x15, 0x00]),
                   Ok(Service::ConnectionstateResponse(ConnectionstateResponse {
                        channel: 21,
                        status: StatusCode::NoError
                    })));
        assert_eq!(Service::<()>::decoded(&vec![0x06, 0x10, 0x02, 0x09, 0x00, 0x10, 0x15, 0x00, 0x08, 0x01, 192, 168, 200, 12, 0xC3, 0xB4]),
                   Ok(Service::DisconnectRequest(DisconnectRequest{
                       channel: 21,
                       control: HPAI{
                           protocol: Protocol::Udp4Protocol,
                           port: 50100,
                           address: [192,168,200,12]
                       }
                   })));
        assert_eq!(Service::<()>::decoded(&vec![0x06, 0x10, 0x02, 0x0A, 0x00, 0x08, 0x15, 0x00]),
                   Ok(Service::DisconnectResponse(DisconnectResponse {
                       channel: 21,
                       status: StatusCode::NoError
                   })));

        assert_eq!(Service::<Vec<u8>>::decoded(&vec![0x6, 0x10, 0x4, 0x20, 0x0, 0x17, 0x4, 0x11, 0, 0, 0x29, 0, 0xBC, 0xE0, 0x11, 0x1, 0, 0x0a, 0x3, 0x0, 0x80, 3, 0xD4]),
                   Ok(Service::TunnelRequest(TunnelRequest{channel:17, seq: 0, data: Message::LDataInd(vec![],LData{
                       frame_type: FrameType::Standard,
                       repetition: Repetition::NoRepeat,
                       system_broadcast: SystemBroadcast::Broadcast,
                       priority: Priority::Low,
                       acknowledge: Acknowledge::NoAcknowledge,
                       confirmation: Confirmation::NoError,
                       destination_address_type: AddressType::Group,
                       hop_count: 6,
                       frame_format: FrameFormat::Standard,
                       source: 0x1101, // 1.1.1
                       destination: 10, // 0/10
                       control: false,
                       numbered: false,
                       seq: 0,
                       data: Apdu::GroupValueWrite(vec![0x03, 0xD4]),
                   }) })));
        assert_eq!(Service::<Vec<u8>>::decoded(&vec![0x06, 0x10, 0x04, 0x21, 0x00, 0x0a, 0x04, 0x11, 0x8D, 0]),
                   Ok(Service::TunnelAck(TunnelAck{channel:17, seq: 141, status: StatusCode::NoError})));
    }

    #[test]
    fn t_service_decode_errors() {
        assert_eq!(Service::<()>::decoded(&vec![0x02, 0x10, 0x02, 0x07, 0x00, 0x10, 0x15, 0x00, 0x08, 0x01, 192, 168, 200, 12, 0xC3, 0xB4]).unwrap_err().to_string(),
            "unexpected header size 2");
        assert_eq!(Service::<()>::decoded(&vec![0x06]).unwrap_err().to_string(),
                   "datagram header was too short (len 1)");
        assert_eq!(Service::<()>::decoded(&vec![0x06, 0x11, 0x02, 0x07, 0x00, 0x10, 0x15, 0x00, 0x08, 0x01, 192, 168, 200, 12, 0xC3, 0xB4]).unwrap_err().to_string(),
                   "unexpected KnxNet/IP version 17");
        assert_eq!(Service::<()>::decoded(&vec![0x06, 0x10, 0x01, 0x07, 0x00, 0x10, 0x15, 0x00, 0x08, 0x01, 192, 168, 200, 12, 0xC3, 0xB4]).unwrap_err().to_string(),
                   "unknown service 0x107");
        assert_eq!(Service::<()>::decoded(&vec![0x06, 0x10, 0x02, 0x07, 0x00, 0x14]).unwrap_err().to_string(),
                   "size in header 20 does not match message length 6");
        assert_eq!(Service::<()>::decoded(&vec![0x06, 0x10, 0x02, 0x07, 0x00, 0x06]).unwrap_err().to_string(),
                   "datagram was too short (len 0)");
        assert_eq!(Service::<()>::decoded(&vec![0x06, 0x10, 0x02, 0x07, 0x00, 0x06]).unwrap_err().to_string(),
                   "datagram was too short (len 0)");
        assert_eq!(Service::<()>::decoded(&vec![0x06, 0x10, 0x02, 0x08, 0x00, 0x08, 0x00, 0xff]).unwrap_err().to_string(),
                   "unknown status code 255");
    }
}
