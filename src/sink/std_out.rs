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

use std::{io, thread};
use std::io::Write;
use std::thread::JoinHandle;
use crossbeam_channel::Receiver;
use log::debug;

pub struct StdOutSink {}

impl StdOutSink {
    pub fn new() -> &'static Self {
        debug!("Creating StdOutSink...");
        let boxed = Box::new(StdOutSink{});
        Box::leak(boxed)
    }
    
    pub fn start (&'static self, rx_chan: Receiver<Vec<String>>) -> JoinHandle<()> {
        debug!("Starting StdOutSink!");
        thread::spawn(move || {
            let std_out = io::stdout();
            for data in rx_chan {
                let mut out = std_out.lock();
                data.iter()
                    .skip(1) // ignore node id
                    .for_each(|line| {
                        out.write_all(line.as_bytes()).unwrap();
                        out.write(b"\n").unwrap();
                    });
            }
        })
    }
}