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
use crate::error::GeneralError;
use crate::util::remove_join_alias_prefix;

pub fn parse_template(template: &str, join_alias: &Option<String>) -> Result<Vec<(bool, String)>, GeneralError> {
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
                            let template_var_name = remove_join_alias_prefix(&current_str, join_alias);
                            template_string_parts.push((true, template_var_name));
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
    use crate::function::template_parser::parse_template;

    #[test]
    fn normal_template() {
        let result = parse_template("Hello {world}!", &None).unwrap();
        let expected = vec![
            (false, "Hello ".to_string()),
            (true,  "world".to_string()),
            (false, "!".to_string()),
        ];
        assert_eq!(expected, result);
    }

    #[test]
    fn no_template_var() {
        let result = parse_template("Hello world!", &None).unwrap();
        let expected = vec![
            (false, "Hello world!".to_string()),
        ];
        assert_eq!(expected, result);
    }

    #[test]
    fn two_template_vars() {
        let result = parse_template("{Hello}{world}!", &None).unwrap();
        let expected = vec![
            (true,  "Hello".to_string()),
            (true,  "world".to_string()),
            (false,  "!".to_string()),
        ];
        assert_eq!(expected, result);
    }

    #[test]
    fn template_var_at_end() {
        let result = parse_template("{a}", &None).unwrap();
        let expected = vec![(true,  "a".to_string())];
        assert_eq!(expected, result);
    }

    #[test]
    fn escaped_template() {
        let result = parse_template("Hello \\{world\\}!", &None).unwrap();
        let expected = vec![
            (false, "Hello {world}!".to_string()),
        ];
        assert_eq!(expected, result);
    }

    #[test]
    fn nested_template_var() {
        let result = parse_template("Hello {{world}}!", &None);
        assert!(result.is_err());
    }

    #[test]
    fn wrong_character_escaped() {
        let result = parse_template("Hello w\\orld!", &None);
        assert!(result.is_err());
    }

    #[test]
    fn unclosed_template_var() {
        let result = parse_template("Hello {world!", &None);
        assert!(result.is_err());
    }

    #[test]
    fn empty_template_var() {
        let result = parse_template("Hello {}!", &None).unwrap();
        let expected = vec![
            (false, "Hello ".to_string()),
            (false, "!".to_string()),
        ];
        assert_eq!(expected, result);
    }

    #[test]
    fn empty_template() {
        let result = parse_template("", &None).unwrap();
        let expected: Vec<(bool, String)> = Vec::new();
        assert_eq!(expected, result);
    }
}