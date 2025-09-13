use std::time::{Instant, SystemTime};
use serde::{Serialize, Serializer};
use crate::ScaleType;

#[derive(Debug,Clone)]
pub struct ValueSerializeWrapper<T: WithTypeAndSerializer>(pub(crate) T);

impl <T: WithTypeAndSerializer> From<T> for ValueSerializeWrapper<T> {
    fn from(value: T) -> Self {
        ValueSerializeWrapper(value)
    }
}

impl<T: WithTypeAndSerializer> Serialize for ValueSerializeWrapper<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer
    {
        T::Serialization::serialize(&self.0, serializer)
    }
}

pub trait WithTypeAndSerializer {
    fn scale_type()->ScaleType;

    type Serialization: SerializeFormat<Self>
    where
        Self: Sized;
}




pub struct DefaultSerialisation;

pub trait SerializeFormat<T: ?Sized> {
    fn serialize<S>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer;
}

impl<T: serde::Serialize + ?Sized> SerializeFormat<T> for DefaultSerialisation {
    fn serialize<S>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        value.serialize(serializer)
    }
}


#[macro_export]
macro_rules! impl_scale_type_default {
    ($type:ident for $($t:ty)*) => ($(
        impl WithTypeAndSerializer for $t {
            fn scale_type() -> ScaleType {
                ScaleType::$type
            }

            type Serialization = DefaultSerialisation;
        }
    )*)
}


#[macro_export]
macro_rules! impl_scale_type {
    ($type:ident for $($t:ty)*) => ($(
        impl WithTypeAndSerializer for $t {
            fn scale_type() -> ScaleType {
                ScaleType::$type
            }
            type Serialization = $t;
        }
    )*)
}





impl_scale_type_default!(Linear for u8 u16 u32 u64 u128 usize i8 i16 i32 i64 i128 isize f32 f64);
impl_scale_type_default!(Category for &str String);
impl_scale_type!(Time for SystemTime Instant);


impl SerializeFormat<SystemTime> for SystemTime{
    fn serialize<S>(value: &SystemTime, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer
    {
        let duration = value.duration_since(SystemTime::UNIX_EPOCH).unwrap();
        let seconds = duration.as_secs();
        let nanos = duration.subsec_nanos();
        let seconds_f64 = seconds as f64 + nanos as f64 / 1_000_000_000.0;
        serializer.serialize_f64(seconds_f64)
    }
}

impl SerializeFormat<Instant> for Instant{
    fn serialize<S>(value: &Instant, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer
    {
        let duration = value.elapsed();
        let seconds = duration.as_secs();
        let nanos = duration.subsec_nanos();
        let seconds_f64 = seconds as f64 + nanos as f64 / 1_000_000_000.0;
        serializer.serialize_f64(seconds_f64)
    }
}