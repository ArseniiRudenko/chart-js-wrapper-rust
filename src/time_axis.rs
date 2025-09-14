use serde::{Serialize, Serializer};
use serde::ser::Error;
use crate::{impl_scale_type, ScaleType};
use crate::serde::{SerializeFormat, WithTypeAndSerializer};
use time::{OffsetDateTime, UtcDateTime};
use time::format_description::well_known::Rfc3339;
use time::macros::format_description;

impl_scale_type!(Category for time::Month time::Weekday time::Time);
impl_scale_type!(Time for OffsetDateTime time::PrimitiveDateTime UtcDateTime time::Date);



impl<T: serde::Serialize + ?Sized + ToString> SerializeFormat<T> for time::Weekday{
    fn serialize<S>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer
    {
        value.to_string().serialize(serializer)
    }
}

impl<T: serde::Serialize + ?Sized + ToString> SerializeFormat<T> for time::Month{
    fn serialize<S>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer
    {
        value.to_string().serialize(serializer)
    }
}



impl SerializeFormat<OffsetDateTime> for OffsetDateTime{
    fn serialize<S>(value: &OffsetDateTime, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer
    {
        let str = value.format(&Rfc3339).map_err(|err| Error::custom(err.to_string()))?;
        //serialize to rfc string
        serializer.serialize_str(str.as_str())
    }
}


impl SerializeFormat<time::PrimitiveDateTime> for time::PrimitiveDateTime{
    fn serialize<S>(value: &time::PrimitiveDateTime, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer
    {

        let str = value.format(&Rfc3339).map_err(|err| Error::custom(err.to_string()))?;
        //serialize to rfc string
        serializer.serialize_str(str.as_str())
    }
}


impl SerializeFormat<UtcDateTime> for UtcDateTime{
    fn serialize<S>(value: &UtcDateTime, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer
    {

        let str = value.format(&Rfc3339).map_err(|err| Error::custom(err.to_string()))?;
        //serialize to rfc string
        serializer.serialize_str(str.as_str())
    }
}


impl SerializeFormat<time::Date> for time::Date{
    fn serialize<S>(value: &time::Date, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer
    {
        let str = value.format(&Rfc3339).map_err(|err| Error::custom(err.to_string()))?;
        //serialize to rfc string
        serializer.serialize_str(str.as_str())
    }
}


impl SerializeFormat<time::Time> for time::Time{
    fn serialize<S>(value: &time::Time, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer
    {
        let str = value.format(&Rfc3339).map_err(|err| Error::custom(err.to_string()))?;
        //serialize to rfc string
        serializer.serialize_str(str.as_str())
    }
}