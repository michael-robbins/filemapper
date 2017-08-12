
use std::path::PathBuf;
use std::io::Read;
use std::fs::File;
use std::process;

// Optional decompressors for source/mapping files
use flate2::read::GzDecoder;
use bzip2::read::BzDecoder;


pub fn open_file(filename: &str) -> Box<Read> {
    let file_path = PathBuf::from(filename);
    let file = match File::open(&file_path) {
        Ok(file) => file,
        Err(_) => {
            error!("Unable to open file '{}'", file_path.display());
            process::exit(1);
        }
    };

    let decompressor: Box<Read> = match file_path.extension().unwrap_or_default().to_str() {
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

pub fn match_ranges(range_string: &str) -> Vec<(u32, u32)> {
    fn parse_dash(source: &str) -> (u32, u32) {
        assert!(!source.contains(','));
        let bounds: Vec<u32> = source.split('-')
                                     .map(|bound| bound.parse::<u32>()
                                                       .expect("Unable to process target-match-range bound into a number"))
                                     .collect();
        assert_eq!(bounds.len(), 2);
        (bounds[0], bounds[1])
    }

    range_string.split(',').map(|range|
        if range.contains('-') {
            parse_dash(range)
        } else {
            // It's just a number
            let range = range.parse::<u32>().unwrap();
            (range, range)
        }
    ).collect()
}

pub fn extract_ranges(line: &[&str], ranges: &[(u32, u32)]) -> Vec<String> {
    let mut extracted: Vec<String> = vec!();

    for range in ranges {
        if range.0 == range.1 {
            // Just extract a single element
            extracted.push(line[(range.0 - 1) as usize].to_owned());
        } else {
            for cell in line.iter().skip((range.0 - 1) as usize).take((1 + range.1 - range.0) as usize) {
                extracted.push(String::from(*cell))
            }
        }
    }

    extracted
}
