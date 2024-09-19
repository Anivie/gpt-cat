macro_rules! impl_client_join_handler {
    ($($variant:ident),*) => {
        use crate::http::server::pre_handler::{ClientJoinPreHandlerImpl, ClientJoinContext, PreHandlerResult};
        use crate::http::server::pre_handler::dispatcher::pre_handler_dispatcher::ClientJoinHandlers;
        use anyhow::Result;

        #[derive(Clone)]
        pub enum ClientJoinPreHandler {
            $(
                $variant($variant),
            )*
        }

        impl ClientJoinPreHandlerImpl for ClientJoinPreHandler {
            async fn client_join<'a>(&'a self, context: &mut ClientJoinContext<'a>) -> Result<PreHandlerResult> {
                match self {
                    $(
                        ClientJoinPreHandler::$variant(handler) => handler.client_join(context).await,
                    )*
                }
            }
        }

        pub fn get_client_join_handler() -> ClientJoinHandlers {
            ClientJoinHandlers::new(vec![
                $(
                    ClientJoinPreHandler::$variant($variant::default()),
                )*
            ])
        }
    }
}