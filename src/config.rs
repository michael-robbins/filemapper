use yaml_rust::{Yaml, YamlLoader};
use std::error::Error;
use getopts::Options;
use std::fs::File;
use std::io::Read;
use std::process;
use env_logger;
use std::env;

use mapping_file::MappingFile;
use source_file::SourceFile;


#[derive(Debug)]
pub struct Config {
    pub source_file: SourceFile,
    pub mapping_files: Vec<MappingFile>,
}

impl Config {
    fn parse_yaml(string: String) -> Option<Yaml> {
        let yaml = match YamlLoader::load_from_str(&string) {
            Ok(contents) => contents,
            Err(err) => panic!("Couldn't read config file: {}", Error::description(&err)),
        };

        match yaml.get(0) {
            Some(doc) => Some(doc.to_owned()),
            None => None
        }
    }

    fn new(config_filename: &str) -> Result<Config, String> {
        // Load & parse the config file
        let mut config_file_contents = String::new();

        let read_result = File::open(config_filename).and_then(
            |file| {let mut file = file; file.read_to_string(&mut config_file_contents)}
        );

        if read_result.is_err() {
            return Err(String::from("Failed"));
        }

        let doc = match Config::parse_yaml(config_file_contents) {
            Some(doc) => doc,
            None => return Err(String::from("Failed to parse YAML config file")),
        };

        let source_delimiter = parse_delimiter(doc["source"]["delimiter"].as_str().unwrap());

        let source_file = SourceFile {
            filename: String::from(doc["source"]["filename"].as_str().unwrap()),
            delimiter: source_delimiter,
        };

        let mut mapping_files: Vec<MappingFile> = vec!();

        match doc["mappings"].as_vec() {
            Some(config_mapping_files) => {
                for mapping_file in config_mapping_files {
                    let mut mapping_instance = MappingFile::new(
                        mapping_file["filename"].as_str().unwrap(),
                        parse_delimiter(mapping_file["delimiter"].as_str().unwrap()),
                        mapping_file["source-key-index"].as_i64().unwrap() as u64,
                        mapping_file["target-key-index"].as_i64().unwrap() as u64,
                        mapping_file["target-match-range"].as_str().unwrap(),
                    );

                    if mapping_file["in-memory"].as_bool().unwrap() {
                        mapping_instance.load_into_memory();
                    }

                    mapping_files.push(mapping_instance);
                }
            },
            None => return Err(String::from("Unable to parse config file correctly."))
        }

        Ok(Config {
            source_file: source_file,
            mapping_files: mapping_files,
        })
    }
}


fn print_usage(program: &str, opts: &Options) {
    let usage = format!("\nUsage: {} [-h] [-v] -- See below for all options", program);
    println!("{}", opts.usage(&usage));
}

pub fn error_usage(message: &str, program: &str, opts: &Options) {
    error!("{}", message);
    print_usage(&program, &opts);
}

fn parse_delimiter(foo: &str) -> char {
    let default_delimiter = ',';
    match foo.as_ref() {
        "tsv" => '\t',
        "csv" => ',',
        "psv" => '|',
        _ => default_delimiter,
    }
}

pub fn parse_args(args: Vec<String>) -> Result<Config, String> {
    let program = args[0].clone();

    // General options
    let mut opts = Options::new();
    opts.optflag("h", "help", "Print out this help.");
    opts.optflagmulti("v", "verbose", "Prints out more info (able to be applied up to 3 times)");
    opts.optopt("", "config-file", "Configuration file in YAML that contains most other settings", "/path/to/config.yaml");

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
        config_filename = matches.opt_str("config-file").unwrap();
        debug!("We got a --config-file of: '{}'", config_filename);
    } else {
        error_usage("We need a --config-file parameter", &program, &opts);
        process::exit(1);
    }

    Config::new(&config_filename)
}
