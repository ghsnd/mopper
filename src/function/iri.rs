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
use iri_string::spec::UriSpec;
use iri_string::validate::{iri, iri_reference};
use log::error;
use crate::function::basic_function::BasicFunction;

pub struct IriFunction {
    base_iri: Option<String>,
    inner_function: Box<dyn BasicFunction + Send>
}

impl IriFunction {
    pub fn new(base_iri: &Option<String>, inner_function: Box<dyn BasicFunction + Send>) -> Self {
        IriFunction {
            base_iri: base_iri.clone(),
            inner_function
        }
    }
}

impl BasicFunction for IriFunction {
    fn variable_names(&mut self, variable_names: &[String]) {
        self.inner_function.variable_names(variable_names);
    }

    fn get_result_type(&self) -> &str {
        "iri"
    }

    fn exec(&self, input: &[String]) -> Vec<String> {
        let output = self.inner_function.exec(input);

        output.into_iter()
            .map(|value| {
                // check if the value is an absolute IRI
                let absolute_iri_check = iri::<UriSpec>(&value);
                if absolute_iri_check.is_ok() {
                    return value;
                } else {
                    let iri = match &self.base_iri {
                        Some(base_iri) => format!("{base_iri}{value}"),
                        None => value
                    };
                    // check if it's a valid IRI
                    let valid_iri_check = iri_reference::<UriSpec>(&iri);
                    if valid_iri_check.is_ok() {
                        return iri
                    } else {
                        error!("Invalid IRI: {iri}");
                        return "INVALID".to_string();
                    }
                }
            })
            .collect()
    }
}
