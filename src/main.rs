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
use yaml_rust::YamlLoader;
use std::path::PathBuf;
use std::error::Error;
use getopts::Options;
use std::fs::File;
use std::process;
use std::env;

// Optional decompressors for source/mapping files
use flate2::read::GzDecoder;
use bzip2::read::BzDecoder;

fn print_usage(program: &str, opts: &Options) {
    let usage = format!("\nUsage: {} [-h] [-v] -- See below for all options", program);
    println!("{}", opts.usage(&usage));
}

fn error_usage(message: &str, program: &str, opts: &Options) {
    error!("{}", message);
    print_usage(&program, &opts);
}

fn main() {
    let args = env::args().collect::<Vec<String>>();
    let program = args[0].clone();


    // General options
    let mut opts = Options::new();
    opts.optflag("h", "help", "Print out this help.");
    opts.optflagmulti("v", "verbose", "Prints out more info (able to be applied up to 3 times)");
    opts.optopt("", "config-file", "Configuration file in YAML that contains most other settings", "/path/to/config.yaml");
    opts.optopt("", "cache-policy", "HEAVY means we will cache the mapping-file in RAM\nLIGHT means we will not", "[LIGHT|HEAVY]");

    // Parse the user provided parameters matching them to the options specified above
    let matches = match opts.parse(args) {
        Ok(matches) => matches,
        Err(failure) => panic!(failure.to_string()),
    };

    // Check if the 'h' flag was present, print the usage if it was, then exit
    if matches.opt_present("h") {
        print_usage(&program, &opts);
    }

    // Configure logging verbosity and initialise the logger
    match matches.opt_count("v") {
        0 => {env::set_var("RUST_LOG", "warn")},
        1 => {env::set_var("RUST_LOG", "info")},
        2 => {env::set_var("RUST_LOG", "debug")},
        _ => {env::set_var("RUST_LOG", "trace")}, // Provided > 2 -v flags
    }

    env_logger::init().unwrap();

    debug!("Applied log level: {}", env::var("RUST_LOG").unwrap());

    // Parse --config-file parameter
    let config_filename;
    if matches.opt_present("config-file") {
        config_filename = PathBuf::from(matches.opt_str("config-file").unwrap());
        debug!("We got a --config-file of: '{}'", config_filename.display());
    } else {
        error_usage("We need a --config-file parameter", &program, &opts);
        process::exit(1);
    }

    debug!("Using config-file: '{}'", config_filename.display());

    // Load & parse the config file
    let mut config_file_contents = String::new();

    if let Err(err) = File::open(config_filename).unwrap().read_to_string(&mut config_file_contents) {
        panic!("Couldn't read config file: {}", Error::description(&err))
    }

    let config_file = match YamlLoader::load_from_str(&config_file_contents) {
        Ok(yaml) => yaml,
        Err(err) => panic!("Couldn't read config file: {}", Error::description(&err)),
    };

    // Example config file:
    /*
    source:
        filename: test.tsv
        delimiter: \t
    mappings:
        - mapping-file-1:
            filename: mapping-1.tsv
            delimiter: \t
            source-key-index: 0
            target-key-index: 0
            target-match-range: 1-2
        - mapping-file-2:
            filename: mapping-2.csv
            delimiter: ,
            source-key-index: 0
            target-key-index: 1
            target-match-range: 0,2
    */

    #[derive(Debug)]
    struct SourceFile<'a> {
        filename: &'a str,
        delimiter: char,
    }

    #[derive(Debug)]
    struct MappingFile {
        filename: String,
        delimiter: char,
        source_key_index: u8,
        target_key_index: u8,
        target_match_range: String,
    }

    #[derive(Debug)]
    struct Config<'a> {
        source_file: SourceFile<'a>,
        mapping_files: Vec<MappingFile>,
    }

    if let Some(doc) = config_file.get(0) {
        let default_delimiter = ',';
        let source_delimiter: char = match doc["source"]["delimiter"].as_str() {
            Some("tsv") => '\t',
            Some("csv") => ',',
            Some("psv") => '|',
            Some(_) => default_delimiter,
            None => default_delimiter,
        };

        let source_file = SourceFile {
            filename: doc["source"]["filename"].as_str().unwrap(),
            delimiter: source_delimiter,
        };

        println!("{:?}", source_file);

        let mapping_files: Vec<MappingFile> = vec!();

        println!("{:?}", &doc["mappings"].into_iter());

        /*for mapping_file in doc["mappings"][0] {
            println!("{:?}", mapping_file);
        }*/
    }

    /*
    // Parse each --mapping-file parameter
    // TODO: Support multiple matching columns, turning (u8, u8) -> Vec<(u8, u8)>
    let mut mapping_files: Vec<(File, (u8,u8))> = vec!();

    if matches.opt_present("mapping-file") {
        for mapping_filename in matches.opt_strs("mapping-file").iter() {
            debug!("We got --mapping-file: {:?}", mapping_filename);

            let mut mapping_filename_parts: Vec<&str> = mapping_filename.split(',').collect();

            let source_column;
            let mapping_column;
            let mapping_target;

            match mapping_filename_parts.len() {
                4 => {
                    mapping_target = mapping_filename_parts.pop().unwrap();
                    // Determine if a '-' is in the string, if so, treat it as a range, otherise a u8


                    mapping_column = mapping_filename_parts.pop().unwrap().parse::<u8>();
                    if mapping_column.is_err() {
                        let message = format!("Mapping column for filename {} failed to parse into an u8", mapping_filename);
                        error_usage(message.as_ref(), &program, &opts);
                        process::exit(1);
                    }

                    source_column = mapping_filename_parts.pop().unwrap().parse::<u8>();
                    if source_column.is_err() {
                        let message = format!("Source column for filename {} failed to parse into an u8", mapping_filename);
                        error_usage(message.as_ref(), &program, &opts);
                        process::exit(1);
                    }

                    let element = (File::open(PathBuf::from(mapping_filename_parts.pop().unwrap())).unwrap(), (source_column.unwrap(), mapping_column.unwrap()));
                    mapping_files.push(element);
                },
                _ => {
                    let message = format!("The following --mapping-file parmeter is not structured correctly\nParameter: {}", mapping_filename);
                    error_usage(message.as_ref(), &program, &opts);
                    process::exit(1);
                }
            }
        }
    } else {
        error_usage("We need at least one --mapping-file", &program, &opts);
        process::exit(1);
    }

    info!("{:?}", mapping_files);

    // Figure out the input file's decompressor
    let source_file_ext = source_filename.extension().unwrap();
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
        for mapping_data in mapping_files.iter_mut() {
            let mapping_file = &mapping_data.0;
            let (source_index, mapping_index) = mapping_data.1;
            let source_key = line.split(source_delimiter_char).nth(source_index as usize).unwrap();
            println!("{}", source_key);
        }
    }
    // For each matting file, start reading from the beginning of the file and keep going until you have a 'match'
    // When a match is found, take the column of interest and append it to the source file's line
    // TODO: Figure out how to read from a buffered reader and write to the underlying file?
    // TODO: Can we save the position of the open file, and map that position to a new 'column' of data to write before the line break?

    */
}
