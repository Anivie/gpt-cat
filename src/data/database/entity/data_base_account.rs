pub struct DataBaseAccount {
    pub id: i32,
    pub is_disabled: bool,
    pub use_proxy: Option<String>,
    pub api_key: String,
    pub endpoint: String
}