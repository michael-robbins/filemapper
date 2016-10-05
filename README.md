# filemapper

File Mapper will take a source file and multiple mapping files, and given a common column between the source file and each mapping file, produce a merged output.

The user can provide individual column indexes per mapping file and select which columns from the mapping file(s) they with to emit along with the source file.

## Features
* Ability to take an arbitrary number of mapping files, each with their own common index pair
* Ability to output any number of individual (or ranges of) columns in the mapping file(s)
* Will be able to store mapping files in-memory (fast) or look up mapping files on-demand (slow)

## Installation
### From source (assuming you have Rust & Cargo installed)
1. Clone the repository: ```git clone https://github.com/michael-robbins/filemapper.git```

2. ```cd filemapper; cargo build --release```. The binary will now be in ```./target/release/filemapper```

3. Done! Test it out by using the supplied test data!

## Usage
  Usage: target\debug\file_mapper.exe [-h] [-v] -- See below for all options

  Options:
      -h, --help          Print out this help.
      -v, --verbose       Prints out more info (able to be applied up to 3
                          times)
          --config-file /path/to/config.yaml
                          Configuration file in YAML that contains most other
                          settings
