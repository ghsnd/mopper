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
use clap::Parser;
use log::info;
use mopper::start;

#[derive(Parser)]
struct Args {
    
    //#[options(help = "print help message")]
    //help: bool,

    /// the path to the AlgeMapLoom mapping plan (JSON)
    #[arg(short, long, value_name = "FILE")]
    mapping_file: String,
    
    /// increase log level
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,

    /// be quiet; no logging
    #[arg(short, long)]
    quiet: bool,
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
    let json_plan = fs::read_to_string(path_to_plan_serialisation).unwrap();
    start(&json_plan);
}