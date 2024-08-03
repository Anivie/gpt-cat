//Macros to help define commands
/*macro_rules! define_commands {
    ($parts:ident, $([$command:expr $(,$args:ident)*] => $action:block help $help:expr),* $(,)?) => {
        use colored::Colorize;
        use std::sync::LazyLock;

        static HELP_STRING: LazyLock<String> = LazyLock::new(|| {
            use colored::Colorize;
            let mut back = String::from('\n');
            $(
                {
                    let tmp = if stringify!($($args)*).is_empty() {
                        format!(
                            "{}: {}",
                            $command.truecolor(5,242,196),
                            $help.truecolor(34, 155, 230)
                        )
                    } else {
                        let args = stringify!($($args),*).split(", ").collect::<Vec<_>>();
                        let mut tmp = String::new();
                        tmp.push_str($command.truecolor(5,242,196).to_string().as_str());
                        tmp.push(' ');
                        for (index, arg) in args.iter().enumerate() {
                            tmp.push('[');
                            tmp.push_str(arg.truecolor(5,210,242).to_string().as_str());
                            tmp.push(']');
                            if index != args.len() - 1 {
                                tmp.push(' ');
                            }
                        }
                        tmp.push_str(": ");
                        tmp.push_str($help.truecolor(34, 155, 230).to_string().as_str());
                        tmp
                    };

                    back.push_str(tmp.as_str());
                    back.push('\n');
                }
            )*
            back
        });

        match $parts.as_slice() {
            $(
                [$command, $($args),*] => {
                    $action
                }
            )*
            ["help"] | ["h"] => {
                info!("{}", "Available commands:".magenta());
                info!("{}", *HELP_STRING);
            }
            _ => {
                info!("{}", "Unknown command.".magenta());
                info!("{}", "Available commands:".magenta());
                info!("{}", *HELP_STRING);
            }
        }
    };
}*/

//Macro to find a user id from an api key.
/*macro_rules! find_user_id {
    ($api_key: expr, $database: expr) => {
        User::find()
            .filter(crate::data::database::entities::user::Column::ApiKey.eq($api_key.to_string()))
            .one($database)
            .await
            .unwrap()
    };
}
*/