use std::io;
use std::fmt::Display;
use std::net::IpAddr;
use std::sync::Arc;
use std::time::Duration;
use knx_rust;
use knx_rust::address::GroupAddress2;
use knx_rust::group_event::GroupEvent;
use knx_rust::group_event::GroupEventType::{GroupValueRead, GroupValueWrite};
use knx_rust::dpt::{DptValueHumidity, DptValueTemp};
use knx_rust::tunnel_connection::{TunnelConnection, TunnelConnectionConfig};

use tokio::select;
use tokio::time::Instant;
use tokio::net::{UdpSocket};
use tokio::sync::Mutex;


// ------------------------------------------------------------------------------
#[tokio::main(flavor = "current_thread")]
async fn main() -> io::Result<()> {

    let knx_addr: core::net::SocketAddr = "192.168.178.60:3671".parse().unwrap();
    let mut socket = UdpSocket::bind("0.0.0.0:0").await.expect("couldn't bind to address");
    socket
        .connect(knx_addr)
        .await.expect("couldn't connect to address");

    let addr = socket.local_addr().unwrap();
    println!("Local Addr {:?}", addr);
    let ipv4 = match addr.ip() {
        IpAddr::V4(ip) => Ok(ip.octets()),
        _ => Err("Invalid IPv4 Address"),
    }.unwrap();

    let mut knx_tunnel = Arc::new(Mutex::new(TunnelConnection::new(ipv4, addr.port(), TunnelConnectionConfig::default())));
    let mut buf = [0; 1 << 16];

    // spawn a task to send out messages
    let mut send_interval = tokio::time::interval(Duration::from_secs(60));
    let knx_send_tunnel = knx_tunnel.clone();
    tokio::spawn(async move {
        loop {
            send_interval.tick().await;
            let mut knx_lock = knx_send_tunnel.lock().await;
            if knx_lock.connected() {
                println!("Sending group events");
                //request data from a group address 8/4
                knx_lock.send(GroupEvent{
                    event_type: GroupValueRead,
                    address: GroupAddress2::new(8, 4).to_u16(),
                    data: vec![],
                });

                //send humidity 12.34% on group address 1/3
                knx_lock.send(GroupEvent{
                    event_type: GroupValueWrite,
                    address: GroupAddress2::new(1, 3).to_u16(),
                    data: DptValueHumidity::from_float32(12.34),
                });
            }

        }
    });

    //main loop handles interaction with library
    loop {
        //send out all pending data
        let timeout = {
            let mut knx_lock = knx_tunnel.lock().await;
            while let Some(data) = knx_lock.get_outbound_data() {
                socket.send(data).await?;
            }
            tokio::time::sleep_until(Instant::from(knx_lock.get_next_time_event()))
        };


        //handle inbound data
        select! {
            v = socket.recv_from(&mut buf) => {
                let mut knx_lock = knx_tunnel.lock().await;
                match v {
                    Ok((packet_size, _)) => {
                        match knx_lock.handle_inbound_message(&buf[..packet_size]) {
                            Some(event)  if event.event_type != GroupValueRead=> {
                                let addr = GroupAddress2::from_u16(event.address);
                                println!("Received {:?} on {} with value {:02X?}", event.event_type, addr, event.data);
                                // let's say we expect a temperature value at group 1/2
                                if addr.main() == 1 && addr.sub() == 2 {
                                    println!("Received a temperature of {}", DptValueTemp::from_bytes(event.data.as_slice()).unwrap());
                                }
                            }
                            _ => {}
                        }
                    }
                    Err(e) => {
                        eprintln!("Error while reading knx data {}", e);
                    }
                }
            }
            _ = timeout => {
                let mut knx_lock = knx_tunnel.lock().await;
                knx_lock.handle_time_events();
            }
        }
    }
}
