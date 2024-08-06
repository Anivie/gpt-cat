use crate::data::config::entity::runtime_data::GlobalData;
use crate::commandline::handlers::command::add_user::AddUser;
use crate::commandline::handlers::command::edit_balance::EditUserBalance;
use crate::commandline::handlers::command::list_account::ListAccount;
use crate::commandline::handlers::command::manage_account_pool::ManageAccountPool;
use crate::commandline::handlers::command::search_balance::SearchBalance;
use crate::commandline::handlers::command::search_user::SearchUser;

mod command;
pub mod command_listener;
mod dispatcher;

#[macro_use]
mod macros;

command_handler_dispatcher! {
    ListAccount,
    AddUser,
    EditUserBalance,
    SearchBalance,
    SearchUser,
    ManageAccountPool
}