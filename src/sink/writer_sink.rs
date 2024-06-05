/*
 * Copyright 2024 Gerald Haesendonck
 *
 *    Licensed under the Apache License, Version 2.0 (the "License");
 *    you may not use this file except in compliance with the License.
 *    You may obtain a copy of the License at
 *
 *        http://www.apache.org/licenses/LICENSE-2.0
 *
 *    Unless required by applicable law or agreed to in writing, software
 *    distributed under the License is distributed on an "AS IS" BASIS,
 *    WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 *    See the License for the specific language governing permissions and
 *    limitations under the License.
 */

use std::io::Write;
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::JoinHandle;
use crossbeam_channel::Receiver;
use log::debug;

pub struct WriterSink {
    writer_mutex: Arc<Mutex<dyn Write + Send>>
}

impl WriterSink {
    pub fn new(out: Box<dyn Write + Send>) -> &'static Self {
        debug!("Creating WriterSink...");
        let boxed = Box::new(WriterSink {
            writer_mutex: Arc::new(Mutex::new(out))
        });
        Box::leak(boxed)
    }
    
    pub fn start (&'static self, rx_chan: Receiver<Vec<String>>) -> JoinHandle<()> {
        debug!("Starting WriterSink!");
        
        let writer_clone = self.writer_mutex.clone();
        
        thread::spawn(move || {
            for data in rx_chan {
                let mut out = writer_clone.lock().unwrap();
                data.iter()
                    .skip(1)
                    .for_each(|line| {
                        out.write_all(line.as_bytes()).unwrap();
                        out.write(b"\n").unwrap();
                    });
            }
            let mut out = writer_clone.lock().unwrap();
            out.flush().unwrap();
        })
    }
}