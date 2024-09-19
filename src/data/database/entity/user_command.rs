pub struct DataBasePublicCommand {
    pub id: i32,
    pub command: String,
    pub describe: String,
    pub prompt: String,
    pub is_disable: bool
}

pub struct DataBasePrivateCommand {
    pub id: i32,
    pub user_id: Option<i32>,
    pub command: String,
    pub describe: String,
    pub prompt: String,
    pub is_disable: bool
}