use std::io::{BufRead,BufReader};
use std::collections::HashMap;
use helpers::open_file;


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
            target_match_ranges: MappingFile::match_ranges(target_match_ranges),
            preloaded_mappings: false,
            mappings: HashMap::new()
        }
    }

    pub fn match_ranges(range_string: &str) -> Vec<(u32, u32)> {
        fn parse_dash(source: &str) -> (u32, u32) {
            assert!(!source.contains(','));
            let bounds: Vec<u32> = source.split('-').map(|x| x.parse::<u32>().unwrap()).collect();
            assert_eq!(bounds.len(), 2);
            (bounds[0], bounds[1])
        }

        range_string.split(',').map(|range|
            if range.contains("-") {
                parse_dash(range)
            } else {
                // It's just a number
                let range = range.parse::<u32>().unwrap();
                (range, range)
            }
        ).collect()
    }

    fn extract_ranges(&self, line: &Vec<String>) -> Option<Vec<String>> {
        let mut ranges: Vec<String> = vec!();

        for range in self.target_match_ranges.iter() {
            if range.0 == range.1 {
                let cell = line.iter().nth(range.0 as usize).unwrap();
                ranges.push(cell.to_owned());
            } else {
                for cell in line.iter().skip(range.0 as usize).take((range.1 - range.0 + 1) as usize) {
                    ranges.push(cell.to_owned())
                }
            }
        }

        if ranges.len() > 0 {
            Some(ranges)
        } else {
            None
        }
    }

    pub fn find_match(&self, source_key: &str) -> Option<Vec<String>> {
        if self.preloaded_mappings {
            match self.mappings.get(source_key) {
                Some(target_line) => {
                    self.extract_ranges(target_line)
                },
                None => None
            }
        } else {
            // Open the file and scan through it on-demand
            for target_line in BufReader::new(open_file(&self.filename)).lines() {
                if target_line.is_err() {
                    error!("Unable to read line from mapping file {}", self.filename);
                    break;
                }

                let target_line = target_line.unwrap();
                let target_key = target_line.split(self.delimiter).nth(self.target_key_index as usize).unwrap();

                if source_key == target_key {
                    for range in self.target_match_ranges.iter() {
                        if range.0 == range.1 {
                            let cell = target_line.split(self.delimiter).nth(range.0 as usize).unwrap().to_owned();
                            return Some(vec!(cell))
                        } else {
                            return Some(target_line.split(self.delimiter)
                                                   .skip(range.0 as usize)
                                                   .take((range.1 - range.0 + 1) as usize)
                                                   .map(|x| String::from(x))
                                                   .collect())
                        }
                    }

                    // We've found our match, don't keep looking
                    break;
                }
            };

            None
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
            let line_pieces: Vec<String> = target_line.split(self.delimiter).map(|x| String::from(x)).collect();

            self.mappings.insert(String::from(target_key), line_pieces);
        }

        self.preloaded_mappings = true;
    }
}
