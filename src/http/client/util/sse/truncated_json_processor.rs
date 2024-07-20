use rayon::prelude::*;

use crate::http::client::util::sse::sse_processor::SSEProcessor;

#[derive(Default)]
pub struct TruncatedJsonProcessor {
    inner: Vec<u8>,

    left: u8,
    right: u8,

    double_quote: bool,
}

impl SSEProcessor for TruncatedJsonProcessor {
    fn process<'a>(&mut self, target: &'a [u8]) -> (Vec<&'a [u8]>, Option<Vec<u8>>) {
        let mut back = Vec::new();
        let mut first = None;
        let mut last_index = 0;

        for (index, &value) in target.iter().enumerate() {
            match value {
                b'"' => {
                    if index > 0 && target[index - 1] != b'\\' {
                        self.double_quote = !self.double_quote;
                        continue;
                    }

                    let backslash_count = target[..index]
                        .iter()
                        .rev()
                        .chain(self.inner.iter().rev())
                        .take_while(|&&x| x == b'\\')
                        .count();

                    if backslash_count % 2 == 0 {
                        self.double_quote = !self.double_quote;
                    }

                    continue;
                }
                b'{' => {
                    if self.double_quote {
                        continue;
                    }
                    self.left += 1;
                    continue;
                }
                b'}' => {
                    if self.double_quote {
                        continue;
                    }
                    self.right += 1;
                }
                _ => {
                    if !self.double_quote && self.left == 0 && self.right == 0 {
                        last_index += 1;
                    }
                    continue;
                }
            }

            if self.left != self.right {
                continue;
            }

            if self.inner.is_empty() {
                back.push(&target[last_index..index + 1]);
            } else {
                first.replace(
                    self.inner
                        .iter()
                        .chain(&target[..index + 1])
                        .copied()
                        .collect(),
                );

                self.inner.clear();
            }

            self.left = 0;
            self.right = 0;
            last_index = index + 1;
        }

        if self.left != 0 {
            self.inner.append(&mut target[last_index..].to_vec());
        }

        (back, first)
    }

    fn process_return_label<'a>(
        &mut self,
        target: &'a [u8],
    ) -> (
        Vec<(Option<&'a [u8]>, &'a [u8])>,
        Option<(Option<Vec<u8>>, Vec<u8>)>,
    ) {
        let mut label = Vec::new();
        let mut back = Vec::new();
        let mut first = None;
        let mut last_index = 0;

        for (index, &value) in target.iter().enumerate() {
            match value {
                b'"' => {
                    if index > 0 && target[index - 1] != b'\\' {
                        self.double_quote = !self.double_quote;
                        continue;
                    }

                    let backslash_count = target[..index]
                        .iter()
                        .rev()
                        .chain(self.inner.iter().rev())
                        .take_while(|&&x| x == b'\\')
                        .count();

                    if backslash_count % 2 == 0 {
                        self.double_quote = !self.double_quote;
                    }

                    continue;
                }
                b'{' => {
                    if self.double_quote {
                        continue;
                    }

                    if self.left == 0 && self.right == 0 {
                        while target[last_index] == b'\n' {
                            last_index += 1;
                        }
                        label.push(&target[last_index..index - 2]);
                        last_index = index;
                    }

                    self.left += 1;
                    continue;
                }
                b'}' => {
                    if self.double_quote {
                        continue;
                    }
                    self.right += 1;
                }
                _ => continue,
            }

            if self.left != self.right {
                continue;
            }

            if self.inner.is_empty() {
                back.push(&target[last_index..index + 1]);
            } else {
                first.replace(
                    //     ↓ bad way, wait for the replace
                    (
                        None,
                        self.inner
                            .iter()
                            .chain(&target[..index + 1])
                            .cloned()
                            .collect(),
                    ),
                );

                self.inner.clear();
            }

            self.left = 0;
            self.right = 0;
            last_index = index + 1;
        }

        if self.left != 0 {
            self.inner.append(&mut target[last_index..].to_vec());
        }

        let back = back
            .par_iter()
            .zip(label.par_iter())
            .map(|(&x, &y)| (Some(x), y))
            .collect();

        (back, first)
    }
}
