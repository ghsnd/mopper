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

use std::fs;
use std::path::PathBuf;
use clap::Parser;
use log::info;
use mopper::mopper_options::MopperOptionsBuilder;
use mopper::start;

#[derive(Parser)]
struct Args {
    
    //#[options(help = "print help message")]
    //help: bool,

    /// The path to the AlgeMapLoom mapping plan (JSON).
    #[arg(short, long, value_name = "FILE")]
    mapping_file: String,
    
    /// Increase log level.
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,

    /// Be quiet; no logging.
    #[arg(short, long)]
    quiet: bool,
    
    /// Force output to standard out, ignoring the targets in the plan. Takes precedence over --force-to-file.
    #[arg(long)]
    force_std_out: bool,
    
    /// Force output to file, ignoring the targets in the plan.
    #[arg(long, value_name = "FILE")]
    force_to_file: Option<String>,

    /// Set the maximum number of messages each communication channel can hold before blocking the
    /// sender thread.
    /// `0` means no messages are hold: 'send' and 'receive' must happen at the same time.
    /// The default is `128`.
    #[arg(long, value_name = "N")]
    message_buffer_capacity: Option<usize>,

    /// Remove duplicate triples or quads. Note that currently deduplication only works on a per-sink basis and
    /// has a negative impact on speed and memory consumption.
    #[arg(short, long)]
    deduplicate: bool
}

fn main() {
    let args = Args::parse();
    
    // init logging
    stderrlog::new()
        .module(module_path!())
        .quiet(args.quiet)
        .timestamp(stderrlog::Timestamp::Second)
        .verbosity(args.verbose as usize)
        .init()
        .unwrap();

    // Read the execution plan
    info!("Reading mapping plan...");
    let path_to_plan_serialisation = &args.mapping_file;
    let json_plan = fs::read_to_string(path_to_plan_serialisation)
        .expect(format!("Mapping file not found: {}", args.mapping_file).as_str());
    let plan_ser_path = PathBuf::from(path_to_plan_serialisation);
    let mapping_parent_dir_option = plan_ser_path.parent();
    
    // set options
    let mut options_builder = MopperOptionsBuilder::default();
    if let Some(forced_output_file) = args.force_to_file {
        options_builder.force_to_file(forced_output_file);
    }
    options_builder
        .force_to_std_out(args.force_std_out)
        .deduplicate(args.deduplicate);
    if let Some(mapping_parent_dir) = mapping_parent_dir_option {
        let parent_dir = mapping_parent_dir.to_str().unwrap();
        if !parent_dir.is_empty() {
            options_builder.working_dir_hint(parent_dir);
        }
    }
    if let Some(buffer_capacity) = args.message_buffer_capacity {
        options_builder.message_buffer_capacity(buffer_capacity);
    }
    let options = options_builder.build().unwrap();
    
    if let Err(error) = start(&json_plan, &options) {
        eprintln!("{}", error);
    }
}