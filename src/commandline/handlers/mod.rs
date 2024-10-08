use crate::commandline::handlers::command::add_user::AddUser;
use crate::commandline::handlers::command::edit_balance::EditUserBalance;
use crate::commandline::handlers::command::list_account::ListAccount;
use crate::commandline::handlers::command::list_model::ListModel;
use crate::commandline::handlers::command::manage_account_pool::ManageAccountPool;
use crate::commandline::handlers::command::search_balance::SearchBalance;
use crate::commandline::handlers::command::search_user::SearchUser;
use crate::data::config::entity::runtime_data::GlobalData;

#[macro_use]
pub mod macros;

pub mod command_listener;
pub mod describer;
mod command;

command_handler_dispatcher! {
    ListAccount,
    AddUser,
    EditUserBalance,
    SearchBalance,
    SearchUser,
    ManageAccountPool,
    ListModel
}