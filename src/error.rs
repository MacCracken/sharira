use thiserror::Error;

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum ShariraError {
    #[error("invalid skeleton: {0}")]
    InvalidSkeleton(String),
    #[error("invalid joint: {0}")]
    InvalidJoint(String),
    #[error("invalid gait: {0}")]
    InvalidGait(String),
    #[error("bone not found: {0}")]
    BoneNotFound(String),
    #[error("computation error: {0}")]
    ComputationError(String),
    #[error("IK error: {0}")]
    IKError(String),
}

pub type Result<T> = std::result::Result<T, ShariraError>;

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn error_display() {
        let e = ShariraError::BoneNotFound("femur".into());
        assert!(e.to_string().contains("femur"));
    }
}
