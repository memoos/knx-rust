//runtime facing functions:

// get data to be transmitted next
// data was transmitted
// get next time event
// handle next time event
// handle received data -> returns a cemi or none
// send data with future<Result<>>



use std::cmp::{min, PartialEq};
use std::collections::VecDeque;
use std::ops::Add;
use std::time::{Duration, Instant};
use strum_macros::FromRepr;
use crate::cemi::apdu::Apdu;
use crate::dpt::DPT;
use crate::cemi::l_data::LData;
use crate::cemi::Message;
use crate::group_event::{GroupEvent, GroupEventType};
use crate::knxnet;
use crate::knxnet::connectionstate::ConnectionstateRequest;
use crate::knxnet::{cri, KnxNetIpError, Service};
use crate::knxnet::disconnect::{DisconnectRequest, DisconnectResponse};
use crate::knxnet::hpai::{HPAI, Protocol};
use crate::knxnet::status::StatusCode;
use crate::knxnet::tunnel::TunnelAck;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct TunnelConnectionConfig {
    resent_interval: Duration,
    response_timeout: Duration,
    heartbeat_response_timeout: Duration,
    heartbeat_interval: Duration,
}

impl TunnelConnectionConfig {
    pub fn default() -> TunnelConnectionConfig {
        TunnelConnectionConfig{
            resent_interval: Duration::from_millis(1000),
            heartbeat_interval: Duration::from_secs(60),
            response_timeout: Duration::from_millis(1500),
            heartbeat_response_timeout: Duration::from_secs(10),
        }
    }
}

struct OutMessage {
    data: Vec<u8>,
    need_ack: bool,
    retried: u8,
}

#[derive(FromRepr, Debug, Copy, Clone, PartialEq)]
enum TunnelConnectionState {
    Disconnected,
    Connecting,
    Connected,
    Disconnecting,
}


pub struct TunnelConnection {
    state: TunnelConnectionState,
    awaiting_heartbeat_response: bool,
    channel: u8,
    host_info: HPAI,
    outbound_seq: u8,
    inbound_seq: u8,
    out_queue: VecDeque<OutMessage>,
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


        let mut con = TunnelConnection{
            state: TunnelConnectionState::Disconnected,
            awaiting_heartbeat_response: false,
            channel: 0,
            next_resent: Instant::now().add(config.resent_interval),
            next_timeout: Instant::now().add(config.response_timeout),
            next_heartbeat: Instant::now().add(config.heartbeat_interval),
            config,
            outbound_seq: 0,
            inbound_seq: 0,
            out_queue: VecDeque::from(vec![]),
            host_info,
            message_pending: true,
        };
        con.send_connect_request();
        con
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
        self.push_out_message(OutMessage{data: req.encoded(), need_ack: true, retried:0});
    }

    pub fn get_outbound_data(&mut self) -> Option<&[u8]> {
        loop {
            if self.message_pending && !self.out_queue.is_empty() {
                self.message_pending = false;
                if self.out_queue[0].need_ack {
                    self.next_resent = Instant::now().add(self.config.resent_interval);
                } else {
                    self.next_resent = Instant::now().add(self.config.response_timeout);
                }
                self.out_queue[0].retried += 1;
                return Some(&self.out_queue[0].data)
            } else {
                if !self.message_pending && !self.out_queue.is_empty() && !self.out_queue[0].need_ack{
                    self.remove_first_message();
                    continue;
                }
                return None
            }
        }
    }

    fn remove_first_message(&mut self){
        self.out_queue.pop_front();
        if !self.out_queue.is_empty(){
            self.message_pending = true;
            self.next_timeout = Instant::now().add(self.config.response_timeout);
        } else {
            // effectively disable resend and timeout by setting it bigger than heartbeat
            self.next_resent = Instant::now().add(self.config.heartbeat_interval);
            self.next_timeout = Instant::now().add(self.config.heartbeat_interval);
        }
    }

    fn handle_outbount_send(&mut self){
        self.remove_first_message();
    }

    pub fn get_next_time_event(&self) -> Instant{
        return min(self.next_heartbeat, min(self.next_resent, self.next_timeout))
    }

    pub fn connected(&self) -> bool {return self.state == TunnelConnectionState::Connected}

    pub fn handle_time_events(&mut self) -> () {
        if self.next_timeout < Instant::now() && !self.out_queue.is_empty() {
            match self.state {
                TunnelConnectionState::Connecting => {
                    // we might want to handle the connect Future here to return an error
                    panic!("Failed to initialize tunnel connection")
                }
                TunnelConnectionState::Disconnected | TunnelConnectionState::Connected => {
                    // outbound message timed out so skip sending it
                    self.remove_first_message();
                }
                // in case we don't get a response for disconnection connection is probably already lost
                TunnelConnectionState::Disconnecting => {
                    self.remove_first_message();
                    self.state = TunnelConnectionState::Disconnected;
                    self.send_connect_request();
                }
            }
        }
        if self.next_heartbeat < Instant::now() && self.state == TunnelConnectionState::Connected{
            if self.awaiting_heartbeat_response {
                //we did not receive a heartbeat response for 2 periods (120s) so disconnect
                self.send_disconnect_request();
            } else {
                self.send_connection_state_request();
                self.awaiting_heartbeat_response = true;
            }
            self.next_heartbeat += self.config.heartbeat_interval;
        }
        if self.next_resent < Instant::now() && !self.out_queue.is_empty() && self.out_queue[0].need_ack {
            // set message back to due to send
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
            Service::DisconnectRequest(req) => {
                if req.channel == self.channel {
                    self.push_out_message(OutMessage{
                        data: Service::<()>::DisconnectResponse(
                            DisconnectResponse{
                                channel: self.channel,
                                status: StatusCode::NoError,
                            }).encoded(),
                        need_ack: false,
                        retried: 0,
                    });
                    self.state = TunnelConnectionState::Disconnected;
                }
                None
            },
            Service::DisconnectResponse(resp) => {
                if resp.channel == self.channel && resp.status == StatusCode::NoError{
                    self.handle_outbount_send();
                    self.state = TunnelConnectionState::Disconnected;
                    self.send_connect_request();
                }
                None
            }
            Service::ConnectResponse(connect) => {
                if connect.status == StatusCode::NoError {
                    self.inbound_seq = 0;
                    self.outbound_seq = 0;
                    self.channel = connect.channel;
                    self.handle_outbount_send();
                    self.state = TunnelConnectionState::Connected;
                }
                None
            }
            Service::ConnectionstateResponse(con_res) => {
                if con_res.status == StatusCode::NoError {
                    self.awaiting_heartbeat_response = false;
                    self.handle_outbount_send()
                } else {
                    self.awaiting_heartbeat_response = false;
                    self.handle_outbount_send();
                    self.state = TunnelConnectionState::Disconnected;
                    self.send_connect_request();
                }
                None
            }
            Service::TunnelAck(tack) => {
                if tack.status == StatusCode::NoError {
                    self.handle_outbount_send()
                }
                None
            },
            Service::TunnelRequest(treq) => {
                //only messages with the expected seq or one less should be accepted (and thereby acked). See 03/08/04 Tunneling 2.6
                if !(self.inbound_seq == treq.seq || self.inbound_seq == treq.seq.wrapping_add(1)) || self.channel != treq.channel{
                    return None
                }
                self.push_out_message(OutMessage{
                    data: Service::<()>::TunnelAck(
                        TunnelAck{
                            seq: treq.seq,
                            channel: treq.channel,
                            status: StatusCode::NoError,
                        }).encoded(),
                    need_ack: false,
                    retried: 0,
                });
                self.inbound_seq = treq.seq.wrapping_add(1);

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
                            Apdu::GroupValueResponse(data) => {
                                Some(GroupEvent::<Vec<u8>> {
                                    data,
                                    address: d.destination,
                                    event_type: GroupEventType::GroupValueResponse,
                                })
                            }
                            Apdu::GroupValueWrite(data) => {
                                Some(GroupEvent::<Vec<u8>> {
                                    data,
                                    address: d.destination,
                                    event_type: GroupEventType::GroupValueWrite,
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

    fn push_out_message(&mut self, msg: OutMessage)  {
        if self.out_queue.is_empty() {
            self.message_pending = true;
            self.next_timeout = Instant::now().add(self.config.response_timeout)
        }
        self.out_queue.push_back(msg)
    }

    fn send_connection_state_request(&mut self){
        let req: Service<()> = Service::ConnectionstateRequest(ConnectionstateRequest{
            channel: self.channel,
            control: self.host_info
        });

        self.push_out_message(OutMessage {
            data: req.encoded(),
            need_ack: true,
            retried: 0,
        });
    }

    fn send_connect_request(&mut self){
        let tunnel_request: Service<()> = Service::ConnectRequest(crate::knxnet::connect::ConnectRequest{
            data: self.host_info,
            control: self.host_info,
            connection_type: cri::ConnectionReqType::TunnelConnection {layer: cri::TunnelingLayer::TunnelLinkLayer}
        });


        let buf = tunnel_request.encoded();
        self.out_queue.clear();
        self.inbound_seq = 0;
        self.outbound_seq = 0;
        self.state = TunnelConnectionState::Connecting;
        self.push_out_message(OutMessage{
            data: buf,
            need_ack: true,
            retried: 0,
        });
    }

    fn send_disconnect_request(&mut self){
        let disconnect_request: Service<()> = Service::DisconnectRequest(crate::knxnet::disconnect::DisconnectRequest{
            channel: self.channel,
            control: self.host_info,
        });


        let buf = disconnect_request.encoded();
        self.state = TunnelConnectionState::Disconnecting;
        self.push_out_message(OutMessage{
            data: buf,
            need_ack: true,
            retried: 0,
        });
    }
}