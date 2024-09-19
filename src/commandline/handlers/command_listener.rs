use std::ops::Deref;
use std::sync::LazyLock;
use std::thread::sleep;
use std::time::Duration;

use colored::Colorize;
use log::{error, info};
use rayon::prelude::*;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
use tokio::spawn;

use crate::commandline::handlers::describer::CommandHandler;
use crate::commandline::handlers::{new_command_handler_dispatcher, CommandHandlerDispatcher};
use crate::data::config::entity::runtime_data::GlobalData;

/// Register a command listener, this should be called in a different task.
pub fn add_cmd_listener(global_data: &'static GlobalData) {
    let mut rl = DefaultEditor::new().expect("Failed to create editor");
    loop {
        let readline = rl.readline(">> ");

        match readline {
            Ok(command) => {
                rl.add_history_entry(command.as_str()).expect("Failed to add history entry");

                static HANDLER: LazyLock<Vec<CommandHandlerDispatcher>> = LazyLock::new(|| new_command_handler_dispatcher());
                static HELP_MESSAGE: LazyLock<String> = LazyLock::new(|| {
                    let mut back = HANDLER
                        .par_iter()
                        .map(|x| {
                            let description = x.description().help_message();
                            format!("{}\n\n", description)
                        })
                        .collect::<String>();
                    back.pop();
                    back.pop();

                    back
                });

                let parts: Vec<&str> = command.trim().split_whitespace().collect();

                if parts.is_empty() {
                    println!("{}", HELP_MESSAGE.deref());
                    continue;
                }

                if let Some(&first) = parts.first() &&
                    (first.is_empty() || first == "help" || first == "h")
                {
                    println!("{}", HELP_MESSAGE.deref());
                    continue;
                }

                let mut running = false;
                for x in HANDLER.iter() {
                    if x.description().name.contains(&parts[0]) {
                        let mut args: Vec<String> = parts.iter().map(|x| x.to_string()).collect();
                        args.remove(0);
                        spawn(async move {
                            let args: Vec<&str> = args.iter().map(|x| x.as_str()).collect();
                            if let Err(err) = x.execute(global_data, &args).await {
                                error!("Error when execute command: {}", err);
                            }
                        });
                        running = true;
                        break;
                    }
                }

                if !running {
                    info!("Command: '{}' not found.", parts[0].red());
                }
            }
            Err(ReadlineError::Interrupted) => {
                info!("Server is shutting down, please wait.");
                sleep(Duration::from_millis(200));
                info!("shutting down now.");
                std::process::exit(0);
            },
            Err(_) => {}
        }
    }
}