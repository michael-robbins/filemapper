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
use std::fs::File;
use std::env;



// Optional decompressors for source/mapping files
use flate2::read::GzDecoder;
use bzip2::read::BzDecoder;

mod config;
use config::parse_args;

fn main() {
    let args = env::args().collect::<Vec<String>>();
    let mut config = parse_args(args).unwrap();

    // Figure out the input file's decompressor
    let source_file_path = PathBuf::from(config.source_file.filename);
    let source_file_ext = source_file_path.extension().unwrap();
    let source_file = File::open(&source_file_path).unwrap();

    let decompressor: Box<Read> = match source_file_ext.to_str() {
        Some("bz2") => {
            debug!("Using BzDecompressor as the input decompressor.");
            Box::new(BzDecoder::new(source_file))
        },
        Some("gz") => {
            debug!("Using GzDecoder as the input decompressor.");
            Box::new(GzDecoder::new(source_file).unwrap())
        },
        Some(_) | None => {
            debug!("Assuming the file is uncompressed.");
            Box::new(source_file)
        },
    };

    // Iterate over each line in the source file
    let source_lines = BufReader::new(decompressor).lines();
    for line in source_lines {
        let line = line.unwrap();
        println!("{}", line);

        // For each mapping file we're 'mapping', extract the source line's column and find it in the mapping file's column
        for mapping_data in config.mapping_files.iter_mut() {
            let mapping_file_path = PathBuf::from(&mapping_data.filename);
            let mapping_file = File::open(mapping_file_path).unwrap();
            let target_key_index = mapping_data.target_key_index;
            let source_key = line.split(config.source_file.delimiter).nth(mapping_data.source_key_index as usize).unwrap();

            let target_ranges = mapping_data.match_range();
            println!("{}", source_key);
            println!("{}", target_key_index);
            println!("{:?}", mapping_file);
            println!("{:?}", target_ranges);
        }
    }

    // For each matting file, start reading from the beginning of the file and keep going until you have a 'match'
    // When a match is found, take the column of interest and append it to the source file's line
    // TODO: Figure out how to read from a buffered reader and write to the underlying file?
    // TODO: Can we save the position of the open file, and map that position to a new 'column' of data to write before the line break?
}
