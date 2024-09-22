use crate::commandline::handlers::describer::CommandDescription;
use crate::data::database::entity::user_command::{DataBasePrivateCommand, DataBasePublicCommand};
use crate::data::http_api::openai::openai_request::Message;
use crate::http::client::client_sender::channel_manager::ChannelSender;
use crate::http::server::pre_handler::command::handlers::CommandHandler;
use crate::http::server::pre_handler::{ClientJoinContext, PreHandlerResult};
use anyhow::anyhow;
use anyhow::Result;
use cat_macro::describe;
use log::{error, info};

#[derive(Default)]
pub struct TemplateHandler;

async fn generate_help_message(context: &mut ClientJoinContext<'_>) -> Result<String> {
    let public_commands: Vec<DataBasePublicCommand> = sqlx::query_as!(
            DataBasePublicCommand,
            "SELECT * FROM public_command"
        )
        .fetch_all(&context.global_data.data_base)
        .await?;

    let private_commands: Vec<DataBasePrivateCommand> = sqlx::query_as!(
            DataBasePrivateCommand,
            "SELECT * FROM private_command WHERE user_id = $1",
            context.user_id
        )
        .fetch_all(&context.global_data.data_base)
        .await?;

    let mut help_message = String::from("# 🛠️ 帮助页面\n\n欢迎使用本平台！以下是您可以使用的一些模板，分为公共模板和私有模板：\n\n## 🌐 全局模板\n\n| 📋 名称          | 📝 描述\t\t      |\n|------------------|----------------------------------|\n");

    for command in public_commands {
        help_message.push_str(&format!("|`{}`|{}|\n", command.command, command.describe));
    }

    if private_commands.is_empty() {
        help_message.push_str("\n## 🔒 私有模板\n\n您还没有添加任何私有模板！\n");
    }else {
        help_message.push_str("\n## 🔒 私有模板\n\n只有您可以使用这些命令：\n\n| 📋 名称\t  | 📝 描述\t\t        |\n|---------------------|------------------------------------|\n");

        for command in private_commands {
            help_message.push_str(&format!("|`{}`|{}|\n", command.command, command.describe));
        }
    }

    help_message.push_str("\n---\n\n**提示：**\n- 使用模板时，请确保模板是否存在额外要求，如特定的询问方式等。\n- 如果您需要更多帮助或指导，请随时使用`help`命令获取详细信息！\n");

    Ok(help_message)
}

impl CommandHandler for TemplateHandler {
    fn description(&self) -> CommandDescription {
        describe! {
            ["template" | "t"] help "将模板应用到当前对话中"
            example "`/t translate` -> 翻译一段文本\n`/t end-template translate` -> 添加自定义模板\n`/t help` -> 查看当前可用模板";
            "template_name" => "The name of the template you want to use.",
            ("end-template [模板名称] [模板描述]") => "添加自定义模板，可供后续使用",
            ("help") => "查看当前可用模板",
        }
    }

    async fn execute(&self, context: &mut ClientJoinContext<'_>, args: &Vec<&str>) -> Result<PreHandlerResult> {
        let &template_name = args.get(0).ok_or(anyhow!("Missing template name"))?;
        if template_name.is_empty() {
            return Err(anyhow!("Template name cannot be empty!"));
        }

        if template_name == "help" {
            let help_message = generate_help_message(context).await?;
            context.sender.send_text(help_message.as_str(), true).await?;
            return Ok(PreHandlerResult::Return);
        }

        if template_name == "end-template" {
            let prompt_messages = serde_json::to_string(&context.sender.request.messages).map_err(|e| {
                error!("Error when serializing prompt messages: {:?}", e);
                anyhow!("Error when serializing prompt messages!")
            })?;
            let &template_name = args.get(1).ok_or(anyhow!("Missing custom template name."))?;
            let &template_describe = args.get(2).ok_or(anyhow!("Missing custom template describe."))?;

            let result = sqlx::query!(
                "INSERT INTO private_command (user_id, command, describe, prompt) VALUES ($1, $2, $3, $4)",
                context.user_id, template_name, template_describe, prompt_messages
            )
                .execute(&context.global_data.data_base)
                .await?;

            return if result.rows_affected() == 0 {
                 Err(anyhow!("Failed to save template!"))
            }else {
                context.sender.send_text("Template saved successfully!", true).await?;
                Ok(PreHandlerResult::Return)
            }
        }

        let private_command: Result<DataBasePrivateCommand, sqlx::Error> = sqlx::query_as!(
                            DataBasePrivateCommand,
                            "SELECT * FROM private_command WHERE user_id = $1 AND command = $2 LIMIT 1",
                            context.user_id,
                            template_name
                        )
            .fetch_one(&context.global_data.data_base)
            .await;

        if let Err(sqlx::Error::RowNotFound) = private_command {
            let public_command: Result<DataBasePublicCommand, sqlx::Error> = sqlx::query_as!(
                        DataBasePublicCommand,
                        "SELECT * FROM public_command WHERE command = $1 LIMIT 1",
                        template_name
                    )
                .fetch_one(&context.global_data.data_base)
                .await;

            if let Err(sqlx::Error::RowNotFound) = public_command {
                return Err(anyhow!("Template not found!"));
            }

            let public_command = public_command.map_err(|e| {
                error!("Error when fetching public command: {:?}", e);
                anyhow!("Error when fetching command!")
            })?;

            apply_template(context, public_command.prompt.as_str())?;
        }else {
            let private_command = private_command.map_err(|e| {
                error!("Error when fetching private command: {:?}", e);
                anyhow!("Error when fetching command!")
            })?;

            apply_template(context, private_command.prompt.as_str())?;
        }

        info!("User {:?} used template {}", context.user_id, template_name);
        Ok(PreHandlerResult::Pass)
    }
}

fn apply_template(context: &mut ClientJoinContext, command: &str) -> Result<()> {
    let index = context.sender.request.messages
        .iter()
        .enumerate()
        .filter(|(_, x)| x.content.starts_with("/t") || x.content.starts_with("/template"))
        .map(|(index, _)| index)
        .next()
        .ok_or(anyhow!("Could not find command in message list!"))?;
    context.sender.request.messages.remove(index);

    let mut prompt_messages = serde_json::from_str::<Vec<Message>>(command)
        .map_err(|e| {
            error!("Error when parsing template: {:?}", e);
            anyhow!("Error when parsing template!")
        })?;

    for _ in 0..prompt_messages.len() {
        let message = prompt_messages.pop().unwrap();
        context.sender.request.messages.insert(index, message);
    }

    Ok(())
}