use crate::knxnet::KnxNetIpError;

pub trait DPT{
    fn encode(&self, buf: &mut Vec<u8>);
    fn decode(&mut self, buf: &[u8]) -> Result<(), KnxNetIpError> where Self: Sized;
    fn bit_len(&self) -> u16;
}

impl DPT for Vec<u8> {
    fn encode(&self, buf: &mut Vec<u8>) {
        buf.extend(self)
    }

    fn decode(&mut self,buf: &[u8]) -> Result<(), KnxNetIpError> {
        self.clear();
        if buf.len() > 1 {
            self.extend_from_slice(&buf[1..]);
        } else if buf.len() == 1{
            self.push(buf[0] & 0x3F)
        }
        Ok(())
    }

    fn bit_len(&self) -> u16 {
        8 * self.len() as u16
    }
}

impl DPT for bool {
    fn encode(&self, buf: &mut Vec<u8>) {
        buf.push(*self as u8)
    }

    fn decode(&mut self,buf: &[u8]) -> Result<(), KnxNetIpError> {
        *self = buf[0] & 0x1 > 0;
        return Ok(())
    }

    fn bit_len(&self) -> u16 {
        1
    }
}

impl DPT for () {
    fn encode(&self, buf: &mut Vec<u8>) {
    }

    fn decode(&mut self,buf: &[u8]) -> Result<(), KnxNetIpError> where Self: Sized {
        Ok(())
    }

    fn bit_len(&self) -> u16 {
        return 0
    }
}