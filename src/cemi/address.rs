use std::fmt::{Display, Formatter};
use std::ops::Sub;
use strum_macros::FromRepr;



/*
type IndividualAddress = u16;

impl IndividualAddress {
    pub fn area(self) -> u8 {
        return self >> 12;
    }
    pub fn line(self) -> u8  {
        return (self >> 8) & 0xf
    }
    pub fn device(self) -> u8 {
        return self & 0xff;
    }
}

impl Display for IndividualAddress {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.area(), self.line(), self.device())
    }
}



type GroupAddress = u16;
type GroupAddress2 = u16;


impl GroupAddress2 {
    pub fn main(self) -> u16 {
        return self >> 11;
    }
    pub fn sub(self) -> u16 {
        return self & 0x7ff;
    }
}

impl Display for GroupAddress2 {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}", self.main(), self.sub())
    }
}

type GroupAddress3 = u16;

impl GroupAddress3 {
    pub fn main(self) -> u16 {
        return self >> 11;
    }
    pub fn middle(self) -> u16 {
        return (self >> 8) & 0x7;
    }
    pub fn sub(self) -> u16 {
        return self & 0xff;
    }
}

impl Display for GroupAddress3 {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}/{}", self.main(), self.middle(), self.sub())
    }
}
*/