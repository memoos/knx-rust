use crate::cemi::dpt::DPT;

pub enum GroupEventType{
    GroupValueRead          = 0b0000_000000,
    GroupValueResponse      = 0b0001_000000,
    GroupValueWrite         = 0b0010_000000,
}

pub struct GroupEvent<D: DPT> {
    pub(crate) address: u16,
    pub(crate) event_type: GroupEventType,
    pub(crate) data: D,
}

