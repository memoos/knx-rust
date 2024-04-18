//runtime facing functions:

// get data to be transmitted next
// data was transmitted
// get next time event
// handle next time event
// handle received data -> returns a cemi or none
// send data with future<Result<>>



use std::cmp::min;
use std::collections::VecDeque;
use std::ops::Add;
use std::time::{Duration, Instant};
use nom::error::ErrorKind::Many0;
use crate::cemi::apdu::Apdu;
use crate::cemi::dpt::DPT;
use crate::cemi::l_data::LData;
use crate::cemi::Message;
use crate::group_event::{GroupEvent, GroupEventType};
use crate::knxnet;
use crate::knxnet::connectionstate::ConnectionstateRequest;
use crate::knxnet::{cri, KnxNetIpError, Service};
use crate::knxnet::hpai::{HPAI, Protocol};
use crate::knxnet::status::StatusCode;
use crate::knxnet::tunnel::TunnelAck;

pub struct TunnelConnectionConfig {
    resent_interval: Duration,
    connect_response_timeout: Duration,
    heartbeat_response_timeout: Duration,
    heartbeat_interval: Duration,
}

impl TunnelConnectionConfig {
    pub fn default() -> TunnelConnectionConfig {
        TunnelConnectionConfig{
            resent_interval: Duration::from_millis(50),
            heartbeat_interval: Duration::from_secs(60),
            connect_response_timeout: Duration::from_secs(1),
            heartbeat_response_timeout: Duration::from_secs(10),
        }
    }
}

pub struct TunnelConnection {
    initialized: bool,
    channel: u8,
    host_info: HPAI,
    outbound_seq: u8,
    inbound_seq: u8,
    out_queue: VecDeque<(Vec<u8>, bool)>,
    message_pending: bool,
    next_resent: Instant,
    next_timeout: Instant,
    next_heartbeat: Instant,
    config: TunnelConnectionConfig,
}


impl TunnelConnection {
    /// Create an TunnelConnReq
    pub fn new(ipv4: [u8;4], port: u16, config: TunnelConnectionConfig) -> TunnelConnection {
        let host_info = HPAI::new(Protocol::Udp4Protocol, ipv4, port);
        let tunnel_request: Service<()> = Service::ConnectRequest(crate::knxnet::connect::ConnectRequest{
            data: host_info,
            control: host_info,
            connection_type: cri::ConnectionReqType::TunnelConnection {layer: cri::TunnelingLayer::TunnelLinkLayer}
        });

        let buf = tunnel_request.encoded();

        return TunnelConnection{
            initialized: false,
            channel: 0,
            next_resent: Instant::now().add(config.resent_interval),
            next_timeout: Instant::now().add(config.connect_response_timeout),
            next_heartbeat: Instant::now().add(config.heartbeat_interval),
            config,
            outbound_seq: 0,
            inbound_seq: 0,
            out_queue: VecDeque::from(vec![(buf, true)]),
            host_info,
            message_pending: true,
        };

    }

    pub fn send<T: DPT+Default>(&mut self, ev: GroupEvent<T>) ->() {
        let msg = match ev.event_type {
            GroupEventType::GroupValueRead => Message::<T>::LDataReq(vec![], LData::<T>{data: Apdu::GroupValueRead, destination: ev.address , ..LData::<T>::default()}),
            GroupEventType::GroupValueWrite => Message::<T>::LDataReq(vec![], LData::<T>{data: Apdu::GroupValueWrite(ev.data), destination: ev.address , ..LData::<T>::default()}),
            GroupEventType::GroupValueResponse => Message::<T>::LDataReq(vec![], LData::<T>{data: Apdu::GroupValueResponse(ev.data), destination: ev.address , ..LData::<T>::default()}),
        };
        let req = Service::TunnelRequest(knxnet::tunnel::TunnelRequest{
            channel: self.channel,
            seq: self.outbound_seq,
            data: msg,
        });
        self.outbound_seq += 1;
        self.out_queue.push_back((req.encoded(), true))
    }

    pub fn get_outbound_data(&mut self) -> Option<&[u8]> {
        if self.message_pending && !self.out_queue.is_empty() {
            self.message_pending = false;
            if self.out_queue[0].1 {
                self.next_resent = Instant::now().add(self.config.resent_interval);
            }
            Some(&self.out_queue[0].0)
        } else {
            if !self.message_pending && !self.out_queue.is_empty() && !self.out_queue[0].1{
                self.remove_first_message()
            }
            None
        }
    }

    fn remove_first_message(&mut self){
        self.out_queue.pop_front();
        if !self.out_queue.is_empty(){
            self.message_pending = true
        }
    }

    fn handle_outbount_send(&mut self){
        self.remove_first_message();
    }

    pub fn get_next_time_event(&self) -> Instant{
        return min(self.next_heartbeat, min(self.next_resent, self.next_timeout))
    }

    pub fn handle_time_events(&mut self) -> () {
        if self.next_timeout < Instant::now() && !self.out_queue.is_empty() {
            // if we are not initialized yet there is nothing we can do
            if !self.initialized {
                // we might want to handle the connect Future here to return an error
                panic!("Failed to initialize tunnel connection")
            }
            // outbound message timed out so skip sending it
            self.remove_first_message();
            self.next_timeout = Instant::now().add(self.config.heartbeat_response_timeout);
        }
        if self.next_heartbeat < Instant::now() && self.initialized{
            self.send_connection_state_request();
            self.next_heartbeat += self.config.heartbeat_interval;
        }
        if self.next_resent < Instant::now() && !self.out_queue.is_empty() && self.out_queue[0].1 {
            self.message_pending = true
        }
    }

    pub fn handle_inbound_message(&mut self, data: &[u8]) -> Option<GroupEvent::<Vec<u8>>> {
        let service = Service::<Vec<u8>>::decoded(data);
        println!("inbound {:?}", service);
        let service = match service {
            Ok(s) => s,
            Err(e) => return None
        };
        return match service {
            Service::ConnectResponse(connect) => {
                if connect.status == StatusCode::NoError {
                    self.inbound_seq = 0;
                    self.outbound_seq = 0;
                    self.channel = connect.channel;
                    self.handle_outbount_send();
                    self.initialized = true;
                }
                None
            }
            Service::ConnectionstateResponse(con_res) => {
                if con_res.status == StatusCode::NoError {
                    self.handle_outbount_send()
                }
                None
            }
            Service::TunnelAck(tack) => {
                None
            },
            Service::TunnelRequest(treq) => {
                self.out_queue.push_back((Service::<()>::TunnelAck(
                    TunnelAck{
                        seq: treq.seq,
                        channel: treq.channel,
                        status: StatusCode::NoError,
                    }).encoded(), false));
                self.message_pending = true;

                match treq.data {
                    Message::LDataInd(i, d) => {
                        match d.data {
                            Apdu::GroupValueRead => {
                                Some(GroupEvent::<Vec<u8>> {
                                    data: vec![],
                                    address: d.destination,
                                    event_type: GroupEventType::GroupValueRead,
                                })
                            }
                            Apdu::GroupValueWrite(data) | Apdu::GroupValueResponse(data) => {
                                Some(GroupEvent::<Vec<u8>> {
                                    data,
                                    address: d.destination,
                                    event_type: GroupEventType::GroupValueRead,
                                })
                            }
                            _ => None
                        }
                    }
                    _ => None
                }
            },
            _ => None,
        }

    }

    fn send_connection_state_request(&mut self){
        let req: Service<()> = Service::ConnectionstateRequest(ConnectionstateRequest{
            channel: self.channel,
            control: self.host_info
        });

        self.out_queue.push_back((req.encoded(), true))
    }
}