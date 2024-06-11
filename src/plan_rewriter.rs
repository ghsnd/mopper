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
use std::hash::{DefaultHasher, Hash, Hasher};
use log::{debug, info};
use operator::Operator;
use crate::plan::{Node, PlanGraph};

// Add destination(s) to node
// Merge Projection operator into source
// Remove Fragment operator: add destinations to previous node
// Merge same source nodes

// TODO: if output is forced to std out and/or file, don't hash and put everything to e.g. 0 (and 1)

pub fn rewrite(plan: &PlanGraph, to_one_target: bool) -> HashMap<usize, Node> {
    info!("Optimizing AlgeMapLoom plan a bit.");
    let mut node_map: HashMap<usize, Node> = HashMap::new();
    
    let mut fragment_indices = Vec::new();
    let mut projection_indices = Vec::new();
    let mut io_hash_to_node_index: HashMap<u64, Vec<usize>> = HashMap::new();
    let mut join_indices = Vec::new();
    
    plan.nodes.iter().enumerate().for_each(|(id, node)| {
        match &node.operator {
            Operator::FragmentOp { .. } => {
                fragment_indices.push(id);
            },
            Operator::ProjectOp { .. } => {
                projection_indices.push(id);
            },
            Operator::SourceOp { config} => {
                add_to_hash_map(&mut io_hash_to_node_index, config, id, false);
            },
            Operator::TargetOp { config } => {
                add_to_hash_map(&mut io_hash_to_node_index, config, id, to_one_target);
            },
            Operator::JoinOp { .. } => {
                join_indices.push(id);
            },
            _ => {}
        }
        node_map.insert(id, node.clone());
    });
    let initial_nr_of_nodes = node_map.len();
   
    // Set "edges" into node objects
    for edge in &plan.edges {
        let from = edge[0].as_u64().unwrap() as usize;
        let to = edge[1].as_u64().unwrap() as usize;
        let from_node = node_map.get_mut(&from).unwrap();
        from_node.add_to(to);
        let to_node = node_map.get_mut(&to).unwrap();
        to_node.add_from(from);
    }
    
    debug!("Merging nodes with same source or sink.");
    let mut merged_io_ids_to_remove: Vec<usize> = Vec::new();
    let mut changed_nodes: Vec<(usize, Node)> = Vec::new();   // these sources remain after merge with updated "to" edges
    io_hash_to_node_index.values()
        .filter(|nodes| nodes.len() > 1)
        .for_each(|same_io_node_ids| {
            // "io" stands for source or sink here
            let mut io_iter = same_io_node_ids.iter();
            
            // get first source node
            let first_io_id = io_iter.next().unwrap();
            let mut first_io = node_map[first_io_id].clone();
            
            // now add the "to" edges from the other nodes to the first one
            // and remove the other nodes from the map
            for other_io_id in io_iter {
                let other_io = &node_map[other_io_id];
                first_io.add_all_to(&other_io.to);
                first_io.add_all_from(&other_io.from);
                merged_io_ids_to_remove.push(*other_io_id);
                
                //let other_source = &node_map[other_io_id];
                // update the "to" nodes, because their "from"s still point to other sources
                for to_node_id in &other_io.to {
                    let mut to_node_to_update = node_map[to_node_id].clone();
                    to_node_to_update.replace_from(*other_io_id, *first_io_id);
                    changed_nodes.push((*to_node_id, to_node_to_update));
                }
                // update the "from" nodes, because their "to"s still point to other sinks
                for from_node_id in &other_io.from {
                    let mut from_node_to_update = node_map[from_node_id].clone();
                    from_node_to_update.replace_to(*other_io_id, *first_io_id);
                    changed_nodes.push((*from_node_id, from_node_to_update));
                }
            }
            changed_nodes.push((*first_io_id, first_io));
        });
    // Remove duplicate source nodes update merged source nodes
    for merged_source_id in merged_io_ids_to_remove {
        debug!("Removing source or sink node {merged_source_id}");
        node_map.remove(&merged_source_id);
    }
    
    // Update changed source nodes
    for (node_id, updated_node) in changed_nodes {
        debug!("Updating node {node_id}");
        node_map.insert(node_id, updated_node);
    }
    
    // Remove Fragment operators by setting their edges to involved nodes
    // e.g. A -> Fragmenter -> B and C
    //      A -> B and C
    debug!("Removing Fragment nodes from plan.");
    for fragment_index in fragment_indices {
        debug!("Removing fragment node {fragment_index}");
        let fragment_node = node_map.remove(&fragment_index).unwrap();
        
        // move "to" edges to "from" node 
        for start_node_index in &fragment_node.from {
            let start_node = node_map.get_mut(&start_node_index).unwrap();
            start_node.change_to_ids(&fragment_node.to, fragment_index);

            // move "from" edges to "to" node
            for end_node_index in &fragment_node.to {
                let end_node = node_map.get_mut(&end_node_index).unwrap();
                end_node.replace_from(fragment_index, *start_node_index);
            }
        }
    }
    
    // Remove Project operators by passing their attributes to the "previous" operator
    debug!("Removing Projection nodes from plan.");
    for projection_index in projection_indices {
        debug!("Removing projection node {projection_index}");
        let projection_node = node_map.remove(&projection_index).unwrap();
        
        // get attributes of projection
        let attributes = match projection_node.operator {
            Operator::ProjectOp { config } => Some(config.projection_attributes),
                                                   _ => None
        };
        
        // add "to" edges to "from" node
        // AND set attributes to "from" node operator
        for start_node_index in &projection_node.from {
            let start_node = node_map.get_mut(&start_node_index).unwrap();
            start_node.change_to_ids(&projection_node.to, projection_index);
            start_node.add_attributes(attributes.clone());

            // move "from" edges to "to" node
            for end_node_index in &projection_node.to {
                let end_node = node_map.get_mut(&end_node_index).unwrap();
                end_node.replace_from(projection_index, *start_node_index);
            }
        }
    }
    
    debug!("Removing self-join nodes from plan.");
    let mut self_join_nodes_to_remove: Vec<usize> = Vec::new();
    let mut changed_join_connected_nodes: Vec<(usize, Node)> = Vec::new();
    
    for join_index in join_indices {
        let join_node = &node_map[&join_index];
        if join_node.from[0] == join_node.from[1] {
            self_join_nodes_to_remove.push(join_index);
            // make sure the renamed attributes also get passed
            match &join_node.operator {
                Operator::JoinOp { config } => {
                    let join_alias = &config.join_alias;
                    for to_node_id in &join_node.to {
                        let mut to_node = node_map[to_node_id].clone();
                        to_node.replace_from(join_index, join_node.from[0]);
                        to_node.join_alias = Some(join_alias.to_string());
                        changed_join_connected_nodes.push((*to_node_id, to_node));
                    }
                    let from_node_id = &join_node.from[0];
                    let mut from_node = node_map[from_node_id].clone();
                    from_node.change_to_ids(&join_node.to, join_index);
                    changed_join_connected_nodes.push((*from_node_id, from_node));
                },
                _ => {}
            }
        }
    }
    for (node_id, updated_node) in changed_join_connected_nodes {
        debug!("Updating node {node_id} affected by removing self-join node");
        node_map.insert(node_id, updated_node);
    }
    
    for id in self_join_nodes_to_remove {
        debug!("Removing self-join {id}");
        node_map.remove(&id);
    }
    
    let final_nr_of_nodes = node_map.len();
    info!("Reduced number of nodes in the plan from {initial_nr_of_nodes} to {final_nr_of_nodes}");
    
    node_map
}

fn add_to_hash_map<T: Hash>(io_hash_to_node_index: &mut HashMap<u64, Vec<usize>>, config: T, id: usize, constant_hash: bool) {
    // The idea here is to group sources with the same configuration together as they are
    // basically the same. The next step is then to merge them into one source.
    let hash = match constant_hash {
        true => 0,
        false => {
            let mut hasher = DefaultHasher::new();
            config.hash(&mut hasher);
            hasher.finish()
        }
    };
    if io_hash_to_node_index.contains_key(&hash) {
        let node_ids = io_hash_to_node_index.get_mut(&hash).unwrap();
        node_ids.push(id);
    } else {
        let node_ids = vec![id];
        io_hash_to_node_index.insert(hash, node_ids);
    }
}