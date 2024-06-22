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
use crate::error::GeneralError;
use crate::function::basic_function::BasicFunction;

pub struct TemplateStrFunction {
    // ex: A {template} string.
    // [(false, 'A '),(true, template), (false, ' string.')] (a vector with template string parts)
    template_string_parts: Vec<(bool, String)>,
    variable_names: Vec<String>
}

impl TemplateStrFunction {
    pub fn new(template: &str) -> Result<Self, GeneralError> {
        Ok(TemplateStrFunction{
            template_string_parts: parse_template(template)?,
            variable_names: Vec::with_capacity(1)
        })
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

fn parse_template(template: &str) -> Result<Vec<(bool, String)>, GeneralError> {
    let mut template_string_parts: Vec<(bool, String)> = Vec::with_capacity(2);
    let mut current_str = String::new();
    let mut between_cb = false;
    let mut escape = false;

    template.chars().try_for_each(|c| {
        match c {
            '{' => {
                if escape {
                    current_str.push('{');
                    escape = false;
                } else {
                    if between_cb {
                        let err_msg = format!("Error parsing template '{template}': Unescaped '{{' found between {{}}.");
                        return Err(GeneralError::from_msg(err_msg.to_string()))
                    } else {
                        if !current_str.is_empty() {
                            template_string_parts.push((false, current_str.to_string()));
                            current_str.clear();
                        }
                        between_cb = true;
                    }
                }
            },
            '}' => {
                if escape {
                    current_str.push('}');
                    escape = false;
                } else {
                    if between_cb {
                        if !current_str.is_empty() {
                            template_string_parts.push((true, current_str.to_string()));
                            current_str.clear();
                        }
                        between_cb = false;
                    } else {
                        let err_msg = format!("Error parsing template '{template}': Unescaped '}}' found between {{}}.");
                        return Err(GeneralError::from_msg(err_msg.to_string()))
                    }
                }
            },
            '\\' => {
                if escape {
                    current_str.push('\\');
                    escape = false;
                } else {
                    escape = true;
                }
            }
            _ => {
                if escape {
                    let err_msg = format!("Error parsing template '{template}': character '{c}' is being escaped, but it doesn't need escaping.");
                    return Err(GeneralError::from_msg(err_msg.to_string()))
                }
                current_str.push(c);
            }
        };

        // End of parsing reached, everything seems to be OK
        Ok(())
    })?;

    if between_cb {
        let err_msg = format!("Error parsing template '{template}': missing '}}'");
        return Err(GeneralError::from_msg(err_msg.to_string()))
    }
    if escape {
        let err_msg = format!("Error parsing template '{template}': expecting character to escape after final '\\'");
        return Err(GeneralError::from_msg(err_msg.to_string()))
    }

    // add last part, if any
    if !current_str.is_empty() {
        template_string_parts.push((false, current_str.to_string()));
    }

    Ok(template_string_parts)
}

#[cfg(test)]
mod tests {
    use crate::function::template_string::parse_template;

    #[test]
    fn normal_template() {
        let result = parse_template("Hello {world}!").unwrap();
        let expected = vec![
            (false, "Hello ".to_string()),
            (true,  "world".to_string()),
            (false, "!".to_string()),
        ];
        assert_eq!(expected, result);
    }

    #[test]
    fn no_template_var() {
        let result = parse_template("Hello world!").unwrap();
        let expected = vec![
            (false, "Hello world!".to_string()),
        ];
        assert_eq!(expected, result);
    }

    #[test]
    fn two_template_vars() {
        let result = parse_template("{Hello}{world}!").unwrap();
        let expected = vec![
            (true,  "Hello".to_string()),
            (true,  "world".to_string()),
            (false,  "!".to_string()),
        ];
        assert_eq!(expected, result);
    }

    #[test]
    fn template_var_at_end() {
        let result = parse_template("{a}").unwrap();
        let expected = vec![(true,  "a".to_string())];
        assert_eq!(expected, result);
    }

    #[test]
    fn escaped_template() {
        let result = parse_template("Hello \\{world\\}!").unwrap();
        let expected = vec![
            (false, "Hello {world}!".to_string()),
        ];
        assert_eq!(expected, result);
    }

    #[test]
    fn nested_template_var() {
        let result = parse_template("Hello {{world}}!");
        assert!(result.is_err());
    }

    #[test]
    fn wrong_character_escaped() {
        let result = parse_template("Hello w\\orld!");
        assert!(result.is_err());
    }

    #[test]
    fn unclosed_template_var() {
        let result = parse_template("Hello {world!");
        assert!(result.is_err());
    }

    #[test]
    fn empty_template_var() {
        let result = parse_template("Hello {}!").unwrap();
        let expected = vec![
            (false, "Hello ".to_string()),
            (false, "!".to_string()),
        ];
        assert_eq!(expected, result);
    }

    #[test]
    fn empty_template() {
        let result = parse_template("").unwrap();
        let expected: Vec<(bool, String)> = Vec::new();
        assert_eq!(expected, result);
    }
}