/// FileMapper
///
/// Provided two or more files, this script will produce a single file output of a set of columns
/// across all the files provided. This is achieved by specifying a common column between a pair of files.
/// The pair of files will be the 'source' file and the 'mapping' file.
/// Currently we only support a 1:M relationship between source and mapping files.
///
/// We will then map each line in the source file to all other mapping files that have a 'match'.

#[macro_use] extern crate log;
extern crate env_logger;
extern crate yaml_rust;
extern crate getopts;
extern crate flate2;
extern crate bzip2;
extern crate csv;

use std::io::{BufRead,BufReader};
use std::env;

mod config;
use config::parse_args;

mod helpers;
use helpers::{open_file, build_new_line};

mod mapping_file;
mod source_file;


fn main() {
    let args: Vec<String> = env::args().collect();
    let mut config = parse_args(args).unwrap();
    let source_file = open_file(&config.source_file.filename);

    // Iterate over each line in the source file
    for source_line in BufReader::new(source_file).lines() {
        let source_line = source_line.expect("Failed reading the source file");
        let mut output: Vec<String> = vec!();

        // For each mapping file we're 'mapping', extract the source line's column and find it in the mapping file's column
        for mapping_file in config.mapping_files.iter_mut() {
            match source_line.split(config.source_file.delimiter).nth(mapping_file.source_key_index as usize) {
                Some(source_key) => {
                    match mapping_file.find_match(&source_key) {
                        Some(data) => {let mut data = data; output.append(&mut data)},
                        None => output.push(String::from(""))
                    }
                },
                None => {
                    error!("Unable to extract the source_key from line '{}' with index {}, skipping this mapping file", source_line, mapping_file.source_key_index);
                    continue;
                }
            }
        }

        let line = build_new_line(&source_line, config.source_file.delimiter, &mut output);
        println!("{}", line);
    }
}
