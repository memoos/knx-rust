use std::fmt::{Display, Formatter};
use std::ops::Sub;


pub struct IndividualAddress {
    addr: u16
}

impl IndividualAddress {
    pub fn new(area: u8, line: u8,device: u8) -> IndividualAddress{
        return IndividualAddress{addr: ((area as u16) << 12) | (((line & 0xf) as u16) << 8) | (device as u16)}
    }
    pub fn from_u16(addr:u16) -> IndividualAddress{
        return IndividualAddress{addr}
    }
    pub fn to_u16(&self) -> u16 {
        self.addr
    }
    pub fn area(&self) -> u8 {
        return (self.addr >> 12) as u8;
    }
    pub fn line(&self) -> u8  {
        return ((self.addr >> 8) & 0xf) as u8;
    }
    pub fn device(&self) -> u8 {
        return (self.addr & 0xff) as u8;
    }
}

impl Display for IndividualAddress {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.area(), self.line(), self.device())
    }
}



pub struct GroupAddress2 {
    addr: u16
}

impl GroupAddress2 {
    pub fn new(main: u8, sub: u16) -> GroupAddress2{
        return GroupAddress2{addr: ((main as u16) << 11) | (sub & 0x7FF)}
    }
    pub fn from_u16(addr:u16) -> GroupAddress2{
        return GroupAddress2{addr}
    }
    pub fn to_u16(&self) -> u16 {
        self.addr
    }
    pub fn main(&self) -> u8 {
        return (self.addr >> 11) as u8;
    }
    pub fn sub(&self) -> u16 {
        return self.addr & 0x7ff;
    }
}

impl Display for GroupAddress2 {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}", self.main(), self.sub())
    }
}

pub struct GroupAddress3 {
    addr: u16
}
impl GroupAddress3 {
    pub fn new(main: u8, middle: u8, sub: u8) -> GroupAddress3{
        return GroupAddress3{addr: ((main as u16) << 11) | ((middle as u16) << 8) | (sub as u16)}
    }
    pub fn from_u16(addr:u16) -> GroupAddress3{
        return GroupAddress3{addr}
    }
    pub fn to_u16(&self) -> u16 {
        self.addr
    }
    pub fn main(&self) -> u8 {
        return (self.addr >> 11) as u8;
    }
    pub fn middle(&self) -> u8 {
        return ((self.addr >> 8) & 0x7) as u8;
    }
    pub fn sub(&self) -> u8 {
        return (self.addr & 0xff) as u8;
    }
}

impl Display for GroupAddress3 {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}/{}", self.main(), self.middle(), self.sub())
    }
}


#[cfg(test)]
mod tests {
    use crate::address::{GroupAddress2, GroupAddress3, IndividualAddress};

    #[test]
    fn individual_address() {
        let address = IndividualAddress::new(1,2,3);

        assert_eq!(address.area(), 1);
        assert_eq!(address.line(), 2);
        assert_eq!(address.device(), 3);
        assert_eq!("1.2.3", format!("{}", address));

        let from_u16 = IndividualAddress::from_u16(0x1234);
        assert_eq!(from_u16.to_u16(), 0x1234)
    }

    #[test]
    fn group_address3() {
        let address = GroupAddress3::new(1,2,3);

        assert_eq!(address.main(), 1);
        assert_eq!(address.middle(), 2);
        assert_eq!(address.sub(), 3);
        assert_eq!("1/2/3", format!("{}", address));

        let from_u16 = GroupAddress3::from_u16(0x1234);
        assert_eq!(from_u16.to_u16(), 0x1234)
    }


    #[test]
    fn group_address2() {
        let address = GroupAddress2::new(1,3);

        assert_eq!(address.main(), 1);
        assert_eq!(address.sub(), 3);
        assert_eq!("1/3", format!("{}", address));

        let from_u16 = GroupAddress2::from_u16(0x1234);
        assert_eq!(from_u16.to_u16(), 0x1234)
    }
}
