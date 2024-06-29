/// Custom SSE processor, which can extract the truncated json from the stream
/// Q: Why we need this?
/// A: This project was originally designed to handle some additional tasks, in those tasks, the SSE stream encountered
///    is not always split according to the standard `\n\n`, so a custom processor is needed to handle this situation.
///    But later it was found that this processor can also work normally when processing standard SSE streams,
///    and the performance is also good, so it has been used all the time.
pub mod truncated_json_processor;
pub mod sse_processor;
pub mod rayon_json_processor;