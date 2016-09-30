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

use std::io::{Read,BufReader,BufRead};
use std::path::PathBuf;
use std::ffi::OsStr;
use std::fs::File;
use std::process;
use std::env;

// Optional decompressors for source/mapping files
use flate2::read::GzDecoder;
use bzip2::read::BzDecoder;

mod config;
use config::parse_args;



fn open_file(filename: &str) -> Box<Read> {
    let file_path = PathBuf::from(filename);
    let file = match File::open(&file_path) {
        Ok(file) => file,
        Err(_) => {
            error!("Unable to open file '{}'", file_path.display());
            process::exit(1);
        }
    };

    let decompressor: Box<Read> = match file_path.extension().unwrap_or(OsStr::new("")).to_str() {
        Some("bz2") => {
            debug!("Using BzDecompressor as the input decompressor.");
            Box::new(BzDecoder::new(file))
        },
        Some("gz") => {
            debug!("Using GzDecoder as the input decompressor.");
            Box::new(GzDecoder::new(file).unwrap())
        },
        Some(_) | None => {
            debug!("Assuming the file is uncompressed.");
            Box::new(file)
        },
    };

    decompressor
}

fn main() {
    let args = env::args().collect::<Vec<String>>();
    let mut config = parse_args(args).unwrap();
    let source_file = open_file(&config.source_file.filename);

    // Iterate over each line in the source file
    for source_line in BufReader::new(source_file).lines() {
        let source_line = source_line.unwrap();
        let mut output: Vec<String> = vec!();

        // For each mapping file we're 'mapping', extract the source line's column and find it in the mapping file's column
        for mapping_data in config.mapping_files.iter_mut() {
            let source_key = source_line.split(config.source_file.delimiter).nth(mapping_data.source_key_index as usize);

            if source_key.is_none() {
                error!("Unable to extract the source_key from line '{}' with index {}, skipping this mapping file", source_line, mapping_data.source_key_index);
                continue;
            }

            let source_key = source_key.unwrap();

            for target_line in BufReader::new(open_file(&mapping_data.filename)).lines() {
                if target_line.is_err() {
                    error!("Unable to read line from mapping file {}", mapping_data.filename);
                    break;
                }

                let target_line = target_line.unwrap();
                let target_key = target_line.split(mapping_data.delimiter).nth(mapping_data.target_key_index as usize).unwrap();

                if source_key == target_key {
                    for range in mapping_data.match_range().iter() {
                        if range.0 == range.1 {
                            let cell = target_line.split(mapping_data.delimiter).nth(range.0 as usize).unwrap().to_owned();
                            output.push(cell);
                        } else {
                            for cell in target_line.split(mapping_data.delimiter).skip(range.0 as usize).take((range.1 - range.0 + 1) as usize) {
                                output.push(cell.to_owned());
                            }
                        }
                    }

                    // We've found our match, don't keep looking
                    break;
                }
            }
        }

        if output.len() > 0 {
            // Build a new line to emit, with the output vec appended to the end
            let mut new_line: Vec<String> = source_line.split(config.source_file.delimiter).map(|x| String::from(x)).collect();
            new_line.append(&mut output);

            let new_line = new_line.iter().fold(String::new(), |acc, element|
                if acc == "" {
                    element.to_owned()
                } else {
                    format!("{}{}{}",acc, config.source_file.delimiter, element)
                }
            );

            println!("{}", new_line);
        } else {
            // Just emit the current line
            println!("{}", source_line);
        }
    }
}
