use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde::de::{self, Visitor};
use std::fmt;

#[derive(Debug,Clone, PartialEq)]
pub struct Rgb(pub u8,pub  u8,pub  u8);

impl Serialize for Rgb {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = format!("rgb({}, {}, {})", self.0, self.1, self.2);
        serializer.serialize_str(&s)
    }
}

impl<'de> Deserialize<'de> for Rgb {
    fn deserialize<D>(deserializer: D) -> Result<Rgb, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct RgbVisitor;

        impl<'de> Visitor<'de> for RgbVisitor {
            type Value = Rgb;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str(r#"a string in the format "rgb(r, g, b)""#)
            }

            fn visit_str<E>(self, v: &str) -> Result<Rgb, E>
            where
                E: de::Error,
            {
                let v = v.trim();
                if !v.starts_with("rgb(") || !v.ends_with(')') {
                    return Err(E::custom("invalid format"));
                }

                let content = &v[4..v.len() - 1]; // strip "rgb(" and ")"
                let parts: Vec<&str> = content.split(',').map(str::trim).collect();
                if parts.len() != 3 {
                    return Err(E::custom("expected three components"));
                }

                let r = parts[0].parse::<u8>().map_err(E::custom)?;
                let g = parts[1].parse::<u8>().map_err(E::custom)?;
                let b = parts[2].parse::<u8>().map_err(E::custom)?;

                Ok(Rgb(r, g, b))
            }
        }

        deserializer.deserialize_str(RgbVisitor)
    }
}

