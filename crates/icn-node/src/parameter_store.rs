use crate::config::NodeConfig;
use icn_common::CommonError;
use std::path::{Path, PathBuf};

#[derive(Clone)]
pub struct ParameterStore {
    path: PathBuf,
    config: NodeConfig,
}

impl ParameterStore {
    /// Load parameters from the given file path. If the file does not exist,
    /// defaults are used.
    pub fn load(path: PathBuf) -> Result<Self, CommonError> {
        let config = if path.exists() {
            NodeConfig::from_file(&path).map_err(|e| {
                CommonError::ConfigError(format!("Failed to load parameter file: {e}"))
            })?
        } else {
            NodeConfig::default()
        };
        Ok(Self { path, config })
    }

    pub fn open_rate_limit(&self) -> u64 {
        self.config.open_rate_limit
    }

    /// Update a parameter and persist changes to disk.
    pub fn set_parameter(&mut self, key: &str, value: &str) -> Result<(), CommonError> {
        match key {
            "open_rate_limit" => {
                let val = value
                    .parse::<u64>()
                    .map_err(|e| CommonError::InvalidInputError(e.to_string()))?;
                self.config.open_rate_limit = val;
                log::info!(target: "audit", "parameter_changed name=open_rate_limit value={}" , val);
                self.save()?;
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
