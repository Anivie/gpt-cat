use anyhow::anyhow;
use sea_orm::ColumnTrait;
use sea_orm::EntityTrait;
use sea_orm::QueryFilter;

use crate::data::database::entities::prelude::User;
use crate::http::server::pre_handler::{ClientJoinContext, ClientJoinPreHandlerImpl};

#[derive(Default, Clone)]
pub(crate) struct UserIDHandler;

impl ClientJoinPreHandlerImpl for UserIDHandler {
    async fn client_join<'a>(
        &'a self,
        context: &mut ClientJoinContext<'a>,
    ) -> anyhow::Result<Option<String>> {
        let user_id = if let Some(auth) = &context.user_key {
            let user = User::find()
                .filter(crate::data::database::entities::user::Column::ApiKey.eq(auth))
                .one(&context.global_data.data_base)
                .await
                .unwrap();
            match user {
                None => {
                    // return Err(anyhow!("Invalid key: {}, please ensure that you have already put a key.", auth));
                    return Err(anyhow!(
                        "无效的Key: {}, 请输入正确的Key或检查拼写是否正确",
                        auth
                    ));
                }
                Some(user) => {
                    if user.is_active {
                        user.id
                    } else {
                        // return Err(anyhow!("Account is inactive, try to ensure your account has not ran out of your usage limit then contact THE cat."));
                        return Err(anyhow!(
                            "帐户处于非活动状态，请尝试检查您的账户是否已超出额度"
                        ));
                    }
                }
            }
        } else {
            // return Err(anyhow!("KEY not found, please set a key in your client."));
            return Err(anyhow!("未找到KEY，请在您的客户端中设置KEY"));
        };

        context.user_id.replace(user_id);
        Ok(None)
    }
}
