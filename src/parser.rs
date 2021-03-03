use crate::transaction::Transaction;
use crate::Result;
use csv::{Reader, ReaderBuilder};

pub struct Parser {
    reader: Reader<std::fs::File>,
    record: csv::StringRecord,
}

impl Parser {
    pub fn new(file_path: &str) -> Result<Parser> {
        let rdr = ReaderBuilder::new()
            .flexible(true)
            .trim(csv::Trim::All)
            .from_path(file_path)?;
        Ok(Parser {
            reader: rdr,
            record: csv::StringRecord::new(),
        })
    }
}

impl Iterator for Parser {
    type Item = Transaction;

    fn next(&mut self) -> Option<Self::Item> {
        match self.reader.read_record(&mut self.record).unwrap_or(false) {
            true => self.record.deserialize(None).ok(),
            false => None,
        }
    }
}
