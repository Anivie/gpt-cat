use std::ops::Deref;
use log::error;
use tokio::sync::mpsc::error::SendError;
use tokio::sync::mpsc::Sender;

use crate::data::openai_api::openai_request::OpenAIRequest;
use crate::data::openai_api::openai_stream_response::OpenAIStreamResponse;
use crate::data::openai_api::openai_sync_response::OpenAISyncResponse;

/// This struct represents an error that occurred while processing a request.
/// It contains information about the component that caused the error, the reason for the error,
/// the error message, and an optional suggestion for how to fix the error.
/// This struct is used to send error messages to the client.
/// The client will display the error message to the user.
pub struct ResponsiveError {
    pub component: String,
    pub reason: String,
    pub message: String,
    pub suggestion: Option<String>
}

pub type ClientSenderInner = Sender<String>;

/// This struct represents a channel that is used to communicate with the client.
/// # Fields
/// * `inner` - The sender that is used to send messages to the client.
/// * `error_message` - A list of error messages that have occurred while processing the request.
/// * `buffer` - A buffer that is used to store messages that are sent to the client.
/// * `request` - The request that is sending from client.
/// * `is_stream` - A flag that indicates whether the request is a stream request.
pub struct ClientSender {
    inner: ClientSenderInner,
    error_message: Vec<ResponsiveError>,
    buffer: String,

    pub request: OpenAIRequest,
}

impl ClientSender {
    pub fn new(inner: ClientSenderInner, request: OpenAIRequest) -> Self {
        Self {
            inner,
            request,
            buffer: String::new(),
            error_message: Vec::new(),
        }
    }

    pub fn is_stream(&self) -> bool {
        self.request.stream.unwrap_or(false)
    }
}

/// This trait defines the methods that are used to manage the channel buffer.
/// The channel buffer is used to store messages that are sent to the client.
pub trait ChannelBufferManager {
    fn append_buffer(&mut self, buffer: &str);
    async fn push_buffer(&self) -> Result<(), SendError<String>>;
    fn get_buffer(&self) -> &str;
}

impl ChannelBufferManager for ClientSender {
    fn append_buffer(&mut self, buffer: &str) {
        self.buffer.push_str(buffer)
    }

    async fn push_buffer(&self) -> Result<(), SendError<String>> {
        self.send_text(self.buffer.as_str(), true).await
    }

    fn get_buffer(&self) -> &str {
        self.buffer.as_str()
    }
}

impl Deref for ClientSender {
    type Target = ClientSenderInner;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

/// This trait defines the methods that are used to send messages to the client.
/// The client is responsible for displaying the messages to the user.
/// The messages can be text messages, JSON messages, or error messages.
/// The client is also responsible for handling the messages that are sent to it.
pub trait ChannelSender {
    async fn send_text(&self, response: &str, end: bool) -> Result<(), SendError<String>>;
    async fn send_error(&self) -> Result<(), SendError<String>>;
    fn has_error(&self) -> bool;
    fn append_error(&mut self, error_message: ResponsiveError);
}

trait ChannelSenderUtil {
    async fn to_json(&self, request: &OpenAIRequest, response: &str, end: bool) -> Result<(), SendError<String>>;
}

impl ChannelSender for ClientSender {
    #[inline]
    async fn send_text(&self, response: &str, end: bool) -> Result<(), SendError<String>> {
        self.to_json(&self.request, response, end).await
    }

    async fn send_error(&self) -> Result<(), SendError<String>> {
        if self.error_message.is_empty() {
            return Ok(());
        }

        let mut error_details = String::new();
        for x in self.error_message.iter() {
            error_details.push_str(format!("|🚝 {}|🚫 {}|🔑 {}|  \n", x.component, x.reason, x.message).as_str())
        }

        let mut suggestions = String::new();
        for x in self.error_message.iter() {
            if let Some(suggestion) = &x.suggestion {
                suggestions.push_str(format!("- 🔎 {}  \n", suggestion).as_str())
            }
        }

        let base_message = format!("❗️ **发生错误！** ❗️
😿好吧，您的请求似乎发生了一点小小的问题……

🛑**错误详情：**
| **组件** | **问题** | **错误消息**           |
|---------------|--------------------|-----------------------|
{}

🔍 **建议：**
- 🔄 请仔细检查您的密钥，然后再试一次。
- 🔎 确保您的密钥与提供给您的账户的密钥一致。
{}- 📞 如果您继续遇到问题，请立即联系我们的支持团队。

🐾**GPT-Cat**始终伴您左右！", error_details, suggestions);
        self.to_json(&self.request, &base_message, false).await
    }

    fn has_error(&self) -> bool {
        !self.error_message.is_empty()
    }

    fn append_error(&mut self, error_message: ResponsiveError) {
        self.error_message.push(error_message);
    }
}

impl ChannelSenderUtil for ClientSender {
    async fn to_json(&self, request: &OpenAIRequest, response: &str, end: bool) -> Result<(), SendError<String>> {
        let json = if request.is_stream() {
            serde_json::to_string(&OpenAIStreamResponse::new(request.model.clone(), &response, end))
        }else {
            serde_json::to_string(&OpenAISyncResponse::new(request.model.clone(), &response, end))
        };

        match json {
            Ok(message) => {
                self.send(message).await
            }
            Err(err) => {
                error!("Success to get response, but serde json make an error: {:?}", err);
                let mut tmp = Vec::new();
                tmp.extend(err.to_string().as_bytes());
                self.send(String::from_utf8(tmp).unwrap()).await
            }
        }
    }
}