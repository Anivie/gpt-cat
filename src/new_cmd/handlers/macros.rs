macro_rules! command_handler_dispatcher {
        ($($dispatcher:ident),*) => {
            use crate::new_cmd::handlers::dispatcher::{CommandHandler, CommandDescription};

            enum CommandHandlerDispatcher {
                $(
                    $dispatcher($dispatcher),
                )*
            }

            impl CommandHandler for CommandHandlerDispatcher {
                fn description(&self) -> CommandDescription {
                    match self {
                        $(
                            CommandHandlerDispatcher::$dispatcher(dispatcher) => dispatcher.description(),
                        )*
                    }
                }

                async fn execute(&self, global_data: &GlobalData, args: &Vec<&str>) -> anyhow::Result<()> {
                    match self {
                        $(
                            CommandHandlerDispatcher::$dispatcher(dispatcher) => dispatcher.execute(global_data, args).await,
                        )*
                    }
                }
            }

            fn new_command_handler_dispatcher() -> Vec<CommandHandlerDispatcher> {
                vec![
                    $(
                        CommandHandlerDispatcher::$dispatcher($dispatcher::default()),
                    )*
                ]
            }
    };
}
