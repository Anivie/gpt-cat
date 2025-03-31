use crate::commandline::handlers::describer::CommandDescription;
use crate::http::server::pre_handler::{ClientJoinContext, PreHandlerResult};
use anyhow::Result;

pub(super) mod say_hi;
pub(super) mod template;
pub(super) mod balance_inquiry;
pub(super) mod show_price;

pub(super) trait CommandHandler {
    fn description(&self) -> CommandDescription;
    async fn execute(&self, context: &mut ClientJoinContext<'_>, args: &Vec<&str>) -> Result<PreHandlerResult>;
}

macro_rules! command_handler_dispatcher {
        ($($dispatcher:ident),*) => {
            use anyhow::Result;
            use crate::commandline::handlers::describer::CommandDescription;
            use crate::http::server::pre_handler::command::handlers::CommandHandler;
            use crate::http::server::{ClientJoinContext, PreHandlerResult};

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

                async fn execute(&self, context: &mut ClientJoinContext<'_>, args: &Vec<&str>) -> Result<PreHandlerResult> {
                    match self {
                        $(
                            CommandHandlerDispatcher::$dispatcher(dispatcher) => dispatcher.execute(context, args).await,
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