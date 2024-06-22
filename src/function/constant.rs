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

pub struct ConstantFunction {
    value: Vec<String>
}

impl ConstantFunction {
    pub fn new(value: String) -> Self {
        ConstantFunction { value: vec![value] }
    }
}

impl BasicFunction for ConstantFunction {
    fn exec(&self, _input: &[String]) -> Vec<String> {
        self.value.clone()
    }
}