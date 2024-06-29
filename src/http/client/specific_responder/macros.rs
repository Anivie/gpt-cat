/// Helper macros that process the stream with ResponseParser
macro_rules! process_stream {
    ($stream:expr, $handler:expr, $sender:expr) => {
        use crate::http::client::util::sse::rayon_json_processor::RayonJsonProcessor;
        use crate::http::client::util::sse::sse_processor::SSEProcessor;
        use futures_util::StreamExt;
        use bytes::Bytes;

        let mut interrupt_processor = RayonJsonProcessor::default();
        let mut handler = $handler;
        let mut stream = $stream;

        use std::io::Write;
        let mut file = std::fs::OpenOptions::new();
        let mut file = file.write(true).create(true).open("data.tmp").unwrap();

        while let Some(item) = stream.next().await {
            let item: Bytes = item.map_err(|e| ResponderError::Request(e.to_string()))?;
            let item = item.as_ref();
            // 把数据保存在临时文件data.tmp中
            file.write_all(item).unwrap();

            let (split, first) = interrupt_processor.process(item);
            if let Some(response) = first {
                handler.parse_response($sender, response.as_slice()).await?;
            }

            for response in split {
                handler.parse_response($sender, response).await?;
            }
        }
        handler.parse_end($sender).await?;
    }
}

/// # Enum dispatcher
/// Helper macros that implement the SpecificResponder trait for all specific responder
/// It will generate the ResponderDispatcher enum and the impl block for Endpoint
/// Which can avoid the dynamic dispatch and make the code more efficient
macro_rules! impl_specific_responder {
        ($($endpoint:ident :: $variant:ident with $responder:ident),*) => {
        use crate::data::config::runtime_data::AccountVisitor;

        use crate::http::client::specific_responder::*;
        use crate::http::client::client_sender::channel_manager::ClientSender;
        use crate::data::config::endpoint::Endpoint;

        pub enum ResponderDispatcher {
            $(
                $responder($responder),
            )*
        }

        impl SpecificResponder for ResponderDispatcher {
            async fn make_response(&self,
                                   sender: &mut ClientSender,
                                   accessor: &AccountVisitor,
            ) -> Result<(), ResponderError> {
                use crate::http::client::client_sender::channel_manager::ChannelBufferManager;
                let back = match self {
                    $(
                        ResponderDispatcher::$responder(responder) => responder.make_response(sender, accessor).await,
                    )*
                };

                let buffer = sender.get_buffer();
                let err: Option<Result<(), ResponderError>> = if let Ok(_) = &back && buffer.is_empty() {
                    Some(Err(ResponderError::Request("Request success, but a empty.".to_string())))
                }else {
                    None
                };

                err.unwrap_or(back)
            }
        }

        impl Endpoint {
            pub fn specific_responder_dispatcher(&self) -> ResponderDispatcher {
                match self {
                    $(
                        $endpoint::$variant => ResponderDispatcher::$responder($responder::default()),
                    )*
                }
            }
        }
    };
}