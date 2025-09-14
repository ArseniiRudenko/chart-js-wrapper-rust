use crate::{impl_scale_type, ScaleType};
use chrono::{DateTime, TimeZone, Weekday};
use serde::{Serialize, Serializer};
use crate::serde::{SerializeFormat, WithTypeAndSerializer};

impl_scale_type!(Category for chrono::Weekday chrono::Month);
impl_scale_type!(Time for chrono::NaiveDateTime);

impl<T:TimeZone> WithTypeAndSerializer for DateTime<T> {
    fn scale_type() -> ScaleType {
        ScaleType::Time
    }

    type Serialization = DateTime<T>;
}

impl<T:TimeZone> SerializeFormat<DateTime<T>> for DateTime<T>{
    fn serialize<S>(value: &DateTime<T>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer
    {
        value.to_rfc3339().serialize(serializer)
    }
}


impl SerializeFormat<chrono::NaiveDateTime> for chrono::NaiveDateTime{
    fn serialize<S>(value: &chrono::NaiveDateTime, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer
    {
        value.and_utc().to_rfc3339().serialize(serializer)
    }
}

impl SerializeFormat<chrono::Month> for chrono::Month{
    fn serialize<S>(value: &chrono::Month, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer
    {
        value.name().serialize(serializer)
    }
}

impl SerializeFormat<Weekday> for Weekday{
    fn serialize<S>(value: &Weekday, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer
    {
        value.to_string().serialize(serializer)
    }
}