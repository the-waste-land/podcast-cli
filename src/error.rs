use thiserror::Error;

#[derive(Debug, Error)]
pub enum PodcastCliError {
    #[error("Configuration error: {0}")]
    Config(String),
    #[error("API error: {0}")]
    Api(String),
    #[error("Validation error: {0}")]
    Validation(String),
    #[error("Metadata error: {0}")]
    Metadata(String),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("Not implemented: {0}")]
    NotImplemented(String),
}

impl PodcastCliError {
    pub fn exit_code(&self) -> i32 {
        match self {
            Self::Validation(_) => 2,
            Self::Config(_) => 3,
            Self::Metadata(_) => 4,
            Self::Api(_) | Self::Http(_) => 5,
            Self::Io(_) => 6,
            Self::Serialization(_) => 1,
            Self::NotImplemented(_) => 7,
        }
    }

    pub fn progress_code(&self) -> &'static str {
        match self {
            Self::Validation(_) => "validation_error",
            Self::Config(_) => "config_error",
            Self::Metadata(_) => "metadata_error",
            Self::Api(_) | Self::Http(_) => "network_error",
            Self::Io(_) => "io_error",
            Self::Serialization(_) => "serialization_error",
            Self::NotImplemented(_) => "not_implemented",
        }
    }
}

pub type Result<T> = std::result::Result<T, PodcastCliError>;

#[cfg(test)]
mod tests {
    use super::PodcastCliError;

    #[test]
    fn exit_code_mapping_matches_contract() {
        assert_eq!(
            PodcastCliError::Validation("bad args".to_string()).exit_code(),
            2
        );
        assert_eq!(
            PodcastCliError::Config("missing key".to_string()).exit_code(),
            3
        );
        assert_eq!(
            PodcastCliError::Metadata("missing enclosure".to_string()).exit_code(),
            4
        );
        assert_eq!(
            PodcastCliError::Api("status 500".to_string()).exit_code(),
            5
        );
        assert_eq!(
            PodcastCliError::Io(std::io::Error::other("disk full")).exit_code(),
            6
        );
    }
}
