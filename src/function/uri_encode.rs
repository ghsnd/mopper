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

use pct_str::{PctString, URIReserved};
use crate::function::basic_function::BasicFunction;

pub struct UriEncodeFunction {
    inner_function: Box<dyn BasicFunction + Send>
}

impl UriEncodeFunction {
    // Not used at the moment...
    pub fn _new(inner_function: Box<dyn BasicFunction + Send>) -> Self {
        UriEncodeFunction {inner_function}
    }
}

impl BasicFunction for UriEncodeFunction {

    fn variable_names(&mut self, variable_names: Vec<String>) {
        self.inner_function.variable_names(variable_names);
    }
    fn exec(&self, input: &[String]) -> Vec<String> {
        let inner_result = self.inner_function.exec(input);
        inner_result.iter().map(|value| {
            let pct_str = PctString::encode(value.chars(), URIReserved);
            pct_str.into_string()
        }).collect()
    }
}
