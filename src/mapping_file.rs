use std::io::{BufRead,BufReader};
use std::collections::HashMap;
use helpers::{open_file, match_ranges, extract_ranges};


#[derive(Debug)]
pub struct MappingFile {
    pub filename: String,
    pub delimiter: char,
    pub source_key_index: u64,
    pub target_key_index: u64,
    target_match_ranges: Vec<(u32, u32)>,
    preloaded_mappings: bool,
    mappings: HashMap<String, Vec<String>>,
}

impl MappingFile {
    pub fn new(filename: &str, delimiter: char, source_key_index: u64, target_key_index: u64, target_match_ranges: &str) -> MappingFile {
        MappingFile {
            filename: filename.to_owned(),
            delimiter: delimiter,
            source_key_index: source_key_index,
            target_key_index: target_key_index,
            target_match_ranges: match_ranges(target_match_ranges),
            preloaded_mappings: false,
            mappings: HashMap::new(),
        }
    }

    pub fn load_into_memory(&mut self) {
        for target_line in BufReader::new(open_file(&self.filename)).lines() {
            // Key is target_key_index
            if target_line.is_err() {
                error!("Unable to read line from mapping file {}", self.filename);
                break;
            }

            let target_line = target_line.unwrap();
            let target_key = target_line.split(self.delimiter).nth(self.target_key_index as usize).unwrap();

            // Value is the line broken into strings
            let line_pieces: Vec<String> = target_line.split(self.delimiter).map(String::from).collect();

            self.mappings.insert(String::from(target_key), line_pieces);
        }

        self.preloaded_mappings = true;
    }

    pub fn find_match(&self, source_key: &str) -> Option<Vec<String>> {
        if self.preloaded_mappings {
            match self.mappings.get(source_key) {
                Some(target_line) => {
                    let ref_target_line: Vec<&str> = target_line.iter().map(|s| &**s).collect();
                    Some(extract_ranges(&ref_target_line, &self.target_match_ranges))
                },
                None => None,
            }
        } else {
            // Open the file and scan through it on-demand
            for target_line in BufReader::new(open_file(&self.filename)).lines() {
                let target_line = target_line.expect(&format!("Unable to read line from mapping file {}", self.filename));
                let mut target_line_parts = target_line.split(self.delimiter);
                let target_key = target_line_parts.nth(self.target_key_index as usize)
                                                  .expect(&format!("Unable to parse the target key from mapping file {}", self.filename));

                if source_key == target_key {
                    let ref_target_line_parts: Vec<&str> = target_line_parts.collect();
                    return Some(extract_ranges(&ref_target_line_parts, &self.target_match_ranges))
                }
            };

            None
        }
    }
}
