mod options;

pub mod render;
pub mod common;

pub use options::*;

#[cfg(test)]
mod tests {
    use super::*;
    

    #[test]
    fn colour_test(){
        let color = common::Rgb(255, 0, 0);
        let json = serde_json::to_string(&color).unwrap();
        assert_eq!(json, "\"rgb(255, 0, 0)\"");
        println!("Serialized: {}", json);
        let deserialized: common::Rgb = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, color);
        println!("Deserialized: {:?}", deserialized);
    }
}
