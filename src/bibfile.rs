use std::collections::HashMap;
use std::error::Error as ErrorTrait;
use std::fmt;
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::parser::bibfile;

pub type BibFile = Vec<BibItem>;

#[derive(Debug)]
pub struct BibError;

impl fmt::Display for BibError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "An error occurred whilst parsing the file.")
    }
}

impl ErrorTrait for BibError {
    fn description(&self) -> &str {
        "An error occurred whilst parsing the file."
    }
}

impl From<std::io::Error> for BibError {
    fn from(_err: std::io::Error) -> BibError {
        BibError
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum BibItem {
    String(HashMap<String, String>),
    Preamble,
    Comment,
    Entry {
        entry_type: String,
        label: String,
        tags: HashMap<String, String>,
    },
}

impl fmt::Display for BibItem {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use BibItem::*;
        match self {
            Entry {
                entry_type,
                label,
                tags,
            } => {
                writeln!(f, "@{}{{{},", entry_type, label)?;
                for (k, v) in tags.iter() {
                    writeln!(f, "    {} = {{{}}},", k, v)?;
                }
                write!(f, "}}\n\n")?;
            }
            Preamble => {
                write!(f, "Preamble")?;
            }
            Comment => {
                write!(f, "Comment")?;
            }
            String(_) => {
                write!(f, "String")?;
            }
        }
        Ok(())
    }
}

impl BibItem {
    pub fn load(path: &Path) -> Result<BibFile, BibError> {
        let file_string = fs::read_to_string(path)?;
        match bibfile(&file_string) {
            Ok((_, file)) => Ok(file),
            Err(_) => Err(BibError),
        }
    }
}
