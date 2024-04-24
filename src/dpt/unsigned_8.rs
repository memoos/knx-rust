use std::fmt::{Display, Formatter};
use crate::dpt::DPT;
use crate::knxnet::KnxNetIpError;

// Datapoint types "8 bit unsigned value" (See 3/7/2 3.5)

macro_rules! impl_scaled_u16_type {
    ($name: ident, $format: literal, $min: literal, $max: literal) => {
        #[derive(Debug, Copy, Clone, PartialEq, Default)]
        pub struct $name(u8);
        impl DPT for $name{
            fn bit_len(&self) -> u16 {
                8
            }
            fn decode(&mut self, buf: &[u8]) -> Result<(), KnxNetIpError> where Self: Sized {
                if buf.len() < 1 {
                    return Err(KnxNetIpError::MessageTooShort(buf.len()))
                }
                Ok(self.0 = buf[0])
            }
            fn encode(&self, buf: &mut Vec<u8>) {
                buf.push(self.0)
            }
        }

        impl Display for $name {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                write!(f, $format, self.as_float32())
            }
        }

        impl $name{
            pub fn from_bytes(buf: &[u8]) -> Result<$name, KnxNetIpError> {
                let mut res = $name::default();
                res.decode(buf)?;
                Ok(res)
            }
            pub fn from_float32(v: f32) -> $name{
                return $name((((v.max($min).min($max)) - $min) * 255.0 / ($max - $min)) as u8)
            }
            pub fn as_float32(&self) -> f32{
                return self.0 as f32 * ($max - $min)/255.0 + $min
            }
        }
    }
}

//5.001
impl_scaled_u16_type!(DptScaling, "{:.1} %", 0.0, 100.0);
//5.003
impl_scaled_u16_type!(DptAngle, "{:.1} °", 0.0, 3600.0);
//5.004
impl_scaled_u16_type!(DptPercentU8, "{:.0} %", 0.0, 255.0);
//5.005
impl_scaled_u16_type!(DptDecimalFactor, "{:.0}", 0.0, 255.0);

