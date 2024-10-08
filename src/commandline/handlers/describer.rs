use crate::data::config::entity::runtime_data::GlobalData;

pub struct CommandDescription {
    pub name: Vec<&'static str>,
    pub help: &'static str,
    pub example: Option<&'static str>,

    pub param: Option<Vec<(&'static str, bool)>>,
    pub param_description: Option<Vec<&'static str>>,
}

pub(super) trait CommandHandler {
    fn description(&self) -> CommandDescription;
    async fn execute(&self, global_data: &GlobalData, args: &Vec<&str>) -> anyhow::Result<()>;
}

impl CommandDescription {
    pub(super) fn help_message(&self) -> String {
        let command_names = self.name.join(",");
        let mut help_msg = format!("[{}]: {}", command_names, self.help);

        if let (Some(params), Some(descriptions)) = (&self.param, &self.param_description) {
            if params.len() == descriptions.len() {
                let param_details: Vec<String> = params
                    .iter()
                    .zip(descriptions.iter())
                    .map(|((param_name, optional), desc)| {
                        if !(*optional) {
                            format!("- {}(可选): {}", param_name, desc)
                        } else {
                            format!("- {}: {}", param_name, desc)
                        }
                    })
                    .collect();
                help_msg.push_str(&format!("\n{}", param_details.join("\n")));
            } else {
                help_msg.push_str("\n错误: 参数与描述数量不匹配。");
            }
        }

        help_msg
    }
}