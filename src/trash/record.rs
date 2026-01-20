//! Trash records - tracking deleted items for recovery

use crate::error::{DevSweepError, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Record of a trashed item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrashRecord {
    /// Unique identifier
    pub id: String,
    /// Original path before deletion
    pub original_path: PathBuf,
    /// Size in bytes
    pub size: u64,
    /// When it was deleted
    pub deleted_at: DateTime<Utc>,
    /// Project name it belonged to
    pub project_name: String,
    /// Type of artifact
    pub artifact_kind: String,
    /// Delete method used
    pub method: String,
}

impl TrashRecord {
    /// Create a new trash record
    pub fn new(
        original_path: PathBuf,
        size: u64,
        project_name: impl Into<String>,
        artifact_kind: impl Into<String>,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            original_path,
            size,
            deleted_at: Utc::now(),
            project_name: project_name.into(),
            artifact_kind: artifact_kind.into(),
            method: "trash".into(),
        }
    }
}

/// Storage for trash records
pub struct TrashRecordStore {
    records_path: PathBuf,
}

impl TrashRecordStore {
    /// Create a new record store
    pub fn new() -> Result<Self> {
        let records_path = dirs::data_dir()
            .ok_or_else(|| DevSweepError::Trash("Cannot find data directory".into()))?
            .join("devsweep")
            .join("trash_records.json");

        // Ensure directory exists
        if let Some(parent) = records_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        Ok(Self { records_path })
    }

    /// Load all records
    pub fn load(&self) -> Result<Vec<TrashRecord>> {
        if !self.records_path.exists() {
            return Ok(Vec::new());
        }

        let content = std::fs::read_to_string(&self.records_path)?;
        let records: Vec<TrashRecord> = serde_json::from_str(&content)?;
        Ok(records)
    }

    /// Save all records
    pub fn save(&self, records: &[TrashRecord]) -> Result<()> {
        let content = serde_json::to_string_pretty(records)?;
        std::fs::write(&self.records_path, content)?;
        Ok(())
    }

    /// Add a record
    pub fn add(&self, record: TrashRecord) -> Result<()> {
        let mut records = self.load()?;
        records.push(record);
        self.save(&records)
    }

    /// Remove a record by ID
    pub fn remove(&self, id: &str) -> Result<Option<TrashRecord>> {
        let mut records = self.load()?;
        let pos = records.iter().position(|r| r.id == id);

        if let Some(pos) = pos {
            let removed = records.remove(pos);
            self.save(&records)?;
            Ok(Some(removed))
        } else {
            Ok(None)
        }
    }

    /// Clear all records
    pub fn clear(&self) -> Result<()> {
        self.save(&[])
    }

    /// Get total size of tracked trash
    pub fn total_size(&self) -> Result<u64> {
        let records = self.load()?;
        Ok(records.iter().map(|r| r.size).sum())
    }

    /// Get records older than a certain number of days
    pub fn get_old_records(&self, days: u32) -> Result<Vec<TrashRecord>> {
        let records = self.load()?;
        let cutoff = Utc::now() - chrono::Duration::days(days as i64);

        Ok(records
            .into_iter()
            .filter(|r| r.deleted_at < cutoff)
            .collect())
    }
}

impl Default for TrashRecordStore {
    fn default() -> Self {
        Self::new().expect("Failed to create trash record store")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_trash_record_creation() {
        let record = TrashRecord::new(
            PathBuf::from("/test/node_modules"),
            1000,
            "my-project",
            "dependencies",
        );

        assert!(!record.id.is_empty());
        assert_eq!(record.original_path, PathBuf::from("/test/node_modules"));
        assert_eq!(record.size, 1000);
        assert_eq!(record.project_name, "my-project");
    }

    // Note: More comprehensive tests would require mocking the filesystem
}
