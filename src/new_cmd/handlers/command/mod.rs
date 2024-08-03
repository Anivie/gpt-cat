macro_rules! describe {
    ([$($command_name:tt)|*] help $help:expr) => {
        CommandDescription {
            name: vec![$($command_name,)*],
            help: $help,
            param: None,
            param_description: None,
        }
    };

    (
        [$($command_name:tt)|*] help $help:expr,
        $($param:expr => $description:expr),*,
    ) => {
        CommandDescription {
            name: vec![$($command_name,)*],
            help: $help,
            param: Some(
                vec![$(($param, true),)* ]
            ),
            param_description: Some(
                vec![$($description,)* ]
            ),
        }
    };

    (
        [$($command_name:tt)|*] help $help:expr,
        $($param:expr => $description:expr),*,
        $(($optional_param:expr) => $optional_description:expr);*;
    ) => {
        CommandDescription {
            name: vec![$($command_name,)*],
            help: $help,
            param: Some(
                vec![
                    $(($param, true),)*
                    $(($optional_param, false),)*
                ]
            ),
            param_description: Some(
                vec![
                    $($description,)*
                    $(($optional_description),)*
                ]
            ),
        }
    };

    (
        [$($command_name:tt)|*] help $help:expr,
        $(($optional_param:expr) => $optional_description:expr);*;
    ) => {
        CommandDescription {
            name: vec![$($command_name,)*],
            help: $help,
            param: Some(
                vec![$(($optional_param, false),)* ]
            ),
            param_description: Some(
                vec![$(($optional_description),)* ]
            ),
        }
    };
}


pub(in crate::new_cmd::handlers) mod list_account;
pub(in crate::new_cmd::handlers) mod add_user;
pub(in crate::new_cmd::handlers) mod edit_balance;
pub(in crate::new_cmd::handlers) mod search_balance;
pub(in crate::new_cmd::handlers) mod search_user;
pub(in crate::new_cmd::handlers) mod manage_account_pool;