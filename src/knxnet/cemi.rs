use strum_macros::FromRepr;

#[derive(FromRepr, Debug, Copy, Clone, PartialEq)]
#[repr(u8)]
pub enum Message {
    DeviceMgmtConnection = 0x03,
    TunnelConnection{
        address: u16 // TODO use individual address type
    } = 0x04,
    RemlogConnection = 0x06,
    RemconfConnection = 0x07,
    ObjsvrConnection = 0x08,
}