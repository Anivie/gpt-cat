use std::convert::Infallible;

use axum::http::HeaderMap;
use axum::response::sse::Event;
use axum::response::{IntoResponse, Response, Sse};
use futures::Stream;

/// The enum for the response data
/// The response data can be SSE or a json, so
/// we can respond either SSE or a json when
/// client request the stream or block mode.
pub enum ResponseData<T: Stream<Item = Result<Event, Infallible>> + Send + 'static> {
    Sse((HeaderMap, Sse<T>)),
    Json((HeaderMap, String)),
}

/// Implement the IntoResponse trait for the ResponseData
impl<T: Stream<Item = Result<Event, Infallible>> + Send + 'static> IntoResponse
    for ResponseData<T>
{
    fn into_response(self) -> Response {
        match self {
            ResponseData::Sse(sse) => sse.into_response(),
            ResponseData::Json(json) => json.into_response(),
        }
    }
}
