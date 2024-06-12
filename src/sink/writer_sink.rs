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
    writer_mutex: Arc<Mutex<dyn Write + Send>>,
    node_id: String
}

impl WriterSink {
    pub fn new(out: Box<dyn Write + Send>, node_id: &usize) -> &'static Self {
        debug!("Creating WriterSink {node_id}...");
        let boxed = Box::new(WriterSink {
            writer_mutex: Arc::new(Mutex::new(out)),
            node_id: node_id.to_string()
        });
        Box::leak(boxed)
    }
    
    pub fn start (&'static self, rx_chan: Receiver<Vec<String>>) -> JoinHandle<(u8, String)> {
        debug!("Starting WriterSink {}", self.node_id);
        
        let writer_clone = self.writer_mutex.clone();
        
        thread::spawn(move || {
            for data in rx_chan {
                let mut data_to_write = data[1..].join("\n");
                data_to_write.push('\n');
                let mut out = writer_clone.lock().unwrap();
                out.write_all(&data_to_write.as_bytes()).unwrap()
            }
            let mut out = writer_clone.lock().unwrap();
            out.flush().unwrap();

            (0, String::new())
        })
    }
}