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

use crate::function::basic_function::BasicFunction;
use crate::util::remove_join_alias_prefix;

pub struct ReferenceFunction {
    variable_name: String,
    index: usize
}

impl ReferenceFunction {
    pub fn new(variable_name: String, join_alias: &Option<String>) -> Self {
        ReferenceFunction{
            variable_name: remove_join_alias_prefix(&variable_name, join_alias),
            index: 0
        }
    }
}

impl BasicFunction for ReferenceFunction {
    fn variable_names(&mut self, variable_names: &[String]) {
        for (index, name) in variable_names.iter().enumerate() {
            if *name == self.variable_name {
                self.index = index;
                break;
            }
        }
    }

    fn exec(&self, input: &[String]) -> Vec<String> {
        vec![input[self.index].to_string()]
    }
}