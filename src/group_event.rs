use strum_macros::FromRepr;
use crate::dpt::DPT;

#[derive(FromRepr, Debug, Copy, Clone, PartialEq)]
#[repr(u16)]
pub enum GroupEventType{
    GroupValueRead          = 0b0000_000000,
    GroupValueResponse      = 0b0001_000000,
    GroupValueWrite         = 0b0010_000000,
}

pub struct GroupEvent<D: DPT> {
    pub address: u16,
    pub event_type: GroupEventType,
    pub data: D,
}

