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

use std::collections::HashMap;
use std::thread;
use std::thread::JoinHandle;
use crossbeam_channel::{Receiver, Sender};
use log::{debug, error};
use operator::formats::DataFormat;
use operator::Serializer;

pub struct SerializeOperator {
    template_string_parts: Vec<(bool, String)>,
    node_id: String
}
impl SerializeOperator {
    pub fn new(config: &Serializer, node_id: &usize) -> &'static Self {
        debug!("Initializing Serialize operator {node_id}.");
        if config.format != DataFormat::NQuads && config.format != DataFormat::NTriples {
            error!("Serializer: only NQuads / NTriples supported at the moment!");
            todo!()
        }

        let template = config.template.as_str();
        let boxed = Box::new(SerializeOperator{
            template_string_parts: create_template_template_string_parts(template),
            node_id: node_id.to_string()
        });
        Box::leak(boxed)
    }
    
    pub fn start(&'static self, rx_chan: Receiver<Vec<String>>, tx_channels: Vec<Sender<Vec<String>>>) -> JoinHandle<()> {
        debug!("Starting Serialize {}!", self.node_id);
        
        thread::spawn(move || {
            
            // Get the variable names ("headers") in the order they will arrive
            let mut iter = rx_chan.iter();
            let variable_names_option = iter.next();
            if variable_names_option.is_some() {
                let variable_names = &variable_names_option.unwrap()[1..];

                // Get the data types of the variables
                let data_types_option = iter.next();
                if data_types_option.is_some() {
                    let data_types = &data_types_option.unwrap()[1..];

                    for values in iter {
                        let mut variable_name_to_value_map = HashMap::with_capacity(variable_names.len());
                        for (index, value) in values.iter().skip(1).enumerate() {   // skip node id
                            let variable_name = &variable_names[index];
                            let data_type = &data_types[index];
                            variable_name_to_value_map.insert(variable_name, (value, data_type));
                        }

                        let mut result_str = String::new();

                        self.template_string_parts.iter()
                            .for_each(|(is_variable, part)| {
                                if *is_variable {
                                    let (value, data_type) = variable_name_to_value_map[part];
                                    let data_type_str = data_type.to_string();  // TODO: this to_string and later as_str must be avoided!
                                    
                                    // TODO: this formatting part should be a separate serialization treat & implementation.
                                    //       Now it just formats N-Triples / N-quads in a hardcoded way. 
                                    let value_str = match data_type_str.as_str() {
                                        "str" => {
                                            data_type_str
                                        },
                                        "iri" => {
                                            // The format! macro would do fine, but is slower
                                            let mut str = String::new();
                                            str.push('<');
                                            str.push_str(value);
                                            str.push('>');
                                            str
                                        },
                                        "lit" => {
                                            // The format! macro would do fine, but is slower
                                            let mut str = String::new();
                                            str.push('"');
                                            str.push_str(value);
                                            str.push('"');
                                            str
                                        },
                                        "blank" => {
                                            let mut str = String::from("_:");
                                            str.push_str(value);
                                            str
                                        }
                                        _ => {
                                            todo!()
                                        }
                                    };
                                    result_str.push_str(&value_str);
                                } else {
                                    result_str.push_str(part);
                                }
                            });
                        tx_channels.iter()
                            .for_each(|tx_chan| tx_chan.send(vec![self.node_id.clone(), result_str.clone()]).unwrap());
                    }
                }
            }
            
        })
    }
}

//// Some helper functions
fn create_template_template_string_parts(template: &str) -> Vec<(bool, String)> {
    let mut template_string_parts: Vec<(bool, String)> = Vec::with_capacity(2);
    let mut current_str = String::new();
    let mut is_variable_name = false;     // TODO: replace by counter to deal with nested '{'

    // TODO: better parsing, error handling, ...
    template.chars().for_each(|c| {
        match c {
            '?' => {
                if !is_variable_name {
                    if !current_str.is_empty() {
                        template_string_parts.push((false, current_str.to_string()));
                        current_str.clear();
                    }
                    is_variable_name = true;
                }
            },
            ' ' => {
                if is_variable_name {
                    if !current_str.is_empty() {
                        template_string_parts.push((true, current_str.to_string()));
                        current_str.clear();
                    }
                    is_variable_name = false;
                }
                current_str.push(' ');
            }
            _ => {
                current_str.push(c);
            }
        }
    });

    // add last part, if any
    if !current_str.is_empty() {
        template_string_parts.push((false, current_str.to_string()));
    }
    
    template_string_parts
}