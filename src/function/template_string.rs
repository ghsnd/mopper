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
use crate::error::GeneralError;
use crate::function::basic_function::BasicFunction;
use crate::function::template_parser::parse_template;

pub struct TemplateStrFunction {
    // ex: A {template} string.
    // [(false, 'A '),(true, template), (false, ' string.')] (a vector with template string parts)
    template_string_parts: Vec<(bool, String)>,
    variable_names: Vec<String>
}

impl TemplateStrFunction {
    pub fn new(template: &str, join_alias: &Option<String>) -> Result<Self, GeneralError> {
        Ok(TemplateStrFunction{
            template_string_parts: parse_template(template, join_alias)?,
            variable_names: Vec::with_capacity(1)
        })
    }
}

impl BasicFunction for TemplateStrFunction {
    fn variable_names(&mut self, variable_names: &[String]) {
        self.variable_names = variable_names.to_vec();
    }
    fn exec(&self, input: &[String]) -> Vec<String> {
        let mut variable_name_to_value_map = HashMap::with_capacity(input.len());
        for (index, value) in input.iter().enumerate() {
            let variable_name = &self.variable_names[index];
            variable_name_to_value_map.insert(variable_name, value);
        }

        let mut result_str = String::new();

        self.template_string_parts.iter()
            .for_each(|(is_variable, part)| {
                if *is_variable {
                    let value = variable_name_to_value_map[part];
                    result_str.push_str(value);
                } else {
                    result_str.push_str(part);
                }
            });
        vec![result_str]
    }
}