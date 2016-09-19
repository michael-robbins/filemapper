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
extern crate getopts;
extern crate flate2;
extern crate bzip2;
extern crate csv;

use std::path::PathBuf;
use getopts::Options;
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

    opts.optopt("", "source-file", "Path to the source file", "/path/to/source_file.csv");
    opts.optopt("", "mapping-file", "Path to mapping file with linkage on the end", "/path/to/mapping_file.tsv,0,0");

    opts.optopt("", "cache-policy", "Determines how we read the source/mapping files, relates to RAM usage", "[LIGHT|HEAVY]");

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
    let config_file;
    if matches.opt_present("config-file") {
        config_file = PathBuf::from(matches.opt_str("config-file").unwrap());
        debug!("We got a --config-file of: '{}'", config_file.display());
    } else {
        warn!("No config file loaded, only using command line args");
        config_file = PathBuf::new();
    }
    debug!("Using config-file: '{}'", config_file.display());

    // Parse --source-file parameter
    if ! matches.opt_present("source-file") {
        error_usage("We need a --source-file parameter", &program, &opts);
        process::exit(1);
    }

    let source_file = PathBuf::from(matches.opt_str("source-file").unwrap());
    debug!("We got a --source-file of: '{}'", source_file.display());

    // Parse each --mapping-file parameter
    // TODO: Support multiple matching columns, turning (u8, u8) -> Vec<(u8, u8)>
    let mut mapping_files: Vec<(PathBuf, (u8,u8))> = vec!();

    if matches.opt_present("mapping-file") {
        for mapping_filename in matches.opt_strs("mapping-file").iter() {
            debug!("We got --mapping-file: {:?}", mapping_filename);

            let mut mapping_filename_parts: Vec<&str> = mapping_filename.split(',').collect();

            let source_column;
            let mapping_column;

            match mapping_filename_parts.len() {
                3 => {
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

                    let element = (PathBuf::from(mapping_filename_parts.pop().unwrap()), (source_column.unwrap(), mapping_column.unwrap()));
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
    let source_file_ext = source_file.extension();
    let decompressor: Box<Read> = match source_file_ext.to_str() {
        Some("bz2") => {
            debug!("Using BzDecompressor as the input decompressor.");
            Box::new(BzDecoder::new(source_file))
        },
        Some("gz") => {
            debug!("Using GzDecoder as the input decompressor.");
            Box::new(GzDecoder::new(source_file).unwrap())
        },
        Some(_) => {
            debug!("Assuming the file is uncompressed.");
            Box::new(source_file)
        },
        None => {
            warn!("Unable to aquire file extention for {}", source_file);
            return Err(Error::new(ErrorKind::Other, format!("File extension invalid?")))
        },
    };
    // Iterate over each line in the source file
    // Read the line and extract the 'key' we will match on
    // For each matting file, start reading from the beginning of the file and keep going until you have a 'match'
    // When a match is found, take the column of interest and append it to the source file's line
    // TODO: Figure out how to read from a buffered reader and write to the underlying file?
    // TODO: Can we save the position of the open file, and map that position to a new 'column' of data to write before the line break?
}
