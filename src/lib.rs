mod options;
mod colour;

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }

    #[test]
    fn colour_test(){
        let color = crate::colour::Rgb(255, 0, 0);
        let json = serde_json::to_string(&color).unwrap();
        println!("Serialized: {}", json); // "rgb(255, 0, 0)"

        let deserialized: crate::colour::Rgb = serde_json::from_str(&json).unwrap();
        println!("Deserialized: {:?}", deserialized); // Rgb(255, 0, 0)
    }
}
