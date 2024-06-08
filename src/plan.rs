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
    pub id: String,     // just a name
    pub operator: Operator,

    // The edges
    #[serde(default = "Vec::new")]
    pub from: Vec<usize>,

    #[serde(default = "Vec::new")]
    pub to: Vec<usize>,

    pub attributes: Option<HashSet<String>>
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
        self.to.push(id);
    }
    
    pub fn add_all_to(&mut self, ids: &[usize]) {
        self.to.extend_from_slice(ids);
    }

    pub fn replace_to(&mut self, old_id: usize, new_id: usize) {
        // TODO: can be more optimal?
        self.to.iter_mut()
            .for_each(|id| {
                if *id == old_id {
                    *id = new_id;
                }
            });
    }
    
    pub fn change_to_ids(&mut self, ids_to_add: &[usize], id_to_remove: usize) {
        self.to = self.to.iter()
            .filter(|id| **id != id_to_remove)
            .chain(ids_to_add)
            .map(|id| *id)
            .collect();
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