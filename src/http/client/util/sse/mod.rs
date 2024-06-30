/// Custom SSE processor, which can extract the truncated json from the stream
pub mod sse_processor;


/// Description for **truncated json processor**:
/// This project was originally designed to handle some additional tasks, in those tasks, the SSE stream encountered
/// is not always split according to the standard `\n\n`, so a custom processor is needed to handle this situation.
/// But later it was found that this processor can also work normally when processing standard SSE streams,
/// and the performance is also good, so it has been used all the time.
/// This processor is a little bit slower than the rayon json processor, but it has a higher tolerance and can handle
/// some non-standard streams.
pub mod truncated_json_processor;

/// Description for **rayon json processor**:
/// The rayon json processor is a high-performance processor that can handle standard SSE streams.
/// It is faster than the truncated json processor, but it is not as tolerant as the truncated json processor,
/// and it may not work properly when processing non-standard streams.
pub mod rayon_json_processor;