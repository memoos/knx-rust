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
use crate::knxnet::connectionstate::ConnectionstateRequest;
use crate::knxnet::{cri, Service};
use crate::knxnet::hpai::{HPAI, Protocol};

pub struct TunnelConnectionConfig {
    resent_interval: Duration,
    connect_response_timeout: Duration,
    heartbeat_response_timeout: Duration,
    heartbeat_interval: Duration,
}

impl TunnelConnectionConfig {
    fn default() -> TunnelConnectionConfig {
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
    next_seq: u8,
    out_queue: VecDeque<Vec<u8>>,
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
        let tunnel_request = Service::ConnectRequest(crate::knxnet::connect::ConnectRequest{
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
            next_seq: 0,
            out_queue: VecDeque::from(vec![buf]),
            host_info,
            message_pending: true,
        };

    }
/*
    pub async fn send(&mut self, msg: crate::cemi) ->() {
        let req = crate::tunnel::TunnelReq {
            payload: msg,
            channel: self.channel,
            seq_number: self.seq,
        };
    }
*/
    pub fn get_outbound_data(&mut self) -> Option<&[u8]> {
        if self.message_pending && !self.out_queue.is_empty() {
            self.message_pending = false;
            self.next_resent = Instant::now().add(self.config.resent_interval);
            Some(&self.out_queue[0])
        } else {
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
        if self.next_timeout < Instant::now() {
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
        if self.next_resent < Instant::now() && !self.out_queue.is_empty() {
            self.message_pending = true
        }
    }

    fn send_connection_state_request(&mut self){
        let req = Service::ConnectionstateRequest(ConnectionstateRequest{
            channel: self.channel,
            control: self.host_info
        });

        self.out_queue.push_back(req.encoded())
    }
}