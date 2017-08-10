use helpers::match_ranges;

#[derive(Debug)]
pub struct SourceFile {
    pub filename: String,
    pub delimiter: char,
    pub target_match_ranges: Vec<(u32, u32)>,
}

impl SourceFile {
    pub fn new(filename: String, delimiter: char, target_match_ranges: String) -> SourceFile {
        SourceFile {
            filename: filename.to_owned(),
            delimiter: delimiter,
            target_match_ranges: match_ranges(&target_match_ranges),
        }
    }
}
