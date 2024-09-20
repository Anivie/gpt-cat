use rust_decimal::Decimal;

pub struct UserUsage {
    pub usage_id: Option<i32>,
    pub user_id: Option<i32>,
    pub total_input_tokens: i64,
    pub total_output_tokens: i64,
    pub total_purchased: Decimal,
}