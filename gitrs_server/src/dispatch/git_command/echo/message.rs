#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum Outbound {
    Result { output: String },
}
