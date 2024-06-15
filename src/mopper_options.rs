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

#[derive(Default, Builder, Debug)]
pub struct MopperOptions {
    
    /// Ignore sink configurations and force output to standard out, unless force_to_file is set.
    #[builder(default="false", setter(strip_option))]
    force_to_std_out: bool,

    /// Ignore sink configurations and force output to file. Overrides force_to_std_out.
    #[builder(setter(into, strip_option), default="None")]
    force_to_file: Option<String>,

    /// Set the working directory virtually to this path.
    /// This is used by file sources to search for files relative to this path. 
    #[builder(setter(into, strip_option), default="None")]
    working_dir_hint: Option<String>,

    /// Set the maximum number of messages each communication channel can hold before blocking the
    /// sender thread. `0` means no messages are hold: 'send' and 'receive' must happen at the same time .
    #[builder(default="128")]
    message_buffer_capacity: usize
}

impl MopperOptions {
    pub fn force_to_std_out(&self) -> bool {
        self.force_to_std_out
    }
    pub fn force_to_file(&self) -> &Option<String> {
        &self.force_to_file
    }
    pub fn working_dir_hint(&self) -> &Option<String> {
        &self.working_dir_hint
    }
    pub fn message_buffer_capacity(&self) -> usize {
        self.message_buffer_capacity
    }
}
