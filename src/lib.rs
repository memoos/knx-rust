#![crate_name = "knx_rust"]
#![crate_type = "lib"]

//#![no_std]

#[macro_use]
extern crate nom;

#[macro_use]
use num_traits;

mod tunnel_connection;
mod knxnet;
