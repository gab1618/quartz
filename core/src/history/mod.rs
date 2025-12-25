use crate::history::error::HistoryError;
use crate::{QuartzResult, snippet};
use std::fmt::Display;
use std::io::Write;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

pub mod error;

#[derive(Serialize, Deserialize, Default)]
pub struct Entry {
    timestamp: i64,
    handle: String,

    /// List of exchanged HTTP messages
    messages: Vec<String>,
}

#[derive(Default)]
pub struct EntryBuilder {
    timestamp: i64,
    handle: Option<String>,
    messages: Vec<String>,
}

pub struct History {
    mount_path: PathBuf,
}

impl History {
    pub fn new(mount_path: PathBuf) -> QuartzResult<Self> {
        Ok(Self { mount_path })
    }

    pub fn entries(&self) -> QuartzResult<Vec<Entry>> {
        let paths = std::fs::read_dir(self.dir()).map_err(|_| HistoryError::ReadEntries)?;
        let mut timestamps = paths
            .map(|path| {
                let timestamp = path
                    .map_err(|_| HistoryError::ReadEntries)?
                    .file_name()
                    .to_str()
                    .ok_or(HistoryError::ReadEntries)?
                    .parse::<i64>()
                    .map_err(|_| HistoryError::ReadEntries)?;

                Ok(timestamp)
            })
            .collect::<QuartzResult<Vec<_>>>()?;

        timestamps.sort();
        timestamps.reverse();
        let entries = timestamps
            .iter()
            .filter_map(|timestamp| Entry::read(&self.dir().join(timestamp.to_string())).ok())
            .collect();

        Ok(entries)
    }

    pub fn dir(&self) -> PathBuf {
        self.mount_path.join("user").join("history")
    }

    pub fn last_entry(&self) -> QuartzResult<Option<Entry>> {
        let last_entry = self.entries()?.into_iter().next();

        Ok(last_entry)
    }

    pub fn write(&self, entry: Entry) -> QuartzResult {
        let content = toml::to_string(&entry).map_err(|_| HistoryError::Serialize)?;

        std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .open(self.dir().join(entry.timestamp.to_string()))
            .map_err(HistoryError::Save)?
            .write_all(content.as_bytes())
            .map_err(HistoryError::Save)?;

        Ok(())
    }
}

impl EntryBuilder {
    pub fn handle<T>(&mut self, value: T) -> &mut Self
    where
        T: Into<String>,
    {
        self.handle = Some(value.into());
        self
    }

    pub fn message<T>(&mut self, value: T) -> &mut Self
    where
        T: Into<snippet::Http>,
    {
        let m: snippet::Http = value.into();
        self.messages.push(m.to_string());
        self
    }

    pub fn message_raw(&mut self, value: String) -> &mut Self {
        self.messages.push(value);
        self
    }

    pub fn timestemp(&mut self, value: i64) -> &mut Self {
        self.timestamp = value;
        self
    }

    pub fn build(self) -> QuartzResult<Entry> {
        let handle = self.handle.ok_or(HistoryError::GetEntryBuilderHandle)?;

        if self.timestamp == 0 || self.messages.is_empty() {
            return Err(HistoryError::Empty.into());
        }

        Ok(Entry {
            handle,
            timestamp: self.timestamp,
            messages: self.messages,
        })
    }
}

impl Entry {
    pub fn builder() -> EntryBuilder {
        EntryBuilder::default()
    }

    pub fn handle(&self) -> &str {
        &self.handle
    }

    pub fn messages(&self) -> &Vec<String> {
        &self.messages
    }

    pub fn read(path: &Path) -> QuartzResult<Self> {
        let content = std::fs::read_to_string(path).map_err(|_| HistoryError::ReadEntry)?;

        Ok(toml::from_str(&content).map_err(|_| HistoryError::ParseEntry)?)
    }
}

impl Display for Entry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self.handle)?;
        write!(f, "{}", self.messages.join("\n"))?;

        Ok(())
    }
}
