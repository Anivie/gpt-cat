pub struct DataBaseUsageList {
    pub id: i32,
    pub user_id: i32,
    pub timestamp: time::Time,
    pub input_tokens: i32,
    pub output_tokens: i32,
    pub input_token_price: f64,
    pub output_token_price: f64
}
