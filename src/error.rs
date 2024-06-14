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

use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub struct GeneralError {
    errors: Vec<(u8, String)>
}

impl GeneralError {
    pub fn new(errors: Vec<(u8, String)>) -> Self {
        GeneralError {errors}
    }
    
    pub fn from_msg(message: String) -> Self {
        Self::new(vec![(1, message)])
    }
}

impl Display for GeneralError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let error_strs: Vec<String> = self.errors.iter()
            .map(|(err_code, msg)| {
                format!("{} (code {})", msg, err_code)
            }).collect();
        f.write_fmt(format_args!("{:?}", error_strs.join("\n")))
    }
}

unsafe impl Send for GeneralError {}

impl Error for GeneralError {}