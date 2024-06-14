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

#[cfg(test)]
mod test_cases {
    use std::collections::HashSet;
    use std::fs;
    use std::fs::File;
    use std::io::{BufRead, BufReader, Error};
    use std::path::Path;
    use crate::start;
    use crate::mopper_options::MopperOptionsBuilder;

    fn exec(test_dir: &str) -> Result<(), Error> {
        let test_dir_path = Path::new(test_dir);
        let mapping_file = test_dir_path.join("mapping.json");
        let expected_output_file = test_dir_path.join("output.nq");
        let mopper_output_file = test_dir_path.join("output-mopper.nq");

        // set some options
        let options = MopperOptionsBuilder::default()
            .force_to_file(mopper_output_file.to_str().unwrap())
            .working_dir_hint(test_dir)
            .build().unwrap();

        // execute the plan
        let plan = fs::read_to_string(mapping_file).unwrap();
        let result = start(&plan, &options);
        assert!(result.is_ok());

        // compare results
        let expected_output = read_and_sort(expected_output_file)?;
        let mopper_output = read_and_sort(mopper_output_file)?;
        assert_eq!(expected_output, mopper_output);

        Ok(())
    }

    fn read_and_sort<P: AsRef<Path>>(file: P) -> Result<HashSet<String>, Error>{
        let file_handle = File::open(file)?;
        let rdr = BufReader::new(file_handle);
        
        let result: HashSet<String> = rdr.lines().flatten()
            .filter(|line| !line.trim().is_empty())
            .filter(|line| !line.trim().starts_with('#'))
            .map(|line| {
                        // remove repeated white space
                        let parts: Vec<_> = line.split_whitespace().collect();
                        parts.join(" ")
                    })
                    .collect();
        Ok(result)
    }

    #[test]
    fn rml_tc_0000_csv() -> Result<(), Error> {
        exec("test-resources/rml-testcases/RMLTC0000-CSV")?;
        Ok(())
    }

    #[test]
    fn rml_tc_0008b_csv() -> Result<(), Error> {
        exec("test-resources/rml-testcases/RMLTC0008b-CSV")?;
        Ok(())
    }
}
