pub use self::buckets::{DataReader, DataReaderBuilder};
pub use self::splittimes::{SplitTimeReader, SplitTimeReaderBuilder};
pub use self::times::TimeReaderBuilder;

mod buckets;
mod dateparser;
mod splittimes;
mod times;

use std::fs::File;
use std::io::{self, BufReader};

/// Return `io::BufRead` from a path, falling back to using stdin if path is "-".
/// Exits the program with exit code 1 if path does not exist.
fn open_file(path: &str) -> Box<dyn io::BufRead> {
    match path {
        "-" => Box::new(BufReader::new(io::stdin())),
        _ => match File::open(path) {
            Ok(fd) => Box::new(io::BufReader::new(fd)),
            Err(error) => {
                error!("Could not open {}: {}", path, error);
                panic!("{}", error);
            }
        },
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    #[should_panic]
    fn test_bad_file() {
        open_file("/no/good");
    }
}
