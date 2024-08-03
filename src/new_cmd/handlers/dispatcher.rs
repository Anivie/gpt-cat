use crate::data::config::runtime_data::GlobalData;

pub(super) struct CommandDescription {
    pub name: Vec<&'static str>,
    pub help: &'static str,

    pub param: Option<Vec<(&'static str, bool)>>,
    pub param_description: Option<Vec<&'static str>>,
}

pub(super) trait CommandHandler {
    fn description(&self) -> CommandDescription;
    async fn execute(&self, global_data: &GlobalData, args: &Vec<&str>) -> anyhow::Result<()>;
}

impl CommandDescription {
    pub fn help_message(&self) -> String {
        // 开始构建命令的基本形式
        let command_names = self.name.join(",");
        let mut help_msg = format!("[{}]: {}", command_names, self.help);

        // 检查是否存在参数和参数描述
        if let (Some(params), Some(descriptions)) = (&self.param, &self.param_description) {
            if params.len() == descriptions.len() {
                // 逐一添加参数及其描述
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
                // 将参数信息添加到帮助信息中
                help_msg.push_str(&format!("\n{}", param_details.join("\n")));
            } else {
                // 如果参数数量和描述数量不匹配，返回一个错误信息
                help_msg.push_str("\n错误: 参数与描述数量不匹配。");
            }
        }

        help_msg
    }
}