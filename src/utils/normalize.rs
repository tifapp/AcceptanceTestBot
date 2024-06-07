/// A trait for normalizing strings to make comparison against natural language
/// easier.
pub trait RoswaalNormalize {
    fn roswaal_normalize(&self) -> String;
}

impl RoswaalNormalize for &str {
    fn roswaal_normalize(&self) -> String {
        self.to_lowercase().split_whitespace().collect()
    }
}
