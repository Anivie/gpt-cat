use crate::data::config::runtime_data::GlobalData;

pub(super) trait CommandHandler {
    fn name(&self) -> Vec<&str>;
    fn help(&self) -> &str;//todo: should return a struct
    async fn execute(&self, global_data: &GlobalData, args: &Vec<&str>) -> anyhow::Result<()>;
}

macro_rules! command_handler_dispatcher {
        ($($dispatcher:ident),*) => {
            use crate::new_cmd::handlers::dispatcher::CommandHandler;

            pub(super) enum CommandHandlerDispatcher {
                $(
                    $dispatcher($dispatcher),
                )*
            }

            impl CommandHandler for CommandHandlerDispatcher {
                fn name(&self) -> Vec<&str> {
                    match self {
                        $(
                            CommandHandlerDispatcher::$dispatcher(dispatcher) => dispatcher.name(),
                        )*
                    }
                }

                fn help(&self) -> &str {
                    match self {
                        $(
                            CommandHandlerDispatcher::$dispatcher(dispatcher) => dispatcher.help(),
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

            pub(super) fn new_command_handler_dispatcher() -> Vec<CommandHandlerDispatcher> {
                vec![
                    $(
                        CommandHandlerDispatcher::$dispatcher($dispatcher::default()),
                    )*
                ]
            }
    };
}
