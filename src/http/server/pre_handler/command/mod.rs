use crate::http::server::pre_handler::command::handlers::balance_inquiry::BalanceInquiryHandler;
use crate::http::server::pre_handler::command::handlers::say_hi::SayHi;
use crate::http::server::pre_handler::command::handlers::template::TemplateHandler;

pub mod command_handler;
#[macro_use]
mod handlers;

command_handler_dispatcher! [
    SayHi,
    BalanceInquiryHandler,
    TemplateHandler
];

impl CommandDescription {
    fn help_messages(&self) -> String {
        let command_names = self.name.join("| ");
        let mut parameters = format!("\n###  🔎命令： **[{}]** \n   **描述:** {}\n", command_names, self.help);

        match (&self.param, &self.param_description) {
            (None, None) => {
                parameters.push_str("   - **参数:** 无参数\n");
            }
            (Some(param), Some(param_describe)) => {
                parameters.push_str("   - **参数:** \n");
                for (index, &(param_name, optional)) in param.iter().enumerate() {
                    parameters.push_str(&format!(
                        "     - `{}` {}: {}\n",
                        param_name,
                        if !optional { "(可选)" } else { "" },
                        param_describe[index],
                    ));
                }
            }
            _ => panic!("Unexpected parameter format."),
        }

        if let Some(example) = self.example {
            parameters.push_str(&format!("\n   - **示例:** \n     - {}\n", example));
        }
        parameters.push_str("\n---\n\n");

        parameters
    }
}