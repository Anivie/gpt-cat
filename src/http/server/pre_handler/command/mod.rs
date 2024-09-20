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
        let command_names = self.name.join(", ");
        let description = format!("### ▶️ [{}]：{}", command_names, self.help);
        let mut parameters = String::new();

        match (&self.param, &self.param_description) {
            (None, None) => {
                parameters.push_str("无参数\n");
            }
            (Some(param), Some(param_describe)) => {
                for (index, &(param_name, optional)) in param.iter().enumerate() {
                    if !optional {
                        parameters.push_str(&format!("- **{}** _(可选)_：{}\n", param_name, param_describe[index]));
                    } else {
                        parameters.push_str(&format!("- **{}**：{}\n", param_name, param_describe[index]));
                    }
                }
            }
            _ => panic!("Unexpected parameter format."),
        }

        format!("{}\n{}\n---\n", description, parameters)
    }
}