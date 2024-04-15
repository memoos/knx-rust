use strum_macros::FromRepr;
use thiserror::Error;

#[derive(Error, FromRepr, Debug, Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd, Default)]
#[repr(u8)]
pub enum StatusCode {
    // NoError indicates a successful operation.
    #[error("no error")]
    #[default]
    NoError = 0x00,

    #[error("unsupported host protocol")]
    ErrHostProtocolType = 0x01,

    #[error("nsupported KNXnet/IP protocol version")]
    ErrVersionNotSupported = 0x02,

    #[error("out-of-order sequence number has been received")]
    ErrSequenceNumber = 0x04,

    #[error("no active data connection with given ID")]
    ErrConnectionID = 0x21,

    #[error("unsupported connection type")]
    ErrConnectionType = 0x22,

    #[error("unsupported connection option")]
    ErrConnectionOption = 0x23,

    #[error("Server cannot accept more connections")]
    ErrNoMoreConnections = 0x24,

    #[error("Tunnelling server has no free Individual address available that could be used by the connection")]
    ErrNoMoreUniqueConnections = 0x25,

    #[error("error with a data connection")]
    ErrDataConnection = 0x26,

    #[error("error with a KNX connection")]
    ErrKNXConnection = 0x27,

    #[error("unsupported tunnelling layer")]
    ErrTunnellingLayer = 0x29,
}