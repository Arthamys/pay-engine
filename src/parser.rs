use super::error::Result;
use super::Transaction;
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

    pub fn next(&mut self) -> Result<Option<Transaction>> {
        let res = match self.reader.read_record(&mut self.record)? {
            true => Some(self.record.deserialize(None)?),
            false => None,
        };
        Ok(res)
    }
}
