use base64::prelude::*;
use std::{fs::File, io::BufReader};

use serde::{
    Deserialize, Deserializer,
    de::{self, DeserializeOwned},
};

pub struct Secrets;

#[derive(Debug, Deserialize)]
struct YamlSecrets<T> {
    data: T,
}

pub fn b64_to_string<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let encoded = String::deserialize(deserializer)?;
    let bytes = BASE64_STANDARD
        .decode(&encoded)
        .map_err(|e| de::Error::custom(format!("base64 decode error: {}", e)))?;
    String::from_utf8(bytes).map_err(|e| de::Error::custom(format!("utf8 error: {}", e)))
}

const SECRETS: &str = "secrets.yaml";
impl Secrets {
    pub fn load<T>() -> T
    where
        T: DeserializeOwned,
    {
        let file = File::open(SECRETS).expect("Failed to open secres file");
        let buf_reader = BufReader::new(file);
        let value: YamlSecrets<T> =
            serde_yaml::from_reader(buf_reader).expect("Failed to read secrets file");
        value.data
    }
}
