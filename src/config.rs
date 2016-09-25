use yaml_rust::YamlLoader;
use std::path::PathBuf;
use std::error::Error;
use getopts::Options;
use std::fs::File;
use std::io::Read;
use std::process;
use env_logger;
use std::env;


#[derive(Debug)]
pub struct SourceFile {
    pub filename: String,
    pub delimiter: char,
}


#[derive(Debug)]
pub struct MappingFile {
    pub filename: String,
    pub delimiter: char,
    pub source_key_index: i64,
    pub target_key_index: i64,
    target_match_range: String,
}

impl MappingFile {
    pub fn match_range(&self) -> Vec<(u32, u32)> {
        fn parse_dash(source: &str) -> (u32, u32) {
            assert!(!source.contains(','));
            let bounds: Vec<u32> = source.split('-').map(|x| x.parse::<u32>().unwrap()).collect();
            assert_eq!(bounds.len(), 2);
            (bounds[0], bounds[1])
        }

        self.target_match_range.split(',').map(|range|
            if range.contains("-") {
                parse_dash(range)
            } else {
                // It's just a number
                let range = range.parse::<u32>().unwrap();
                (range, range)
            }
        ).collect()
    }
}

#[derive(Debug)]
pub struct Config {
    pub source_file: SourceFile,
    pub mapping_files: Vec<MappingFile>,
}


fn print_usage(program: &str, opts: &Options) {
    let usage = format!("\nUsage: {} [-h] [-v] -- See below for all options", program);
    println!("{}", opts.usage(&usage));
}

pub fn error_usage(message: &str, program: &str, opts: &Options) {
    error!("{}", message);
    print_usage(&program, &opts);
}

pub fn parse_args(args: Vec<String>) -> Result<Config, String> {
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

    fn parse_delimiter(foo: &str) -> char {
        let default_delimiter = ',';
        match foo.as_ref() {
            "tsv" => '\t',
            "csv" => ',',
            "psv" => '|',
            _ => default_delimiter,
        }
    }

    if let Some(doc) = config_file.get(0) {

        let source_delimiter = parse_delimiter(doc["source"]["delimiter"].as_str().unwrap());

        let source_file = SourceFile {
            filename: String::from(doc["source"]["filename"].as_str().unwrap()),
            delimiter: source_delimiter,
        };

        let mut mapping_files: Vec<MappingFile> = vec!();

        for i in 0 .. 100 {
            let mapping_file = &doc["mappings"][i];

            if mapping_file.is_badvalue() {
                // No more elements in the 'mappings' list to parse
                break;
            } else {
                mapping_files.push(MappingFile {
                    filename: String::from(mapping_file["filename"].as_str().unwrap()),
                    delimiter: parse_delimiter(mapping_file["delimiter"].as_str().unwrap()),
                    source_key_index: mapping_file["source-key-index"].as_i64().unwrap(),
                    target_key_index: mapping_file["target-key-index"].as_i64().unwrap(),
                    target_match_range: String::from(mapping_file["target-match-range"].as_str().unwrap()),
                });
            }
        }

        Ok(Config {
            source_file: source_file,
            mapping_files: mapping_files,
        })
    } else {
        Err(String::from("Incorrectly formatted YAML?"))
    }
}
