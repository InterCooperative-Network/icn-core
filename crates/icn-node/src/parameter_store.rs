use crate::config::NodeConfig;
use icn_common::CommonError;
use icn_eventstore::{EventStore, FileEventStore, ParameterUpdate};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

pub struct ParameterStore {
    path: PathBuf,
    config: NodeConfig,
    #[allow(clippy::type_complexity)]
    event_store: Option<Mutex<Box<dyn EventStore<ParameterUpdate>>>>,
}

impl ParameterStore {
    /// Load parameters from the given file path. If the file does not exist,
    /// defaults are used.
    pub fn load(path: PathBuf) -> Result<Self, CommonError> {
        let events_path = path.with_extension("events.jsonl");
        let mut store = FileEventStore::new(events_path.clone());
        let events = store.query(None)?;
        let mut config = if path.exists() {
            NodeConfig::from_file(&path).map_err(|e| {
                CommonError::ConfigError(format!("Failed to load parameter file: {e}"))
            })?
        } else {
            NodeConfig::default()
        };
        for ev in &events {
            match ev.name.as_str() {
                "open_rate_limit" => {
                    let val = ev
                        .value
                        .parse::<u64>()
                        .map_err(|e| CommonError::InvalidInputError(e.to_string()))?;
                    config.http.open_rate_limit = val;
                }
                _ => {}
            }
        }
        Ok(Self {
            path,
            config,
            event_store: Some(Mutex::new(Box::new(store))),
        })
    }

    pub fn open_rate_limit(&self) -> u64 {
        self.config.http.open_rate_limit
    }

    /// Update a parameter and persist changes to disk.
    pub fn set_parameter(&mut self, key: &str, value: &str) -> Result<(), CommonError> {
        match key {
            "open_rate_limit" => {
                let val = value
                    .parse::<u64>()
                    .map_err(|e| CommonError::InvalidInputError(e.to_string()))?;
                self.config.http.open_rate_limit = val;
                log::info!(target: "audit", "parameter_changed name=open_rate_limit value={}" , val);
                self.save()?;
                if let Some(store) = &self.event_store {
                    store.lock().unwrap().append(&ParameterUpdate {
                        name: key.to_string(),
                        value: value.to_string(),
                    })?;
                }
                Ok(())
            }
            _ => Err(CommonError::InvalidInputError(format!(
                "Unknown parameter {key}"
            ))),
        }
    }

    pub fn save(&self) -> Result<(), CommonError> {
        self.config
            .save_to_file(&self.path)
            .map_err(|e| CommonError::IoError(e.to_string()))
    }
}

impl ParameterStore {
    pub fn path(&self) -> &Path {
        &self.path
    }
}
