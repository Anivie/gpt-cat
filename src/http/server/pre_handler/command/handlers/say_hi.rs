use crate::commandline::handlers::describer::CommandDescription;
use crate::describe;
use crate::http::client::client_sender::channel_manager::ChannelSender;
use crate::http::server::pre_handler::command::handlers::CommandHandler;
use crate::http::server::pre_handler::{ClientJoinContext, PreHandlerResult};

#[derive(Default)]
pub struct SayHi;

impl CommandHandler for SayHi {
    fn description(&self) -> CommandDescription {
        describe! {
            ["say_hi" | "sh"] help "Say hi to the user.",
            ("name") => "The name of the user.";
        }
    }

    async fn execute(&self, context: &mut ClientJoinContext<'_>, args: &Vec<&str>) -> anyhow::Result<PreHandlerResult> {
        let name = args.get(0).unwrap_or(&"world");
        context.sender.send_text(&format!("Hi, {}!", name), false).await?;
        Ok(PreHandlerResult::Return)
    }
}