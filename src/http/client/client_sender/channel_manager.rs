use crate::data::http_api::openai::openai_request::OpenAIRequest;
use crate::data::http_api::openai::openai_stream_response::OpenAIStreamResponse;
use crate::data::http_api::openai::openai_sync_response::OpenAISyncResponse;
use anyhow::Result;
use log::{debug, error, info};
use ntex::util::Bytes;
use std::time::{Duration, Instant};
use tokio::spawn;
use tokio::sync::mpsc::Sender;
use tokio::sync::Mutex;
use tokio::time::interval;

/// This struct represents an error that occurred while processing a request.
/// It contains information about the component that caused the error, the reason for the error,
/// the error message, and an optional suggestion for how to fix the error.
/// This struct is used to send error messages to the client.
/// The client will display the error message to the user.
#[derive(Debug)]
pub struct ResponsiveError {
    pub component: String,
    pub reason: String,
    pub message: String,
    pub suggestion: Option<String>,
}

pub type ClientSenderInner = Sender<Bytes>;

/// This struct represents a channel that is used to communicate with the client.
/// # Fields
/// * `inner` - The sender that is used to send messages to the client.
/// * `error_message` - A list of error messages that have occurred while processing the request.
/// * `buffer` - A buffer that is used to store messages that are sent to the client.
/// * `request` - The request that is sending from client.
/// * `is_stream` - A flag that indicates whether the request is a stream request.
#[derive(Debug)]
pub struct ClientSender {
    inner: ClientSenderInner,
    error_message: Vec<ResponsiveError>,
    buffer: String,
    is_empty: bool,
    last_activity: &'static mut Mutex<Instant>,

    pub stopped: bool,
    pub request: OpenAIRequest,
}

impl ClientSender {
    pub fn new(inner: ClientSenderInner, request: OpenAIRequest) -> Self {
        let last_activity = Box::leak(Box::new(Mutex::new(Instant::now())));
        if request.is_stream() {
            let sender_weak = inner.downgrade();
            let last_activity_clone = last_activity as *const Mutex<Instant> as usize;
            let last_activity_clone = unsafe {
                &mut *(last_activity_clone as *mut Mutex<Instant>)
            };

            // 启动心跳检查任务
            debug!("Every sender will keep alive for 25 seconds");
            spawn(async move {
                let mut interval = interval(Duration::from_secs(5));

                loop {
                    interval.tick().await;

                    if let Some(sender) = sender_weak.upgrade() {
                        let mut last = last_activity_clone.lock().await;
                        let should_send_heartbeat = {
                            last.elapsed() > Duration::from_secs(30)
                        };

                        if should_send_heartbeat {
                            info!("Send heartbeat to client");
                            if let Err(e) = sender.send(Bytes::from(
                                concat!(r#"data:  {"id":"chatcmpl-9709rQdvMSIASrvcWGVsJMQouP2UV","object":"chat.completion.chunk","created":1746818209,"model":"heartbeat","system_fingerprint":"fp_3bc1b5746c","choices":[{"index":0,"delta":{"content":""},"logprobs":null,"finish_reason":null}]}"#, "\n\n")
                            )).await {
                                error!("Error when send heartbeat to client: {}", e);
                                break;
                            }
                            *last = Instant::now();
                        }

                        drop(last);
                        drop(sender);
                    }else {
                        drop(unsafe {
                            Box::from_raw(last_activity_clone)
                        });
                        break;
                    }
                }
            });
        }

        Self {
            inner,
            request,
            is_empty: true,
            stopped: false,
            buffer: String::new(),
            error_message: Vec::new(),
            last_activity,
        }
    }

    pub fn is_stream(&self) -> bool {
        self.request.stream.unwrap_or(false)
    }

    pub fn is_empty(&self) -> bool {
        self.is_empty && self.buffer.is_empty()
    }

    pub fn not_empty(&mut self) {
        self.is_empty = false;
    }
}

/// This trait defines the methods that are used to manage the channel buffer.
/// The channel buffer is used to store messages that are sent to the client.
pub trait ChannelBufferManager {
    fn append_buffer(&mut self, buffer: &str);
    async fn push_buffer(&self) -> Result<()>;
    fn get_buffer(&self) -> &str;
}

impl ChannelBufferManager for ClientSender {
    fn append_buffer(&mut self, buffer: &str) {
        self.buffer.push_str(buffer)
    }

    async fn push_buffer(&self) -> Result<()> {
        self.send_text(self.buffer.as_str(), true).await
    }

    fn get_buffer(&self) -> &str {
        self.buffer.as_str()
    }
}

impl Drop for ClientSender {
    fn drop(&mut self) {
        if self.is_stream() && !self.inner.is_closed() {
            let sender = self.inner.clone();
            spawn(async move {
                sender.send(Bytes::from("data: [DONE]\n\n")).await.unwrap();
            });
        }
    }
}

/// This trait defines the methods that are used to send messages to the client.
/// The client is responsible for displaying the messages to the user.
/// The messages can be text messages, JSON messages, or error messages.
/// The client is also responsible for handling the messages that are sent to it.
pub trait ChannelSender {
    async fn send(&self, buffer: Vec<u8>) -> Result<()>;
    async fn send_text(&self, response: &str, end: bool) -> Result<()>;
    async fn send_json(&self, response: &str) -> Result<()>;
    async fn send_error(&self) -> Result<()>;
    fn append_error(&mut self, error_message: ResponsiveError);
}

trait ChannelSenderUtil {
    async fn to_json(
        &self,
        request: &OpenAIRequest,
        response: &str,
        end: bool,
    ) -> Result<()>;
}

impl ChannelSender for ClientSender {
    async fn send(&self, mut buffer: Vec<u8>) -> Result<()> {
        {
            let mut last = self.last_activity.lock().await;
            *last = Instant::now();
        }

        if self.is_stream() {
            let mut vec = b"data: ".to_vec();
            vec.append(&mut buffer);
            vec.push(b'\n');
            vec.push(b'\n');
            Ok(self.inner.send(Bytes::from(vec)).await?)
        }else {
            Ok(self.inner.send(Bytes::from(buffer)).await?)
        }
    }

    async fn send_text(&self, response: &str, end: bool) -> Result<()> {
        self.to_json(&self.request, response, end).await
    }

    async fn send_json(&self, response: &str) -> Result<()> {
        Ok(self.send(response.as_bytes().to_vec()).await?)
    }

    async fn send_error(&self) -> Result<()> {
        if self.error_message.is_empty() {
            return Ok(());
        }

        let mut error_details = String::new();
        for x in self.error_message.iter() {
            error_details.push_str(
                format!("|🚝 {}|🚫 {}|🔑 {}|  \n", x.component, x.reason, x.message).as_str(),
            )
        }

        let mut suggestions = String::new();
        for x in self.error_message.iter() {
            if let Some(suggestion) = &x.suggestion {
                suggestions.push_str(format!("- 🔎 {}  \n", suggestion).as_str())
            }
        }

        let base_message = format!(
            "❗️ **发生错误！** ❗️
😿好吧，您的请求似乎发生了一点小小的问题……

🛑**错误详情：**
| **组件** | **问题** | **错误消息**           |
|---------------|--------------------|-----------------------|
{}

🔍 **建议：**
- 🔄 请仔细检查您的密钥，然后再试一次。
- 🔎 确保您的密钥与提供给您的账户的密钥一致。
{}- 📞 如果您继续遇到问题，请立即联系我们的支持团队。

🐾**GPT-Cat**始终伴您左右！",
            error_details, suggestions
        );
        self.to_json(&self.request, &base_message, false).await
    }

    fn append_error(&mut self, error_message: ResponsiveError) {
        self.error_message.push(error_message);
        self.stopped = true;
    }
}

impl ChannelSenderUtil for ClientSender {
    async fn to_json(
        &self,
        request: &OpenAIRequest,
        response: &str,
        end: bool,
    ) -> Result<()> {
        let json = if request.is_stream() {
            serde_json::to_string(&OpenAIStreamResponse::new(
                request.model.clone(),
                &response,
                end,
            ))
        } else {
            serde_json::to_string(&OpenAISyncResponse::new(
                request.model.clone(),
                &response,
                end,
            ))
        };

        match json {
            Ok(message) => {
                Ok(self.send(message.into_bytes()).await?)
            },
            Err(err) => {
                let err = format!(
                    "Success to get response, but serde json make an error: {:?}",
                    err
                );
                error!("{}", err);
                Ok(self.send(err.into_bytes()).await?)
            }
        }
    }
}
