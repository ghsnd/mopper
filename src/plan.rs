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
use operator::Operator;
use serde::Deserialize;
use serde_json::Value;

#[derive(Deserialize, Clone)]
pub struct Node {
    pub operator: Operator,

    // The edges
    #[serde(default = "Vec::new")]
    pub from: Vec<usize>,

    #[serde(default = "HashSet::new")]
    pub to: HashSet<usize>,

    pub attributes: Option<HashSet<String>>,
    
    pub join_alias: Option<String>
}


impl Node {
    pub fn add_from(&mut self, id: usize) {
        self.from.push(id);
    }

    pub fn replace_from(&mut self, old_id: usize, new_id: usize) {
        // TODO: can be more optimal?
        self.from.iter_mut()
            .for_each(|id| {
                if *id == old_id {
                    *id = new_id;
                }
            });
    }
    
    pub fn add_all_from(&mut self, ids: &[usize]) {
        self.from.extend_from_slice(ids);
    }

    pub fn add_to(&mut self, id: usize) {
        self.to.insert(id);
    }
    
    pub fn add_all_to(&mut self, ids: &HashSet<usize>) {
        self.to.extend(ids);
    }

    pub fn replace_to(&mut self, old_id: usize, new_id: usize) {
        self.to.remove(&old_id);
        self.to.insert(new_id);
    }
    
    pub fn change_to_ids(&mut self, ids_to_add: &HashSet<usize>, id_to_remove: usize) {
        self.to.remove(&id_to_remove);
        self.to.extend(ids_to_add);
    }
    
    pub fn add_attributes(&mut self, attributes: Option<HashSet<String>>) {
        if self.attributes.is_some() {
            if attributes.is_some() {
                let original_attributes = self.attributes.as_mut().unwrap();
                let new_attributes: HashSet<String> = original_attributes.union(&attributes.unwrap())
                    .map(|value| {value.to_string()})
                    .collect();
                self.attributes = Some(new_attributes);
            }
        } else {
            self.attributes = attributes;
        }
    }
}

#[derive(Deserialize)]
pub struct PlanGraph {
    pub nodes: Vec<Node>,
    pub edges: Vec<Vec<Value>>
}