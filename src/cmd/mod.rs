//! Command listener for this app.
//! It will listen to stdin and execute commands.

use uuid::Uuid;

#[macro_use]
mod utils;
pub mod hot_reload;

/// Register a command listener, this should be called in a different task.
/*pub async fn add_cmd_listener(global_data: &GlobalData) {
    let mut reader = BufReader::new(tokio::io::stdin());
    let mut buffer = String::new();
    loop {
        select! {
            command = reader.read_line(&mut buffer) => {
                if let Ok(a) = command && a > 0 {
                    let parts: Vec<&str> = buffer.trim().split_whitespace().collect();
                    if process_command(global_data, parts).await { continue; }
                    buffer.clear();
                }
            }

            _ = tokio::signal::ctrl_c() => {
                sleep(Duration::from_millis(200)).await;
                println!("shutting down now.");
                std::process::exit(0);
            }
        }
    }
}*/

/*async fn process_command(global_data: &GlobalData, parts: Vec<&str>) -> bool {
    define_commands! { parts,
        ["adu"] => {
            let key = generate_key();
            let user = user::ActiveModel {
                api_key: Set(key.clone()),
                ..Default::default()
            };
            let user = User::insert(user)
                .exec(&global_data.data_base)
                .await
                .unwrap();

            info!("User {:?} has been added, id: {}.", key, user.last_insert_id);
        } help "add a user with random api key and default balances.",
        ["adu", init_purchase] => {
            let key = generate_key();
            let user = user::ActiveModel {
                api_key: Set(key.clone()),
                ..Default::default()
            };
            let user = User::insert(user)
                .exec(&global_data.data_base)
                .await
                .unwrap();
            let user_usage = UserUsage::find()
                .filter(user_usage::Column::UserId.eq(user.last_insert_id))
                .one(&global_data.data_base)
                .await
                .unwrap();
            match user_usage {
                None => {
                    info!("internal error: user usage not found.");
                }
                Some(usage) => {
                    let mut usage: user_usage::ActiveModel = usage.into();
                    usage.total_purchased = Set(init_purchase.parse().unwrap());
                    let update = UserUsage::update(usage)
                        .exec(&global_data.data_base)
                        .await
                        .unwrap();
                    info!("User {:?} has been added, id: {}, init purchase: {}.", key, user.last_insert_id, update.total_purchased);
                }
            }
        } help "add user with init purchase",
        ["dae", endpoint, disable] => {
            let target = if *disable == "t" {
                true
            } else if *disable == "f" {
                false
            } else {
                info!("invalid argument, you only can type 't' or 'f'.");
                return true;
            };

            let endpoint_str = Endpoint::from_str(*endpoint).to_string();
            let account = AccountList::find()
                .filter(
                    account_list::Column::Endpoint.eq(endpoint_str.clone())
                        .and(account_list::Column::IsDisabled.eq(!target))
                )
                .all(&global_data.data_base)
                .await
                .unwrap();

            if account.is_empty() {
                info!("No eligible account could be found.");
                return true;
            }

            for account in account {
                let mut account: account_list::ActiveModel = account.into();
                account.is_disabled = Set(target);
                let update = AccountList::update(account)
                    .exec(&global_data.data_base)
                    .await
                    .unwrap();
                info!("{:?}'s account has been {}.", update.endpoint, if target {"disabled"} else {"enabled"});
            }

            info!("Now change the account pool in memory, please wait for existing client remove.");
            let mut account_pool = global_data.account_pool.write();
            info!("Account lock got.");
            if target {
                let mut account_need_remove = vec![];
                for (index, value) in account_pool.iter().enumerate() {
                    if value.get_endpoint().to_string() == endpoint_str {
                        account_need_remove.push(index);
                    }
                }
                account_need_remove.reverse();
                for index in account_need_remove {
                    account_pool.remove(index);
                }
            } else {
                let config = global_data.config.read();
                let account_need_join = AccountList::find()
                    .filter(account_list::Column::Endpoint.eq(endpoint_str))
                    .all(&global_data.data_base)
                    .await
                    .unwrap()
                    .into_iter()
                    .map(|account| {
                        let endpoint = Endpoint::from_str(&account.endpoint);
                        let client = get_client(&account.use_proxy, config.deref(), &endpoint, &account.password);
                        AccountVisitor {
                            endpoint: endpoint.clone(),

                            account_id: account.id,
                            endpoint_url: endpoint.to_url(config.deref()),

                            responder: endpoint.specific_responder_dispatcher(),

                            client,
                        }
                    })
                    .collect::<Vec<AccountVisitor>>();
                let config = global_data.config.read();
                *account_pool = account_need_join.to_vec_safe_pool(config.request_concurrency_count);
            }
            info!("Account pool has been changed, now {} accounts in pool.", account_pool.len());
        } help "enable/disable some account by it's endpoint",
        ["la"] => {
            let account = global_data.account_pool.read();
            let account = account.deref();
            info!("total {} accounts found.", account.len())
        } help "list accounts in account pool.",
        ["edco", account, cookie] => {
            let account = AccountList::find()
                .filter(account_list::Column::Username.eq(account.to_string()))
                .one(&global_data.data_base)
                .await
                .unwrap();
            match account {
                None => {
                    info!("Account not found.");
                    return true;
                }
                Some(account) => {
                    let mut account: account_list::ActiveModel = account.into();
                    account.password = Set(cookie.to_string());
                    let update = AccountList::update(account)
                        .exec(&global_data.data_base)
                        .await
                        .unwrap();
                    info!("{:?}'s cookie has been updated.", update.username);
                }
            }
        } help "edit cookie",
        ["rmchat", apikey] => {
            let user = find_user_id!(apikey, &global_data.data_base);
            match user {
                None => {
                    info!("User not found.");
                    return true;
                }
                Some(user_id) => {
                    let delete = ChatList::delete_many()
                        .filter(crate::data::database::entities::chat_list::Column::UserId.eq(user_id.id))
                        .exec(&global_data.data_base)
                        .await
                        .unwrap();
                    info!("{}'s {} chat list has been cleared.", delete.rows_affected, apikey);
                }
            }
        } help "remove chat list",
        ["uu", apikey] => {
            let user = find_user_id!(apikey, &global_data.data_base);
            match user {
                None => {
                    info!("User not found.");
                    return true;
                }
                Some(user) => {
                    let user_usage = UserUsage::find()
                        .filter(user_usage::Column::UserId.eq(user.id))
                        .one(&global_data.data_base)
                        .await
                        .unwrap();
                    match user_usage {
                        None => {
                            info!("User usage not found.");
                            return true;
                        }
                        Some(user_usage) => {
                            info!("{:?}'s usage message: {:?}", user, user_usage);
                        }
                    }
                }
            }
        } help "Showing the utilization of account balances for specified users",
        ["eduu", apikey, purchased] => {
            let user = find_user_id!(apikey, &global_data.data_base);
            match user {
                None => {
                    info!("User not found.");
                    return true;
                }
                Some(user) => {
                    let user_usage = UserUsage::find()
                        .filter(user_usage::Column::UserId.eq(user.id))
                        .one(&global_data.data_base)
                        .await
                        .unwrap();
                    match user_usage {
                        None => {
                            info!("User usage not found.");
                            return true;
                        }
                        Some(user_usage) => {
                            let mut user_usage: user_usage::ActiveModel = user_usage.into();
                            user_usage.total_purchased = Set(purchased.parse().unwrap());
                            let update = UserUsage::update(user_usage)
                                .exec(&global_data.data_base)
                                .await
                                .unwrap();
                            info!("{:?}'s usage has been updated.", update);
                        }
                    }
                }
            }
        } help "edit user usage",
        ["lstus"] => {
            let users = User::find().all(&global_data.data_base).await.unwrap();
            info!("User list:");
            for user in users.iter() {
                info!("{:?}", user);
            }
            info!("{} users in total.", users.len());
        } help "list users",
        ["lstuu"] => {
            let users = UserUsage::find().all(&global_data.data_base).await.unwrap();
            info!("User usage:");
            for user in users.iter() {
                info!("{:?}", user);
            }
            info!("{} users in total.", users.len());
        } help "list user usages",
    }
    false
}*/

fn generate_key() -> String {
    let base = Uuid::new_v4().to_string().replace("-", "");
    let extra = Uuid::new_v4().to_string();
    format!(
        "sk-{}{}{}{}",
        base,
        &extra[0..8],
        &extra[9..13],
        &extra[19..23]
    )
}
