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

/*#[test]
fn test_rayon_json_processor_with_label() {
    use crate::data::openai_api::openai_stream_response::OpenAIStreamResponse;

    let mut processor = RayonJsonProcessor::default();
    let target1 = include_bytes!("../../../../../temp_lite1.data");
    let target2 = include_bytes!("../../../../../temp_lite2.data");
    let (lines, first_line) = processor.process_return_label(target1);

    if let Some((label, value)) = first_line {
        // println!("first line's label: {:?}, value: {:?}", String::from_utf8_lossy(label.as_ref().unwrap()), String::from_utf8(value.clone()));
        println!("{:?}", serde_json::from_slice::<OpenAIStreamResponse>(value.as_slice()).unwrap().choices.first().unwrap().delta.content);
    }

    for (label, value) in lines {
        // println!("label: {:?}, value: {:?}", String::from_utf8_lossy(label.as_ref().unwrap()), String::from_utf8_lossy(value));
        match serde_json::from_slice::<OpenAIStreamResponse>(value) {
            Ok(res) => {
                println!("{:?}", res.choices.first().unwrap().delta.content);
            }
            Err(err) => {
                eprintln!("Error: {:?}, {:?}", err, String::from_utf8_lossy(value));
            }
        }
    }

    let (lines, first_line) = processor.process_return_label(target2);

    if let Some((label, value)) = first_line {
        // println!("first line's label: {:?}, value: {:?}", String::from_utf8_lossy(label.as_ref().unwrap()), String::from_utf8(value.clone()));
        println!("{:?}", serde_json::from_slice::<OpenAIStreamResponse>(value.as_slice()).unwrap().choices.first().unwrap().delta.content);
    }

    for (label, value) in lines {
        // println!("label: {:?}, value: {:?}", String::from_utf8_lossy(label.as_ref().unwrap()), String::from_utf8_lossy(value));
        // println!("value: {:?}", String::from_utf8_lossy(value));
        // println!("{:?}", serde_json::from_slice::<OpenAIStreamResponse>(value).unwrap().choices.first().unwrap().delta.content);
        match serde_json::from_slice::<OpenAIStreamResponse>(value) {
            Ok(res) => {
                println!("{:?}", res.choices.first().unwrap().delta.content);
            }
            Err(err) => {
                eprintln!("Error: {:?}, {:?}", err, String::from_utf8_lossy(value));
            }
        }
    }
}

#[test]
fn test_rayon_json_processor() {
    use crate::data::openai_api::openai_stream_response::OpenAIStreamResponse;

    let mut processor = RayonJsonProcessor::default();
    let mut origin = Vec::with_capacity(953);
    for index in 0..953 {
        let mut file = File::open(format!("/home/gpt-cat-custom/tmp_data/temp_data_{}", index)).unwrap();
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).unwrap();
        origin.push(buffer);
    }

    for target in origin {
        let (lines, first_line) = processor.process(target.as_slice());

        if let Some(first_line) = first_line {
            match serde_json::from_slice::<OpenAIStreamResponse>(first_line.as_slice()) {
                Ok(_) => {
                    // println!("Success: {:?}", String::from_utf8_lossy(first_line.as_slice()));
                },
                Err(err) => {
                    println!("Error: {:?}, origin text: {}.", err, String::from_utf8_lossy(first_line.as_slice()));
                }
            }
        }

        lines.iter().for_each(|line| {
            match serde_json::from_slice::<OpenAIStreamResponse>(line) {
                Ok(_) => {
                    // println!("Success: {:?}", String::from_utf8_lossy(line));
                }
                Err(err) => {
                    println!("Error: {:?}, origin text: {}.", err, String::from_utf8_lossy(line));
                }
            }
        });
    }

}

#[test]
fn random_test_rayon_json_processor() {
    use crate::data::openai_api::openai_stream_response::OpenAIStreamResponse;

    let origin = include_bytes!("../../../../../temp_full.data");
    let origin = Arc::new(origin);

    use rand::Rng;

    fn random_split(slice: &[u8]) -> Vec<&[u8]> {
        let mut rng = rand::thread_rng();
        let len = slice.len();

        if len == 0 {
            return vec![];
        }

        let mut split_points = vec![0];
        let num_splits = rng.gen_range(1..=len);

        for _ in 1..num_splits {
            let split_point = rng.gen_range(1..len);
            if !split_points.contains(&split_point) {
                split_points.push(split_point);
            }
        }

        split_points.push(len);
        split_points.sort_unstable();

        split_points.windows(2)
            .map(|w| &slice[w[0]..w[1]])
            .collect()
    }

    let success_count = Arc::new(Mutex::new(0));
    let error_count = Arc::new(Mutex::new(0));
    let concurrency = 1000;

/*    let vec = random_split(origin.as_slice());
    println!("{}", vec.len());
    for x in vec {
        println!("{:?}", String::from_utf8_lossy(x));
    }
*/
    //随机分割原文并进行处理，不报错即为成功
    for _ in 0..concurrency {
        let target = origin.clone();
        let success_count = success_count.clone();
        let error_count = error_count.clone();
        thread::spawn(move || {
            let mut processor = RayonJsonProcessor::default();
            let vec = random_split(target.as_slice());

            for &x in vec.iter() {
                let (lines, first_line) = processor.process(x);

                if let Some(first_line) = first_line {
                    match serde_json::from_slice::<OpenAIStreamResponse>(first_line.as_slice()) {
                        Ok(_) => {
                            let mut result = success_count.lock().unwrap();
                            *result = *result + 1;
                        },
                        Err(err) => {
                            if *error_count.lock().unwrap() == 0 {
                                println!("File!");
                                for (index, &x) in vec.iter().enumerate() {
                                    let mut file = std::fs::OpenOptions::new()
                                        .write(true)
                                        .create(true)
                                        .append(true)
                                        .open(format!("/home/gpt-cat-custom/tmp_data/temp_data_{}", index))
                                        .unwrap();
                                    file.write_all(x).unwrap();
                                }
                            }
                            let mut result = error_count.lock().unwrap();
                            *result = *result + 1;
                            panic!("Error: {:?}, origin text: {}.", err, String::from_utf8_lossy(first_line.as_slice()));
                        }
                    }
                }

                lines.iter().for_each(|line| {
                    match serde_json::from_slice::<OpenAIStreamResponse>(line) {
                        Ok(_) => {
                            let mut result = success_count.lock().unwrap();
                            *result = *result + 1;
                        }
                        Err(err) => {
                            let mut result = error_count.lock().unwrap();
                            *result = *result + 1;
                            panic!("Error: {:?}, origin text: {}.", err, String::from_utf8_lossy(line));
                        }
                    }
                });
            }
        });
    }

    //等待所有线程结束
    while *success_count.lock().unwrap() + *error_count.lock().unwrap() < concurrency {
        thread::sleep(std::time::Duration::from_secs(1));
    }

    //打印结果
    println!("Success count: {}, Error count: {}", *success_count.lock().unwrap(), *error_count.lock().unwrap());
}*/