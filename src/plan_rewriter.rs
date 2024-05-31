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
use log::debug;
use operator::Operator;
use crate::plan::{Node, PlanGraph};

// Add destination(s) to node
// Merge Projection operator into source
// Remove Fragment operator: add destinations to previous node
pub fn rewrite(plan: &PlanGraph) -> HashMap<usize, Node> {
    let mut node_map: HashMap<usize, Node> = HashMap::new();
    
    debug!("Removing Fragment and Projection operators from plan.");
    let mut fragment_indices = Vec::new();
    let mut projection_indices = Vec::new();
    plan.nodes.iter().enumerate().for_each(|(id, node)| {
        match node.operator {
            Operator::FragmentOp { .. } => {
                fragment_indices.push(id);
            },
            Operator::ProjectOp { .. } => {
                projection_indices.push(id);
            }
            _ => {}
        }
        node_map.insert(id, node.clone());
    });
   
    // Set "edges" into node objects
    for edge in &plan.edges {
        let from = edge[0].as_u64().unwrap() as usize;
        let to = edge[1].as_u64().unwrap() as usize;
        let from_node = node_map.get_mut(&from).unwrap();
        from_node.add_to(to);
        let to_node = node_map.get_mut(&to).unwrap();
        to_node.add_from(from);
    }
    
    // Remove Fragment operators by setting their edges to involved nodes
    // e.g. A -> Fragmenter -> B and C
    //      A -> B and C
    for fragment_index in fragment_indices {
        let fragment_node = node_map.remove(&fragment_index).unwrap();
        
        // move "to" edges to "from" node 
        for start_node_index in &fragment_node.from {
            let start_node = node_map.get_mut(&start_node_index).unwrap();
            start_node.set_all_to(&fragment_node.to);

            // move "from" edges to "to" node
            for end_node_index in &fragment_node.to {
                let end_node = node_map.get_mut(&end_node_index).unwrap();
                end_node.replace_from(fragment_index, *start_node_index);
            }
        }
    }
    
    // Remove Project operators by passing their attributes to the "previous" operator
    for projection_index in projection_indices {
        let projection_node = node_map.remove(&projection_index).unwrap();
        
        // get attributes of projection
        let attributes = match projection_node.operator {
            Operator::ProjectOp { config } => Some(config.projection_attributes),
                                                   _ => None
        };
        
        // move "to" edges to "from" node
        // AND set attributes to "from" node operator
        for start_node_index in &projection_node.from {
            let start_node = node_map.get_mut(&start_node_index).unwrap();
            start_node.set_all_to(&projection_node.to);
            start_node.add_attributes(attributes.clone());

            // move "from" edges to "to" node
            for end_node_index in &projection_node.to {
                let end_node = node_map.get_mut(&end_node_index).unwrap();
                end_node.replace_from(projection_index, *start_node_index);
            }
        }
    }
    
    // TODO: check for same sources & sinks
    
    node_map
}