use crate::{impl_scale_type, ScaleType, WithScaleType};
use chrono::{DateTime, TimeZone, Weekday};

impl_scale_type!(Category for  Weekday chrono::Month);
impl_scale_type!(Time for chrono::NaiveDateTime);

impl<T:TimeZone> WithScaleType for DateTime<T> {
    fn scale_type() -> ScaleType {
        ScaleType::Time
    }

}