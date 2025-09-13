use crate::{impl_scale_type, ScaleType, WithScaleType};
use time::{OffsetDateTime, UtcDateTime};


impl_scale_type!(Category for  time::Month time::Weekday time::Time);
impl_scale_type!(Time for OffsetDateTime time::PrimitiveDateTime UtcDateTime time::Date);

