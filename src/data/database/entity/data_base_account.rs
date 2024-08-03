pub struct DataBaseAccount {
    pub id: i32,
    pub is_disabled: bool,
    pub use_proxy: Option<String>,
    pub username: Option<String>,
    pub password: String,
    pub endpoint: String
}