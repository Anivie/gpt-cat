use rayon::prelude::*;
use crate::http::client::util::sse::sse_processor::SSEProcessor;

#[derive(Default)]
pub struct RayonJsonProcessor {
    inner: Vec<u8>,
}

impl SSEProcessor for RayonJsonProcessor {
    fn process<'a>(&mut self, target: &'a [u8]) -> (Vec<&'a [u8]>, Option<Vec<u8>>) {
        let mut lines: Vec<&[u8]> = Vec::new();
        let mut first_line = None;

        let tail_index = target
            .par_iter()
            .enumerate()
            .filter(|(_, &value)| value == b'\n')
            .map(|x| x.0 + 2)
            .collect::<Vec<_>>();

        let tail_index = tail_index
            .par_iter()
            .step_by(2)
            .collect::<Vec<_>>();

        let jump_first = if let Some(&tail_first) = tail_index.first() &&
            !self.inner.is_empty()
        {
            self.inner.extend_from_slice(&target[..*tail_first]);
            first_line.replace(self.inner.clone());
            self.inner.clear();
            true
        } else {
            false
        };

        for (index, &value) in tail_index.iter().enumerate() {
            if index == 0 && jump_first { continue; }

            if index == 0 {
                lines.push(&target[6 .. *value])
            }else {
                lines.push(&target[*tail_index[index - 1] + 6 .. *value]);
            }
        }
        
        if !target.ends_with(&[b'\n', b'\n']) {
            self.inner.extend(&target[*tail_index[tail_index.len() - 1] + 6 ..]);
        }

        if let Some(&last) = lines.last() {
            if last.starts_with(b"[DONE]") {
                lines.remove(lines.len() - 1);
            }
        }

        (lines, first_line)
    }

    fn process_return_label<'a>(&mut self, target: &'a [u8]) -> (Vec<(&'a [u8], &'a [u8])>, Option<Vec<u8>>) {
        let mut lines: Vec<(&[u8], &[u8])> = Vec::new();
        let mut first_line = None;

        let tail_index = target
            .par_iter()
            .enumerate()
            .filter(|(_, &value)| value == b'\n')
            .map(|x| x.0 + 2)
            .collect::<Vec<_>>();

        let tail_index = tail_index
            .par_iter()
            .step_by(2)
            .collect::<Vec<_>>();

        let jump_first = if let Some(&tail_first) = tail_index.first() &&
            !self.inner.is_empty()
        {
            self.inner.extend_from_slice(&target[..*tail_first]);
            first_line.replace(self.inner.clone());
            self.inner.clear();
            true
        } else {
            false
        };

        for (index, &value) in tail_index.iter().enumerate() {
            if index == 0 && jump_first {
                continue;
            }

            if index == 0 {
                if let Some(position) = target[0..*value].par_windows(2).position_any(|x| x == b": ") {
                    lines.push((&target[.. position], &target[position + 2 .. *value]));
                }
                continue;
            }

            if let Some(position) = target[*tail_index[index - 1] .. *value].par_windows(2).position_any(|x| x == b": ") {
                lines.push((&target[*tail_index[index - 1] .. *tail_index[index - 1] + position], &target[*tail_index[index - 1] + position + 2 .. *value]));
            }
        }

        if !target.ends_with(&[b'\n', b'\n']) {
            self.inner.extend(&target[*tail_index[tail_index.len() - 1] ..]);
        }

        if let Some(&last) = lines.last() {
            if last.1.starts_with(b"[DONE]") {
                lines.remove(lines.len() - 1);
            }
        }

        (lines, first_line)
    }
}