use thiserror::Error;

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("Duplicate key: '{0}' already exists")]
    DuplicateKey(String),
}
