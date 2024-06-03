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
use pct_str::{IriReserved, PctString};

pub trait BasicFunction {
    fn variable_names(&mut self, _variable_names: Vec<String>) {}  // by default ignore the headers

    // Returns the type of the result of the function
    // The default is 'str'
    // TODO replace type string with enum
    fn get_result_type(&self) -> &str {
        "str"
    }

    fn exec(&self, input: &[String]) -> Vec<String>;
}

pub struct ConstantFunction {
    value: Vec<String>
}

///////////////// Constant /////////////////

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

///////////////// UriEncode /////////////////

// pub struct UriEncodeFunction {
//     inner_function: Box<dyn BasicFunction + Send>
// }
// 
// impl UriEncodeFunction {
//     pub fn new(inner_function: Box<dyn BasicFunction + Send>) -> Self {
//         UriEncodeFunction {inner_function}
//     }
// }
// 
// impl BasicFunction for UriEncodeFunction {
// 
//     fn variable_names(&mut self, variable_names: Vec<String>) {
//         self.inner_function.variable_names(variable_names);
//     }
//     fn exec(&self, input: &[String]) -> Vec<String> {
//         let inner_result = self.inner_function.exec(input);
//         inner_result.iter().map(|value| {
//             let pct_str = PctString::encode(value.chars(), URIReserved);
//             pct_str.into_string()
//         }).collect()
//     }
// }

///////////////// IRI /////////////////
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

///////////////// TemplateString /////////////////

pub struct TemplateStrFunction {
    // ex: A {template} string.
    // [(false, 'A '),(true, template), (false, ' string.')] (a vector with template string parts)
    template_string_parts: Vec<(bool, String)>,
    variable_names: Vec<String>
}

impl TemplateStrFunction {
    pub fn new(template: &str) -> Self {


        // dummy template parsing
        let mut template_string_parts: Vec<(bool, String)> = Vec::with_capacity(2);
        let mut current_str = String::new();
        let mut between_cb = false;     // TODO: replace by counter to deal with nested '{'

        // TODO: better parsing, error handling, ...
        template.chars().for_each(|c| {
            match c {
                '{' => {
                    if !between_cb {
                        if !current_str.is_empty() {
                            template_string_parts.push((false, current_str.to_string()));
                            current_str.clear();
                        }
                        between_cb = true;
                    }
                },
                '}' => {
                    if between_cb {
                        if !current_str.is_empty() {
                            template_string_parts.push((true, current_str.to_string()));
                            current_str.clear();
                        }
                        between_cb = false;
                    }
                }
                _ => {
                    current_str.push(c);
                }
            }
        });
        
        // add last part, if any
        if !current_str.is_empty() {
            template_string_parts.push((false, current_str.to_string()));
        }
        
        TemplateStrFunction{
            template_string_parts,
            variable_names: Vec::with_capacity(1)
        }
    }
}

impl BasicFunction for TemplateStrFunction {
    fn variable_names(&mut self, variable_names: Vec<String>) {
        self.variable_names = variable_names;
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
                    let pct_str = PctString::encode(value.chars(), IriReserved::Segment);
                    result_str.push_str(pct_str.as_str());
                } else {
                    result_str.push_str(part);
                }
            });
        vec![result_str]
    }
}


///////////////// Literal /////////////////
pub struct LiteralFunction {
    inner_function: Box<dyn BasicFunction + Send>
}

impl LiteralFunction {
    pub fn new(inner_function: Box<dyn BasicFunction + Send>) -> Self {
        LiteralFunction { inner_function }
    }
}

 impl BasicFunction for LiteralFunction {

     fn variable_names(&mut self, variable_names: Vec<String>) {
         self.inner_function.variable_names(variable_names);
     }

     fn get_result_type(&self) -> &str {
         // TODO: send data type of literal somehow
         "lit"
     }

     fn exec(&self, input: &[String]) -> Vec<String> {
         self.inner_function.exec(input)
    }
 }


///////////////// Reference /////////////////

pub struct ReferenceFunction {
    variable_name: String,
    index: usize
}

impl ReferenceFunction {
    pub fn new(variable_name: String) -> Self {
        ReferenceFunction{variable_name, index: 0}
    }
}

impl BasicFunction for ReferenceFunction {
    fn variable_names(&mut self, variable_names: Vec<String>) {
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

///////////////// Literal /////////////////
pub struct BlankNodeFunction {
    inner_function: Box<dyn BasicFunction + Send>
}

impl BlankNodeFunction {
    pub fn new(inner_function: Box<dyn BasicFunction + Send>) -> Self {
        BlankNodeFunction { inner_function }
    }
}

impl BasicFunction for BlankNodeFunction {

    fn variable_names(&mut self, variable_names: Vec<String>) {
        self.inner_function.variable_names(variable_names);
    }

    fn get_result_type(&self) -> &str {
        "blank"
    }
    
    fn exec(&self, input: &[String]) -> Vec<String> {
        self.inner_function.exec(input)
    }
}