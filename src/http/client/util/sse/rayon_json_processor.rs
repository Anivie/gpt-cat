use rayon::prelude::*;

use crate::http::client::util::sse::sse_processor::SSEProcessor;

#[derive(Default)]
pub struct RayonJsonProcessor {
    inner: Vec<u8>,
}

impl SSEProcessor for RayonJsonProcessor {
    fn process<'a>(&mut self, target: &'a [u8]) -> (Vec<&'a [u8]>, Option<Vec<u8>>) {
        let positions = target.par_windows(2).positions(|x| x == b"\n\n").collect::<Vec<_>>();
        if positions.is_empty() {
            if !self.inner.is_empty() {
                self.inner.extend_from_slice(target);
                return (vec![], None);
            }
            return (vec![target], None);
        }

        let (target, positions, first_line) = if self.inner.is_empty() {
            (target, positions, None)
        } else {
            let position = positions[0];

            self.inner.extend_from_slice(&target[..position]);
            let first = self.inner
                .iter()
                .position(|x| *x == b':')
                .map_or(self.inner.clone(), |x| {
                    self.inner[x + 1..].to_vec()
                });
            self.inner.clear();

            let positions = positions.par_iter()
                .skip(1)
                .map(|x| *x - position - 2)
                .collect();

            (&target[position + 2..], positions, Some(first))
        };

        let target = if target.ends_with(&[b'\n', b'\n']) {
            target
        } else {
            if let Some(position) = positions.last() {
                self.inner.extend_from_slice(&target[*position + 2..]);
                &target[.. *position]
            }else {
                target
            }
        };

        let lines = {
            let mut lines = Vec::new();

            let mut last_position = 0;
            for position in positions {
                lines.push(&target[last_position..position]);
                last_position = position + 2;
            }

            lines
        };

        let mut lines = lines.par_iter()
            .map(|&x| {
                x.par_split(|x| *x == b'\n')
                    .map(|x| {
                        x.iter().position(|x| *x == b':').map_or((None, x), |position| {
                            (Some(&x[..position]), &x[position + 1..])
                        })
                    })
                    .filter(|&(label, _)| {
                        label.is_some() && label.unwrap() == b"data"
                    })
                    .map(|x| x.1)
                    .collect::<Vec<_>>()
            })
            .flat_map(|v| v.into_par_iter())
            .collect::<Vec<_>>();

        if let Some(last) = lines.last() &&
            last.ends_with(b"[DONE]")
        {
            lines.pop();
        }

        (lines, first_line)
    }

    fn process_return_label<'a>(&mut self, target: &'a [u8]) -> (Vec<(Option<&'a [u8]>, &'a [u8])>, Option<(Option<Vec<u8>>, Vec<u8>)>) {
        let positions = target.par_windows(2).positions(|x| x == b"\n\n").collect::<Vec<_>>();
        if positions.is_empty() {
            if !self.inner.is_empty() {
                self.inner.extend_from_slice(target);
                return (vec![(None, &target[0..0])], None);
            }
            return (vec![(None, target)], None);
        }

        let (target, positions, first_line) = if self.inner.is_empty() {
            (target, positions, None)
        } else {
            let position = positions[0];

            self.inner.extend_from_slice(&target[..position]);
            let first = self.inner
                .iter()
                .position(|x| *x == b':')
                .map_or((None, self.inner.clone()), |position| {
                    (Some(self.inner[..position].to_vec()), self.inner[position + 1..].to_vec())
                });
            self.inner.clear();

            let positions = positions.par_iter()
                .skip(1)
                .map(|x| *x - position - 2)
                .collect();

            (&target[position + 2..], positions, Some(first))
        };

        let target = if target.ends_with(&[b'\n', b'\n']) {
            target
        } else {
            if let Some(position) = positions.last() {
                self.inner.extend_from_slice(&target[*position + 2..]);
                &target[.. *position]
            }else {
                target
            }
        };

        let lines = {
            let mut lines = Vec::new();

            let mut last_position = 0;
            for position in positions {
                lines.push(&target[last_position..position]);
                last_position = position + 2;
            }

            lines
        };

        let mut lines = lines.par_iter()
            .map(|&x| {
                x.par_split(|x| *x == b'\n')
                    .map(|x| {
                        x.iter().position(|x| *x == b':').map_or((None, x), |position| {
                            (Some(&x[..position]), &x[position + 1..])
                        })
                    })
                    .collect::<Vec<_>>()
            })
            .flat_map(|v| v.into_par_iter())
            .collect::<Vec<_>>();

        if let Some(&(_, last)) = lines.last() &&
            last.ends_with(b"[DONE]")
        {
            lines.pop();
        }

        (lines, first_line)
    }
}