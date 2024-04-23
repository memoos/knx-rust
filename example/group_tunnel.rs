use std::{io, time};
use std::net::IpAddr;
use std::time::Duration;
use knx_rust;
use knx_rust::address::GroupAddress2;
use knx_rust::group_event::GroupEvent;
use knx_rust::group_event::GroupEventType::{GroupValueRead};

use mio::net::{UdpSocket};

const UDP_SOCKET: Token = Token(0);
const LIB_TIMER: Token = Token(1);
const SEND_TIMER: Token = Token(2);

use mio::{Events, Interest, Poll, Token};
use mio_timerfd::{ClockId, TimerFd};

use knx_rust::tunnel_connection::{TunnelConnection, TunnelConnectionConfig};


// ------------------------------------------------------------------------------
fn main() -> io::Result<()> {

    let mut poll = Poll::new()?;

    let mut events = Events::with_capacity(3); // udp and 2 kinds of timer

    let knx_addr: core::net::SocketAddr = "192.168.1.10:3671".parse().unwrap();
    let mut socket = UdpSocket::bind("0.0.0.0:0".parse().unwrap()).expect("couldn't bind to address");
    socket
        .connect(knx_addr)
        .expect("couldn't connect to address");

    let addr = socket.local_addr().unwrap();
    println!("Local Addr {:?}", addr);
    let ipv4 = match addr.ip() {
        IpAddr::V4(ip) => Ok(ip.octets()),
        _ => Err("Invalid IPv4 Address"),
    }.unwrap();


    let mut lib_timer = TimerFd::new(ClockId::Monotonic).unwrap();
    let mut send_timer = TimerFd::new(ClockId::Monotonic).unwrap();
    poll.registry()
        .register(&mut socket, UDP_SOCKET,  Interest::READABLE.add(Interest::WRITABLE))?;

    poll.registry()
        .register(&mut lib_timer, LIB_TIMER, Interest::READABLE)?;

    // send out a message every 30s
    send_timer.set_timeout_interval(&Duration::from_secs(30));
    poll.registry()
        .register(&mut send_timer, SEND_TIMER, Interest::READABLE)?;

    let mut knx_tunnel = TunnelConnection::new(ipv4, addr.port(), TunnelConnectionConfig::default());

    let mut buf = [0; 1 << 16];

    loop {
        //send out all pending data
        while match knx_tunnel.get_outbound_data() {
            Some(data) => match socket.send(data) {
                Ok(count) => {count},
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => {0},
                Err(e) => {
                    return Err(e);
                }
            },
            None => 0
        } > 0 {}

        // set next timer event
        let duration = knx_tunnel.get_next_time_event().duration_since(time::Instant::now());
        lib_timer.set_timeout(&duration);

        // Poll to give computing time to operating system until we have new events
        if let Err(err) = poll.poll(&mut events, None) {
            if err.kind() == io::ErrorKind::Interrupted {
                continue;
            }
            return Err(err);
        }

        //handle events
        for event in events.iter() {
            match event.token() {
                // handle all incoming data
                UDP_SOCKET => loop {
                    match socket.recv_from(&mut buf) {
                        Ok((packet_size, source_address)) => {
                            match knx_tunnel.handle_inbound_message(&buf[..packet_size]) {
                                Some(event) => {
                                    let addr = GroupAddress2::from_u16(event.address);
                                    println!("Received {:?} on {} with value {:02X?}", event.event_type, addr, event.data);
                                }
                                None => {}
                            }
                        }
                        Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                            break;
                        }
                        Err(e) => {
                            return Err(e);
                        }
                    }
                },
                //handle library time events
                LIB_TIMER => {
                    match lib_timer.read() {
                        Ok(nr) => knx_tunnel.handle_time_events(),
                        Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                            //ignore
                        }
                        Err(e) => {
                            return Err(e);
                        }
                    }
                }
                //handle send timer
                SEND_TIMER => {
                    if knx_tunnel.connected() {
                        println!("Sending group event");
                        //request data from a group address
                        knx_tunnel.send(GroupEvent{
                            event_type: GroupValueRead,
                            address: GroupAddress2::new(8, 4).to_u16(),
                            data: vec![],
                        });
                    }
                }
                _=>{}
            }
        }
    }
}
