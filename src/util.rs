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

pub fn remove_join_alias_prefix(variable_name: &str, join_alias: &Option<String>) -> String {
    match join_alias {
        Some(alias) => {
            if variable_name.starts_with(alias) {
                variable_name[alias.len() + 1..].to_string()
            } else {
                variable_name.to_string()
            }
        },
        None => variable_name.to_string()
    }
}