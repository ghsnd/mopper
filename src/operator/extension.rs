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
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::JoinHandle;
use crossbeam_channel::{Receiver, Sender};
use log::{debug, error};
use operator::Function;
use crate::error::GeneralError;
use crate::function::basic_function::BasicFunction;
use crate::function::blank_node::BlankNodeFunction;
use crate::function::constant::ConstantFunction;
use crate::function::iri::IriFunction;
use crate::function::literal::LiteralFunction;
use crate::function::reference::ReferenceFunction;
use crate::function::template_string::TemplateStrFunction;

pub struct ExtendOperator {
    functions_mutex: Arc<Mutex<Vec<(String, Box<dyn BasicFunction + Send>)>>>,
    node_id: String,
    join_alias: Option<String>
}

impl ExtendOperator {
    pub fn new(extend_pairs: &HashMap<String, Function>, node_id: &usize, join_alias: &Option<String>) -> Result<&'static Self, GeneralError> {
        debug!("Initializing Extend operator {node_id}.");

        let mut functions: Vec<(String, Box<dyn BasicFunction + Send>)> = Vec::new();
        
        extend_pairs.iter().try_for_each(|(name, function_description)| {
            let function = get_function(function_description)?;
            functions.push((name.clone(), function));
            Ok(())
        })?;
        
        let boxed = Box::new(ExtendOperator{
            functions_mutex: Arc::new(Mutex::new(functions)),
            node_id: node_id.to_string(),
            join_alias: join_alias.clone()
        });
        Ok(Box::leak(boxed))
    }

    pub fn start(&'static self, rx_chan: Receiver<Vec<String>>, tx_channels: Vec<Sender<Vec<String>>>) -> JoinHandle<(u8, String)> {
        debug!("Starting ExtendOperator {}!", self.node_id);

        let functions_clone = self.functions_mutex.clone();
        
        thread::spawn(move || {
            let mut functions = functions_clone.lock().unwrap();
            
            // first send headers
            let mut node_id_plus_function_names = vec![self.node_id.clone()];
            node_id_plus_function_names.extend(functions.iter()
                .map(|(name, _function)| {
                    let minus_first_char = &name[1..];
                    minus_first_char.to_string()
                })
            );
            tx_channels.iter()
                .for_each(|tx_chan| tx_chan.send(node_id_plus_function_names.clone()).unwrap());
            
            // then send result types, so the serializer knows what to do with the string values
            let mut node_id_plus_result_types = vec![self.node_id.clone()];
            node_id_plus_result_types.extend(functions.iter()
                .map(|(_name, function)| {
                    function.get_result_type().to_string()
                })
            );
            tx_channels.iter()
                .for_each(|tx_chan| tx_chan.send(node_id_plus_result_types.clone()).unwrap());
            
            // now process values
            // Set the variable names ("headers") for the functions first
            let mut iter = rx_chan.iter();
            let variable_names_option = iter.next();
            if variable_names_option.is_some() {
                let variable_names = variable_names_option.unwrap();
                
                // Now here comes a hack to address removed self-joins:
                // the variable names are duplicated, but the join alias is prepended.
                // That way the functions keep on working if they expect variable names that would
                // be used by the join function
                if let Some(join_alias) = &self.join_alias {
                    let copy_of_variable_names_with_prefix= variable_names.iter()
                        .map(|name| {
                            let mut new_variable_name = join_alias.clone();
                            new_variable_name.push('_');
                            new_variable_name.push_str(name);
                            new_variable_name
                        });
                        //.collect();
                    let new_variable_names: Vec<String> = variable_names.iter()
                        .cloned()
                        .chain(copy_of_variable_names_with_prefix)
                        .collect();

                    functions.iter_mut().for_each(|(_name, function)| {
                        function.variable_names(new_variable_names.clone());
                    });
                    
                } else {
                    functions.iter_mut().for_each(|(_name, function)| {
                        function.variable_names(variable_names.clone());
                    });
                }
            }
            
            // Let each function process the data
            for data in iter {
                
                let data_to_process = match self.join_alias {
                    Some(_) => data.iter().chain(data.iter()).cloned().collect(), // is this a performant way to append a vec to itself?
                    None => data
                };
                
                // prepend node id
                let mut node_id_plus_result = vec![self.node_id.clone()];
                node_id_plus_result.extend(
                    functions.iter()
                        .map(|(_name, function)| { function.exec(&data_to_process) })
                        .flatten()
                );

                tx_channels.iter()
                    .for_each(|tx_chan| tx_chan.send(node_id_plus_result.clone()).unwrap());
            }

            (0, String::new())
            
        })
    }
}

fn get_function(function: &Function) -> Result<Box<dyn BasicFunction + Send>, GeneralError> {
    match function {
        Function::Constant { value } => {
            debug!(" function 'Constant': [{value}]");
            Ok(Box::new(ConstantFunction::new(value.clone())))
        },
        Function::UriEncode { inner_function } => {
            debug!(" function 'UriEncode'. Ignoring bc of issue in AlgeMapLoom where it occurs at the wrong place (it's handled in template processing now). Just passing through the inner function.");
            get_function(inner_function)
        },
        Function::Iri { inner_function } => {
            debug!(" function 'Iri'");
            let inner = get_function(inner_function)?;
            Ok(Box::new(IriFunction::new(inner)))
        },
        Function::TemplateString { value } => {
            debug!(" function 'TemplateString': [{value}]");
            let function = TemplateStrFunction::new(value)?;
            Ok(Box::new(function))
        },
        Function::TemplateFunctionValue { .. } => {
            error!(" function 'TemplateFunctionValue' not implemented yet.");
            todo!()
        },
        Function::BlankNode { inner_function } => {
            debug!(" function 'BlankNode'");
            let inner = get_function(inner_function)?;
            Ok(Box::new(BlankNodeFunction::new(inner)))
        },
        Function::Concatenate { .. } => {
            error!(" function 'Concatenate' not implemented yet.");
            todo!()
        },
        Function::FnO { .. } => {
            error!(" function 'FnO' not implemented yet.");
            todo!()
        },
        Function::Literal { inner_function, .. } => {
            debug!(" function 'Literal'");
            let inner = get_function(inner_function)?;
            Ok(Box::new(LiteralFunction::new(inner)))
        },
        Function::Lower { .. } => {
            error!(" function 'Lower' not implemented yet.");
            todo!()
        },
        Function::Upper { .. } => {
            error!(" function 'Upper' not implemented yet.");
            todo!()
        },
        Function::Reference { value } => {
            debug!(" function 'Reference': [{value}]");
            Ok(Box::new(ReferenceFunction::new(value.to_string())))
        },
        Function::Replace { .. } => {
            error!(" function 'Relace' not implemented yet.");
            todo!()
        }
    }
}
