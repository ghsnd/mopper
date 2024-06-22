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

pub struct IriFunction {
    inner_function: Box<dyn BasicFunction + Send>
}

impl IriFunction {
    pub fn new(inner_function: Box<dyn BasicFunction + Send>) -> Self {
        IriFunction {inner_function}
    }
}

impl BasicFunction for IriFunction {
    fn variable_names(&mut self, variable_names: Vec<String>) {
        self.inner_function.variable_names(variable_names);
    }

    fn get_result_type(&self) -> &str {
        "iri"
    }

    fn exec(&self, input: &[String]) -> Vec<String> {
        // TODO: maybe check if the IRI is valid?
        self.inner_function.exec(input)
    }
}
