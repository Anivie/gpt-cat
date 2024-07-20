pub trait SSEProcessor {
    /// Process the stream and return the split and the first response
    /// # Arguments
    /// * `target` - The target stream
    /// # Returns
    /// * A tuple contains the **split** and the **first response** in
    /// previous stream
    fn process<'a>(&mut self, target: &'a [u8]) -> (Vec<&'a [u8]>, Option<Vec<u8>>);

    /// Process the stream and return the label, split and the first response
    /// # Arguments
    /// * `target` - The target stream
    /// # Returns
    /// * A tuple contains the **label**, **split** and the **first response** in
    /// previous stream
    #[allow(dead_code)]
    fn process_return_label<'a>(
        &mut self,
        target: &'a [u8],
    ) -> (
        Vec<(Option<&'a [u8]>, &'a [u8])>,
        Option<(Option<Vec<u8>>, Vec<u8>)>,
    );
}
