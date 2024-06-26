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

use std::collections::HashSet;
use std::fs::File;
use std::io::BufReader;
use std::iter::once;
use std::ops::Index;
use std::thread;
use std::thread::JoinHandle;
use crossbeam_channel::Sender;
use log::{debug, error, warn};

pub struct CSVFileSource {
    file_path: String,
    // TODO: delimiter etc
    attributes: Vec<String>,     // TODO: remove Option part?
    node_id: String
}

impl CSVFileSource {

    pub fn new(file_path: String, attributes: &Option<HashSet<String>>, node_id: &usize) -> &'static Self {
        debug!("Creating CSVFileSource...");
        let attributes_vec: Vec<String> = match  attributes { 
            Some(attr) => attr.iter().map(|value| value.to_string()).collect(),
            None => Vec::new()
        };
        let boxed = Box::new(
            CSVFileSource{
                file_path,
                attributes: attributes_vec,
                node_id: node_id.to_string()
            },
        );
        Box::leak(boxed)
    }

    pub fn start(&'static self, tx_channels: Vec<Sender<Vec<String>>>) -> JoinHandle<(u8, String)> {
        thread::Builder::new()
            .name(format!("CSVFileSource {}", self.node_id))
            .spawn(move || {
            debug!("Starting CSVFileSource!");
                        
            let file_res = File::open(self.file_path.clone());
            if let Err(file_err) = file_res {
                let msg = format!("Cannot open {}: {}", self.file_path, file_err.to_string());
                error!("{msg}");
                return (1u8, msg)
            }
                //.expect(format!("File not found: {}", self.file_path).as_str());
            let br = BufReader::new(file_res.unwrap());
            let mut rdr = 
                csv::ReaderBuilder::new()
                    .has_headers(false)
                    .from_reader(br);
            
            let mut attribute_indices: Vec<usize> = Vec::with_capacity(self.attributes.len());
            
            // First map the headers / field names to an index
            let mut iter = rdr.records();
            let headers_result = iter.next();
            if headers_result.is_some() {
                let headers = headers_result.unwrap().unwrap();
                for attribute in &self.attributes {
                    let index = headers.iter().position(|r| r == attribute);
                    match index {
                        Some(i) => {
                            attribute_indices.push(i);
                        },
                        None => {
                            warn!("WARNING: no field found with name {}", attribute);
                        }
                    }
                }
                
                // prepend node_id to attributes
                let node_id_plus_headers: Vec<String> = once(&self.node_id)
                    .chain(self.attributes.iter())
                    .map(|data| data.to_string())
                    .collect();
                
                tx_channels.iter()
                    .for_each(|tx_chan| tx_chan.send(node_id_plus_headers.clone()).unwrap());
            }
            
            for result in iter {
                let record = result.unwrap();
                let node_id_plus_data: Vec<String> = once(&self.node_id)
                    .map(|data| data.to_string())
                    .chain(
                        attribute_indices.iter()
                            .map(|index| String::from(&record.index(*index).to_string()))
                    )
                    .collect();
                tx_channels.iter()
                    .for_each(|tx_chan| tx_chan.send(node_id_plus_data.clone()).unwrap());
            }

            (0, String::new())
        }).unwrap()
    }
}