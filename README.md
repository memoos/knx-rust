# knx_rust

Knx rust is a library implementing the KNXNet/Ip protocol, to enable communication with KNX-Devices through
KNX Ip Gateways or Routers.
The library is implemented without any io calls (https://sans-io.readthedocs.io/how-to-sans-io.html) to have minimal runtime requirements so that it could be used with tokio, async-std or 
mio only in embedded environments.

## Usage

Add `knx_rust` as a dependency in `Cargo.toml`:

```toml
[dependencies]
knx_rust = "0.0.1"
```

You need to make sure that the following functions called regularly (for exmaple in your event loop):
- `knx.get_outbound_data()` and send the retuned data out to UDP if there is some. There might be new data after initialisation or after a call to `knx.send`, `knx.handle_inbound_message` or `knx.handle_time_events`.
- `knx.handle_inbound_message(buf)` in case data is received from UDP. In case the data contains a group communication message is is returned by the function and could handled further
- `knx.handle_time_events()` needs to be called next at point in time defined by `knx.get_next_time_event()`. The time needs to be updated after data has been send out to UDP (after `knx.get_outbound_data()`).

Apart from that messages can be send to the bus at any time using `knx.send(group_event)`. 

An example how to interact with the library using [mio](https://docs.rs/mio/latest/mio/) or [tokio](https://tokio.rs/) can be found in the [examples](./examples/) folder.

These examples can be executed using
```
cargo run --example group_tunnel_mio
```
or
```
cargo run --example group_tunnel_tokio
```
