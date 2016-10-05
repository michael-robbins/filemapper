
use std::path::PathBuf;
use std::ffi::OsStr;
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

pub fn build_new_line(line: &str, delimiter: char, extra: &mut Vec<String>) -> String{
    if extra.len() > 0 {
        // Turn the &str into a vector of strings, so we can append 'extra'
        let mut new_line: Vec<String> = line.split(delimiter).map(|x| String::from(x)).collect();
        new_line.append(extra);

        let new_line = new_line.iter().fold(String::new(), |acc, element|
            if acc == "" {
                element.to_owned()
            } else {
                format!("{}{}{}",acc, delimiter, element)
            }
        );

        new_line
    } else {
        // Just emit the current line
        line.to_owned()
    }
}
