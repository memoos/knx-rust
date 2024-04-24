use std::fmt::{Display, Formatter};
use crate::dpt::DPT;
use crate::knxnet::KnxNetIpError;
// Datapoint types "2-Octed Float Value" (See 3/7/2 3.10)

#[derive(Debug, Copy, Clone, PartialEq, Default)]
pub struct f16(u16);

impl f16{
    fn from_f32(f: f32) -> f16{
        let x = (f*100f32).to_bits();
        let sign = x & 0x8000_0000u32;
        let mut exp = ((x & 0x7F80_0000u32) >> 23) as i32 - 127 - 10; // 127 from IEEE standard and -10 because comma sits at the end in KNX f16
        let mut man = ((x & 0x007F_FFFFu32) | 0x0080_0000u32) >> 13 ; // only 11 of 24 bits fit into our f16 type

        //special handling for 0
        if exp == -127 {
            return f16(0)
        }

        while exp < 0 {
            man >>= 1;
            exp += 1
        }

        while exp > 15 {
            man <<= 1;
            exp -= 1
        }

        if sign > 0 {
            man = !man + 1
        }

        f16(((man & 0x800) << 4) as u16 | ((exp as u16)  << 11) | (man & 0x7FF)  as u16)
    }

    fn as_f32(&self) -> f32{
        let man =  self.0 & 0x7FF;
        let exp = (self.0 >> 11) & 0xf;

        if (self.0 & 0x8000) > 0 {
            -0.01f32 * (((!man + 1)&0x7FF)<<exp) as f32
        } else {
            0.01f32 * (man<<exp) as f32
        }
    }
}

impl DPT for f16 {
    fn encode(&self, buf: &mut Vec<u8>) {
        self.0.encode(buf);
    }

    fn decode(&mut self, buf: &[u8]) -> Result<(), KnxNetIpError> where Self: Sized {
        self.0.decode(buf)
    }

    fn bit_len(&self) -> u16 {
        self.0.bit_len()
    }
}

macro_rules! impl_f16_type {
    ($name: ident, $format: literal, $min: literal, $max: literal) => {
        #[derive(Debug, Copy, Clone, PartialEq, Default)]
        pub struct $name(f16);

        impl $name {
            pub fn from_bytes(buf: &[u8]) -> Result<$name, KnxNetIpError> {
                let mut res = $name::default();
                res.decode(buf)?;
                Ok(res)
            }
            pub fn from_float32(v: f32) -> $name{
                return $name(f16::from_f32(v))
            }
            pub fn as_float32(&self) -> f32{
                return self.0.as_f32()
            }
        }

        impl DPT for $name {
            fn encode(&self, buf: &mut Vec<u8>) {
                self.0.encode(buf);
            }

            fn decode(&mut self, buf: &[u8]) -> Result<(), KnxNetIpError> where Self: Sized {
                self.0.decode(buf)
            }

            fn bit_len(&self) -> u16 {
                self.0.bit_len()
            }
        }

        impl Display for $name {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                write!(f, $format, self.0.as_f32())
            }
        }
    }
}

//9.001
impl_f16_type!(DptValueTemp, "{:.2} °C", -273.0, 670_760.0);
//9.002
impl_f16_type!(DptValueTempd, "{:.2} K", -670_760.0, 670_760.0);
//9.003
impl_f16_type!(DptValueTempa, "{:.2} K/h", -670_760.0, 670_760.0);
//9.004
impl_f16_type!(DptValueLux, "{:.2} Lux", 0, 670_760.0);
//9.005
impl_f16_type!(DptValueWsp, "{:.2} m/s", 0, 670_760.0);
//9.006
impl_f16_type!(DptValuePres, "{:.2} Pa", 0, 670_760.0);
//9.007
impl_f16_type!(DptValueHumidity, "{:.2} %", 0, 670_760.0);
//9.008
impl_f16_type!(DptValueAirQuality, "{:.2} ppm", 0, 670_760.0);
//9.010
impl_f16_type!(DptValueTime1, "{:.2} s", -670_760.0, 670_760.0);
//9.011
impl_f16_type!(DptValueTime2, "{:.2} ms", -670_760.0, 670_760.0);
//9.020
impl_f16_type!(DptValueVolt, "{:.2} mV", -670_760.0, 670_760.0);
//9.021
impl_f16_type!(DptValueCurr, "{:.2} mA", -670_760.0, 670_760.0);
//9.022
impl_f16_type!(DptPowerDensity, "{:.2} W/m²", -670_760.0, 670_760.0);
//9.023
impl_f16_type!(DptKelvinPerPercent, "{:.2} K/%", -670_760.0, 670_760.0);
//9.024
impl_f16_type!(DptPower, "{:.2} kW", -670_760.0, 670_760.0);
//9.025
impl_f16_type!(DptValueVolumeFlow, "{:.2} l/h", -670_760.0, 670_760.0);
//9.026
impl_f16_type!(DptRainAmount, "{:.2} l/m²", -671_088.64, 670_760.96);
//9.027
impl_f16_type!(DptValueTempF, "{:.2} °F", -459.6, 670_760.96);
//9.028
impl_f16_type!(DptValueWspKmh, "{:.2} km/h", 0, 670_760.96);


#[cfg(test)]
mod tests {
    use crate::dpt::float_16::f16;

    #[test]
    fn f16_enc_tests() {
        assert_eq!(f16::from_f32(0f32), f16(0));
        assert_eq!(f16::from_f32(1.0f32), f16(100));
        assert_eq!(f16::from_f32(0.01f32), f16(1));
        assert_eq!(f16::from_f32(-1f32), f16(0x879c));
        assert_eq!(f16::from_f32(20.48f32), f16(0x0C00));
    }

    #[test]
    fn f16_dec_test() {
        assert_eq!(f16(0).as_f32(), 0f32);
        assert_eq!(f16(100).as_f32(), 1f32);
        assert_eq!(f16(1).as_f32(), 0.01f32);
        assert_eq!(f16(0x879c).as_f32(), -1f32);
        assert_eq!(f16(0xc00).as_f32(), 20.48f32);
    }
}
