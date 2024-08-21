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

pub trait BasicFunction {
    fn variable_names(&mut self, _variable_names: &[String]) {}  // by default ignore the headers

    // Returns the type of the result of the function
    // The default is 'str'
    // TODO replace type string with enum
    fn get_result_type(&self) -> &str {
        "str"
    }

    fn exec(&self, input: &[String]) -> Vec<String>;
}
