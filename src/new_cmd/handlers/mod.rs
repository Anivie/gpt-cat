use crate::data::config::runtime_data::GlobalData;
use crate::new_cmd::handlers::command::add_user::AddUser;
use crate::new_cmd::handlers::command::list_account::ListAccount;

mod command;
pub mod command_listener;

#[macro_use]
mod dispatcher;

command_handler_dispatcher! {
    ListAccount,
    AddUser
}