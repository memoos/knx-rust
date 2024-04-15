use strum_macros::FromRepr;

#[derive(FromRepr, Debug, Copy, Clone, PartialEq, Default)]
#[repr(u8)]
pub enum Message {
    //TODO fill
    #[default]
    LData = 0x11,
}