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

An example how to interact with the library using mio (https://docs.rs/mio/latest/mio/) can be found in the example folder.
