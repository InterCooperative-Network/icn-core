use icn_common::CommonError;
use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::marker::PhantomData;
use std::path::PathBuf;

pub trait EventStore<E>: Send + Sync
where
    E: Serialize + for<'de> Deserialize<'de> + Clone + Send + Sync,
{
    fn append(&mut self, event: &E) -> Result<(), CommonError>;
    fn query(&self, since: Option<usize>) -> Result<Vec<E>, CommonError>;
}

#[derive(Default)]
pub struct MemoryEventStore<E> {
    events: Vec<E>,
}

impl<E> MemoryEventStore<E> {
    pub fn new() -> Self {
        Self { events: Vec::new() }
    }
}

impl<E> EventStore<E> for MemoryEventStore<E>
where
    E: Serialize + for<'de> Deserialize<'de> + Clone + Send + Sync,
{
    fn append(&mut self, event: &E) -> Result<(), CommonError> {
        self.events.push(event.clone());
        Ok(())
    }

    fn query(&self, since: Option<usize>) -> Result<Vec<E>, CommonError> {
        let start = since.unwrap_or(0);
        Ok(self.events.iter().skip(start).cloned().collect())
    }
}

pub struct FileEventStore<E> {
    path: PathBuf,
    _marker: PhantomData<E>,
}

impl<E> FileEventStore<E> {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            _marker: PhantomData,
        }
    }
}

impl<E> EventStore<E> for FileEventStore<E>
where
    E: Serialize + for<'de> Deserialize<'de> + Clone + Send + Sync,
{
    fn append(&mut self, event: &E) -> Result<(), CommonError> {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
            .map_err(|e| CommonError::DatabaseError(format!("open: {e}")))?;
        let mut writer = BufWriter::new(file);
        let line = serde_json::to_string(event)
            .map_err(|e| CommonError::SerializationError(format!("{e}")))?;
        writer
            .write_all(line.as_bytes())
            .and_then(|_| writer.write_all(b"\n"))
            .map_err(|e| CommonError::DatabaseError(format!("write: {e}")))?;
        writer
            .flush()
            .map_err(|e| CommonError::DatabaseError(format!("flush: {e}")))?;
        Ok(())
    }

    fn query(&self, since: Option<usize>) -> Result<Vec<E>, CommonError> {
        if !self.path.exists() {
            return Ok(Vec::new());
        }
        let file = OpenOptions::new()
            .read(true)
            .open(&self.path)
            .map_err(|e| CommonError::DatabaseError(format!("open: {e}")))?;
        let reader = BufReader::new(file);
        let mut events = Vec::new();
        for (idx, line) in reader.lines().enumerate() {
            if let Some(start) = since {
                if idx < start {
                    continue;
                }
            }
            let line = line.map_err(|e| CommonError::DatabaseError(format!("read: {e}")))?;
            let evt: E = serde_json::from_str(&line)
                .map_err(|e| CommonError::DeserializationError(format!("{e}")))?;
            events.push(evt);
        }
        Ok(events)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
    struct TestEvent {
        value: u32,
    }

    #[test]
    fn memory_store_round_trip() {
        let mut store = MemoryEventStore::<TestEvent>::new();
        store.append(&TestEvent { value: 1 }).unwrap();
        store.append(&TestEvent { value: 2 }).unwrap();
        let events = store.query(None).unwrap();
        assert_eq!(events.len(), 2);
        assert_eq!(events[1].value, 2);
    }
}
