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

mod plan;
pub mod source;
mod extension;
mod basic_functions;
mod serializer;
mod sink;
mod plan_rewriter;
mod join;

use std::collections::HashMap;
use std::fs;
use std::thread::JoinHandle;
use argh::FromArgs;
use crossbeam_channel::{bounded, Receiver, Sender};
use log::info;
use operator::{Function, IOType, Operator};
use operator::formats::ReferenceFormulation;
use stderrlog::LogLevelNum;
use crate::extension::ExtendOperator;
use crate::join::JoinOperator;
use crate::plan::PlanGraph;
use crate::plan_rewriter::rewrite;
use crate::serializer::SerializeOperator;
use crate::sink::std_out::StdOutSink;
use crate::source::csv_file::CSVFileSource;

type VecSender = Sender<Vec<String>>;
type VecReceiver = Receiver<Vec<String>>;

/*
For now, the messages over channels are Vec<String>, where the first message contains the headers (keys)
and subsequent messages contain the values.
 */

/// Execute an AlgeMapLoom mapping plan
#[derive(FromArgs)]
struct Args {
    /// the path to the AlgeMapLoom mapping plan (JSON)
    #[argh(option, short = 'm')]
    mapping_file: String,

    /// increase log level
    #[argh(switch)]
    v: bool,

    /// be quiet; no logging
    #[argh(switch)]
    q: bool,
}

fn main() {
    let args: Args = argh::from_env();
    
    let log_level = match args.v {
        true => LogLevelNum::Debug,
        false => LogLevelNum::Info
    };
    
    // init logging
    stderrlog::new()
        .module(module_path!())
        .quiet(args.q)
        .timestamp(stderrlog::Timestamp::Second)
        .verbosity(log_level)
        .init()
        .unwrap();
    
    // Read the execution plan
    info!("Reading mapping plan...");
    let path_to_plan_serialisation = &args.mapping_file;
    let json_plan = fs::read_to_string(path_to_plan_serialisation).unwrap();
    let plan_graph: PlanGraph = serde_json::from_str(&json_plan).unwrap();
    
    info!("Optimizing plan a little bit...");
    let reduced_plan = rewrite(&plan_graph);

    info!("Initializing execution engine...");
    // Create map of start node -> `send` channel 
    let mut sender_map: HashMap<usize, Vec<VecSender>> = HashMap::new();

    // Create map of end node -> `receive` channel
    let mut receiver_map: HashMap<usize, VecReceiver> = HashMap::new();

    for (id, node) in reduced_plan.iter() {
        // create channel: ONE per node (for incoming messages)
        if !node.from.is_empty() {
            let (sender, receiver) = bounded::<Vec<String>>(128);
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
    let mut join_handles: Vec<JoinHandle<()>> = Vec::new();

    for (id, node) in reduced_plan.iter() {
        let operator = &node.operator;

        match operator {

            // Create a source
            Operator::SourceOp { config } => {
                match config.source_type {
                    IOType::File => {
                        let file_path = config.config.get("path").unwrap();
                        let reference_formulation = &config.root_iterator.reference_formulation;
                        match reference_formulation {
                            ReferenceFormulation::CSVRows => {
                                let csv_file_source = CSVFileSource::new(file_path.to_string(), &node.attributes, id);
                                let senders = sender_map.remove(id).unwrap();
                                join_handles.push(csv_file_source.start(senders));
                            },
                            _ => {}
                        }
                    }
                    _ => {}
                }
            },

            // Create an Extension operator
            Operator::ExtendOp { config } => {
                let extend_pairs: &HashMap<String, Function> = &config.extend_pairs;
                let extend_operator = ExtendOperator::new(extend_pairs, id);
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
            Operator::TargetOp { .. } => {
                // TODO: do sometinh with config, just create a std out sink for now
                let receiver = receiver_map.remove(id).unwrap();
                let std_out_sink = StdOutSink::new();
                join_handles.push(std_out_sink.start(receiver));
            },

            Operator::JoinOp {config} => {
                // find left and right node ids
                let left = &node.from[0];
                let right = &node.from[1];
                
                let join_operator = JoinOperator::new(config, left, right, id);
                let senders = sender_map.remove(id).unwrap();
                let receiver = receiver_map.remove(id).unwrap();
                join_handles.push(join_operator.start(receiver, senders));
                println!();
            },
            
            _ => todo!()
        }
        
    }
    
    info!("Up and running!");

    for join_handle in join_handles {
        join_handle.join().unwrap();
    }
}