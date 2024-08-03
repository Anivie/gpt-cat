use std::sync::LazyLock;
use std::time::Duration;

use colored::Colorize;
use log::info;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::select;
use tokio::time::sleep;

use crate::data::config::runtime_data::GlobalData;
use crate::new_cmd::handlers::{CommandHandlerDispatcher, new_command_handler_dispatcher};
use crate::new_cmd::handlers::dispatcher::CommandHandler;

/// Register a command listener, this should be called in a different task.
pub async fn add_cmd_listener(global_data: &GlobalData) {
    let mut reader = BufReader::new(tokio::io::stdin());
    let mut buffer = String::new();
    loop {
        select! {
            command = reader.read_line(&mut buffer) => {
                static HANDLER: LazyLock<Vec<CommandHandlerDispatcher>> = LazyLock::new(|| new_command_handler_dispatcher());

                if let Ok(a) = command && a > 0 {
                    let mut parts: Vec<&str> = buffer.trim().split_whitespace().collect();
                    if let Some(&first) = parts.first() && first == "help" {
                        continue;
                    }

                    let mut running = false;
                    for x in HANDLER.iter() {
                        if x.name().contains(&parts[0]) {
                            parts.pop();
                            x.execute(global_data, &parts).await.expect("Failed to execute command");
                            running = true;
                            break;
                        }
                    }

                    if !running {
                        info!("Command: '{}' not found.", parts[0].red());
                    }

                    buffer.clear();
                }
            }

            _ = tokio::signal::ctrl_c() => {
                sleep(Duration::from_millis(200)).await;
                info!("shutting down now.");
                std::process::exit(0);
            }
        }
    }
}