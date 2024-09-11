use futures::Stream;
use ntex::http::header::ContentEncoding;
use ntex::http::Response;
use ntex::util::Bytes;
use ntex::web::{BodyEncoding, Error, HttpResponse};
use std::ops::Deref;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::sync::mpsc::Receiver;

struct Client(Receiver<Bytes>);

impl Stream for Client {
    type Item = Result<Bytes, Error>;

    fn poll_next(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        match Pin::new(&mut self.0).poll_recv(cx) {
            Poll::Ready(Some(v)) => Poll::Ready(Some(Ok(v))),
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}

pub(super) async fn end(
    mut receiver: Receiver<Bytes>,
    is_stream: bool,
) -> Response {
    if is_stream {
        HttpResponse::Ok()
            .encoding(ContentEncoding::Identity)
            .content_type("text/event-stream")
            .header("Cache-Control", "no-cache")
            .keep_alive()
            .streaming(Client(receiver))
    }else {
        let mut back = String::default();
        while let Some(message) = receiver.recv().await {
            back = String::from_utf8_lossy(message.deref()).to_string();
        }

        HttpResponse::Ok()
            .content_type("application/json")
            .encoding(ContentEncoding::Identity)
            .header("Cache-Control", "no-cache, must-revalidate")
            .body(back)
    }
}