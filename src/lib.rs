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

#[macro_use]
extern crate derive_builder;


mod plan;

mod source;
mod extension;
mod basic_functions;
mod serializer;
mod sink;
mod plan_rewriter;
mod join;
pub mod error;
pub mod mopper_options;
#[cfg(test)]
mod tests;


use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io;
use std::io::BufWriter;
use std::path::{Path, PathBuf};
use std::thread::JoinHandle;
use crossbeam_channel::{bounded, Receiver, Sender};
use log::{error, info};
use operator::{Function, IOType, Operator};
use operator::formats::ReferenceFormulation;
use crate::error::GeneralError;
use crate::extension::ExtendOperator;
use crate::join::JoinOperator;
use crate::mopper_options::{MopperOptions, MopperOptionsBuilder};
use crate::plan::PlanGraph;
use crate::plan_rewriter::rewrite;
use crate::serializer::SerializeOperator;
use crate::sink::writer_sink::WriterSink;
use crate::source::csv_file::CSVFileSource;

type VecSender = Sender<Vec<String>>;
type VecReceiver = Receiver<Vec<String>>;

/// Start mopper with the default options
pub fn start_default(algemaploom_plan: &str) -> Result<(), Box<dyn Error>> {
    let options = MopperOptionsBuilder::default().build()?;
    println!();
    start(algemaploom_plan, &options)
}

/// Start mopper with the given options
pub fn start(algemaploom_plan: &str, options: &MopperOptions) -> Result<(), Box<dyn Error>> {
    let plan_graph: PlanGraph = serde_json::from_str(algemaploom_plan).unwrap();

    // force_std_out takes precedence over force_to_file
    let to_one_target = options.force_to_std_out() || options.force_to_file().is_some();
    let reduced_plan = rewrite(&plan_graph, to_one_target);

    info!("Initializing execution engine...");
    // Create map of start node -> `send` channel 
    let mut sender_map: HashMap<usize, Vec<VecSender>> = HashMap::new();

    // Create map of end node -> `receive` channel
    let mut receiver_map: HashMap<usize, VecReceiver> = HashMap::new();

    for (id, node) in reduced_plan.iter() {
        // create channel: ONE per node (for incoming messages)
        // For now, the messages over channels are Vec<String>, where the first message contains the headers (keys)
        // and subsequent messages contain the values.
        if !node.from.is_empty() {
            let (sender, receiver) = bounded::<Vec<String>>(options.message_buffer_capacity());
            receiver_map.insert(*id, receiver);

            // now find the "from" nodes and add this node id as "sender"
            for from_node_id in &node.from {
                if let Some(senders) = sender_map.get_mut(from_node_id) {
                    senders.push(sender.clone());
                } else {
                    let mut senders: Vec<VecSender> = Vec::with_capacity(1);
                    senders.push(sender.clone());
                    sender_map.insert(*from_node_id, senders);
                }
            }
        }
    }

    // Create a vector of the join handles created by the operator threads.
    let mut join_handles: Vec<JoinHandle<(u8, String)>> = Vec::new();

    for (id, node) in reduced_plan.iter() {
        let operator = &node.operator;

        match operator {

            // Create a source
            Operator::SourceOp { config } => {
                match config.source_type {
                    IOType::File => {
                        let file_path_option = find_file(
                            &config.config["path"],
                            options.working_dir_hint()
                        );
                        if let Some(file_path) = file_path_option {
                            let reference_formulation = &config.root_iterator.reference_formulation;
                            match reference_formulation {
                                ReferenceFormulation::CSVRows => {
                                    let csv_file_source = CSVFileSource::new(file_path.to_str().unwrap().to_string(), &node.attributes, id);
                                    let senders = sender_map.remove(id).unwrap();
                                    join_handles.push(csv_file_source.start(senders));
                                },
                                _ => {}
                            }
                        } else {
                            let msg = format!("File not found:  {}", &config.config["path"]);
                            error!("{msg}");
                            return Err(Box::new(GeneralError::from_msg(msg)));
                        }
                    }
                    _ => {}
                }
            },

            // Create an Extension operator
            Operator::ExtendOp { config } => {
                let extend_pairs: &HashMap<String, Function> = &config.extend_pairs;
                let extend_operator = ExtendOperator::new(extend_pairs, id, &node.join_alias);
                let senders = sender_map.remove(id).unwrap();
                let receiver = receiver_map.remove(id).unwrap();
                join_handles.push(extend_operator.start(receiver, senders));
            },

            // Create a Serialize operator
            Operator::SerializerOp { config } => {
                let serialize_operator = SerializeOperator::new(config, id);
                let senders = sender_map.remove(id).unwrap();
                let receiver = receiver_map.remove(id).unwrap();
                join_handles.push(serialize_operator.start(receiver, senders));
            },

            // Create a Target operator
            Operator::TargetOp { config } => {
                let receiver = receiver_map.remove(id).unwrap();
                
                // Forcing output to standard out or to file overrides the target settings
                if options.force_to_std_out() {
                    let stdout = io::stdout();
                    let writer_sink = WriterSink::new(Box::new(stdout), id);
                    join_handles.push(writer_sink.start(receiver.clone())); // is this a good idea?
                } else if let Some(file_path) = options.force_to_file() {
                    let file = File::create(file_path).unwrap();
                    let file_out = BufWriter::new(file);
                    let writer_sink = WriterSink::new(Box::new(file_out), id);
                    join_handles.push(writer_sink.start(receiver.clone())); // is this a good idea?
                } else {

                    // TODO: do something with config, just create a std out sink for now
                    match config.target_type {
                        IOType::StdOut => {
                            let stdout = io::stdout();
                            let writer_sink = WriterSink::new(Box::new(stdout), id);
                            join_handles.push(writer_sink.start(receiver));
                        },
                        _ => {
                            error!("Target type {:?} not implemented yet!", config.target_type);
                            error!("You can force all output to be written to standard out or a file.");
                            todo!();
                        }
                    }
                }
            },

            Operator::JoinOp {config} => {
                // find left and right node ids
                let left = &node.from[0];
                let right = &node.from[1];

                let join_operator = JoinOperator::new(config, left, right, id);
                let senders = sender_map.remove(id).unwrap();
                let receiver = receiver_map.remove(id).unwrap();
                join_handles.push(join_operator.start(receiver, senders));
            },

            _ => todo!()
        }

    }

    info!("Up and running!");

    let mut errors: Vec<(u8, String)> = Vec::new();
    for join_handle in join_handles {
        let (err_code, msg) = join_handle.join().unwrap();
        if err_code > 0 {
            error!("{msg}");
            errors.push((err_code, msg));
        }
    }
    
    if errors.is_empty() {
        info!("Done!");
        Ok(())
    } else {
        Err(Box::new(GeneralError::new(errors)))
    }
    
    
}

fn find_file(file: &str, working_dir_hint: &Option<String>) -> Option<PathBuf> {
    let file_path = Path::new(file);
    
    match file_path.exists() { 
        true => Some(file_path.to_path_buf()),
        false => {
            if let Some(working_dir) = working_dir_hint {
                let working_dir_path = Path::new(working_dir);
                let new_path = working_dir_path.join(file);
                match new_path.exists() { 
                    true => Some(new_path),
                    false => None
                }
            } else {
                None
            }
        }
    }
    
}