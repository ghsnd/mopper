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
use operator::Join;
use operator::JoinType::InnerJoin;

// TODO: can be optimized when using only attributes of the next operator.
// TODO: this algorithm assumes every join attribute gets checked against only *1* other join attribute.

pub struct JoinOperator {
    node_id: String,
    left_node_id: String,   // in RML: the "child"
    right_node_id: String,  // in RML: the "parent"
    left_right_join_attr_pairs: Vec<(String, String)>,
    right_node_attr_prefix: String      // = "join alias" in the mapping plan. Prefix to use for attribute names coming from the right node
}

impl JoinOperator {
    pub fn new(config: &Join, left_node_id: &usize, right_node_id: &usize, node_id: &usize) -> &'static Self {
        debug!("Initializing Join operator {node_id}.");

        // Only inner join supported for now.
        if config.join_type != InnerJoin {
            error!("Join type {:?} is not supported", config.join_type);
            todo!()
        }

        let boxed = Box::new(JoinOperator{
            node_id: node_id.to_string(),
            left_node_id: left_node_id.to_string(),
            right_node_id: right_node_id.to_string(),
            left_right_join_attr_pairs: config.left_right_attr_pairs.clone(),
            right_node_attr_prefix: format!("{}_", config.join_alias) // use this as prefix to attributes of right node
        });
        Box::leak(boxed)
    }
    
    pub fn start(&'static self, rx_chan: Receiver<Vec<String>>, tx_channels: Vec<Sender<Vec<String>>>) -> JoinHandle<(u8, String)>{
        debug!("Starting Join operator {}!", self.node_id);

        thread::Builder::new()
            .name(format!("Join {}", self.node_id))
            .spawn(move || {
            let mut left_attribute_names: Vec<String> = Vec::with_capacity(self.left_right_join_attr_pairs.len());
            let mut right_attribute_names: Vec<String> = Vec::with_capacity(self.left_right_join_attr_pairs.len());

            // initialize some data structures used during join
            let mut left_join_attribute_indices: Vec<usize> = Vec::with_capacity(self.left_right_join_attr_pairs.len());
            let mut right_join_attribute_indices: Vec<usize> = Vec::with_capacity(self.left_right_join_attr_pairs.len());

            let mut left_join_data = JoinData::new(self.left_right_join_attr_pairs.len());
            let mut right_join_data = JoinData::new(self.left_right_join_attr_pairs.len());
            
            for data in rx_chan.iter() {
                let node_id = &data[0];
                debug!("Processing join data of node {node_id}");
                let real_data = &data[1..];
                
                if node_id.eq(&self.left_node_id) {
                    // process left data


                    if left_join_attribute_indices.is_empty() {
                        // get names and positions of attributes
                        let join_attribute_names: Vec<&String> = self.left_right_join_attr_pairs.iter()
                            .map(|(left, _right)| left)
                            .collect();

                        for (position, name) in real_data.iter().enumerate() {
                            left_attribute_names.push(name.clone());
                            if join_attribute_names.contains(&name) {
                                left_join_attribute_indices.push(position);
                            }
                        }
                        left_join_data.set_join_attribute_positions(&left_join_attribute_indices);

                        if !right_attribute_names.is_empty() {
                            let all_attribute_names: Vec<String> = vec![self.node_id.clone()].iter()
                                .chain(left_attribute_names.iter())
                                .chain(right_attribute_names.iter())
                                .map(|value| value.clone())
                                .collect();
                            tx_channels.iter()
                                .for_each(|tx_chan| tx_chan.send(all_attribute_names.clone()).unwrap());
                            left_attribute_names.clear();
                        }

                    } else {
                        // we have some data!
                        let join_result_option = process_data_for_one_join_side(real_data, &mut left_join_data, &mut right_join_data);
                        if let Some(join_result) = join_result_option {
                            for join_data in join_result {
                                let data_to_send: Vec<String> = vec![self.node_id.clone()].iter()
                                    .chain(real_data)
                                    .chain(join_data)
                                    .map(|value| value.clone())
                                    .collect();
                                tx_channels.iter()
                                    .for_each(|tx_chan| tx_chan.send(data_to_send.clone()).unwrap());
                            }
                        }
                    }

                } else if node_id.eq(&self.right_node_id) {
                    // process right data

                    // find the indices of the attributes (first time only)
                    if right_join_attribute_indices.is_empty() {
                        // get names and positions of attributes
                        let join_attribute_names: Vec<&String> = self.left_right_join_attr_pairs.iter()
                            .map(|(_left, right)| right)
                            .collect();

                        for (position, name) in real_data.iter().enumerate() {
                            let new_name = format!("{}{}", self.right_node_attr_prefix, name);
                            right_attribute_names.push(new_name);
                            if join_attribute_names.contains(&name) {
                                right_join_attribute_indices.push(position);
                            }
                        }
                        right_join_data.set_join_attribute_positions(&right_join_attribute_indices);

                        if !left_attribute_names.is_empty() {
                            let all_attribute_names: Vec<String> = vec![self.node_id.clone()].iter()
                                .chain(left_attribute_names.iter())
                                .chain(right_attribute_names.iter())
                                .map(|value| value.clone())
                                .collect();
                            tx_channels.iter()
                                .for_each(|tx_chan| tx_chan.send(all_attribute_names.clone()).unwrap());
                            right_attribute_names.clear();
                        }
                    } else {
                        // we have some data!
                        let join_result_option = process_data_for_one_join_side(real_data, &mut right_join_data, &mut left_join_data);
                        if let Some(join_result) = join_result_option {
                            for join_data in join_result {
                                let data_to_send: Vec<String> = vec![self.node_id.clone()].iter()
                                    .chain(join_data)
                                    .chain(real_data)
                                    .map(|value| value.clone())
                                    .collect();
                                tx_channels.iter()
                                    .for_each(|tx_chan| tx_chan.send(data_to_send.clone()).unwrap());
                            }
                        }
                    }
                }
            }

            (0, String::new())
            
        }).unwrap()
    }
}

fn process_data_for_one_join_side<'a> (data:                    &[String],
                                   join_data:               &mut JoinData, 
                                   other_join_data:         &'a mut JoinData,
) -> Option<Vec<&'a Vec<String>>>
{
    let join_attr_values = join_data.add(data);
    other_join_data.return_values_if_match(&join_attr_values)

}

struct JoinData {
    // the position of join attributes in the original data
    join_attr_positions: Vec<usize>,

    // data of non-join attributes
    data: Vec<Vec<String>>,
    //    |   └> a data row: a value for every attribute, in order of original data
    //    └> vector of rows

    // data of join attributes
    join_attr_indices: Vec<HashMap<String, Vec<usize>>>,
    //                 |           |       └> vector of indices to the 'data' vector
    //                 |           └> the value of the join attribute
    //                 └> the map at index 'n' applies to the n-th join attribute
    
    // every record in 'data' before this index is processed by the other join data instance
    //latest_retrieved_data_index: usize
}

impl JoinData {
    fn new (nr_join_attributes: usize) -> JoinData {
        let mut join_attr_indices = Vec::with_capacity(nr_join_attributes);
        
        // initialize join_attr_indices with empty maps to avoid creating them when adding data
        for _i in 0..nr_join_attributes {
            let empty_map: HashMap<String, Vec<usize>> = HashMap::new();
            join_attr_indices.push(empty_map);
        }
        
        JoinData {
            join_attr_positions: Vec::new(),
            data: Vec::new(),
            join_attr_indices,
        }
    }
    
    fn set_join_attribute_positions(&mut self, join_attribute_positions: &[usize]) {
        self.join_attr_positions.extend(join_attribute_positions);
    }

    fn add(&mut self, data: &[String]) -> Vec<String> { // return join_attr_values
        
        // get the values of the join attributes
        let join_attr_values: Vec<String> = data.iter().enumerate()
            .filter(|(position, _value)| self.join_attr_positions.contains(position))
            .map(|(_position, value)| value)
            .map(|value| value.clone())
            .collect();
        
        self.data.push(data.to_vec());
        
        let data_row_nr = self.data.len() - 1;
        
        // for every join attribute value, add its index in the data value to the map value -> indices
        for (join_attr_position, join_attr_value) in join_attr_values.iter().enumerate() {
            let attr_index_map = self.join_attr_indices.get_mut(join_attr_position).unwrap();
            let data_position_vec_option = attr_index_map.get_mut(join_attr_value);
            if data_position_vec_option.is_some() {
                let data_position_vec = data_position_vec_option.unwrap();
                data_position_vec.push(data_row_nr);
            } else {
                let mut data_position_vec = Vec::with_capacity(2);
                data_position_vec.push(data_row_nr);
                attr_index_map.insert(join_attr_value.clone(), data_position_vec);
            }
        }

        join_attr_values
    }

    fn return_values_if_match(&self, join_attr_values: &[String]) -> Option<Vec<&Vec<String>>> {

        let mut found_data_indices: Vec<&Vec<usize>> = Vec::new();

        for (position, join_attr_value) in join_attr_values.iter().enumerate() {
            // look up value in map with this index
            let index_map = &self.join_attr_indices[position];

            // see if there is matching data
            if let Some(data_indices) = index_map.get(join_attr_value) {
                found_data_indices.push(data_indices);
            } else {
                return None
            }
        }

        // now only the indices occurring in all found data indices are a real match
        let mut found_data_indices_iter = found_data_indices.iter();
        let mut result: Vec<usize> = found_data_indices_iter.next().unwrap().to_vec();

        for found_data_index_vec in found_data_indices_iter {
            result.retain(|index| { found_data_index_vec.contains(index) });
        }

        if result.is_empty() {
            None
        } else {
            let final_result: Vec<&Vec<String>> = result.iter()
                .map(|index| &self.data[*index])
                .collect();
            Some(final_result)
        }
    }
}