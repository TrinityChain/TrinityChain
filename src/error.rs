//! Error types for TrinityChain

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ChainError {
    #[error("Invalid block linkage")]
    InvalidBlockLinkage,
    #[error("Network error: {0}")]
    NetworkError(String),
    #[error("Database error: {0}")]
    DatabaseError(String),
    #[error("Invalid proof of work")]
    InvalidProofOfWork,
    #[error("Invalid Merkle root")]
    InvalidMerkleRoot,
    #[error("Invalid transaction: {0}")]
    InvalidTransaction(String),
    #[error("Triangle not found: {0}")]
    TriangleNotFound(String),
    #[error("Cryptographic error: {0}")]
    CryptoError(String),
    #[error("Wallet error: {0}")]
    WalletError(String),
    #[error("Orphan block")]
    OrphanBlock,
    #[error("API error: {0}")]
    ApiError(String),
    #[error("Authentication error: {0}")]
    AuthenticationError(String),
    #[error("Mempool is full")]
    MempoolFull,
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Bincode error: {0}")]
    BincodeError(#[from] Box<bincode::ErrorKind>),
    #[error("Fork not found")]
    ForkNotFound,
    #[error("Invalid block: {0}")]
    InvalidBlock(String),
    #[error("Internal error: {0}")]
    InternalError(String),
}

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("Internal server error")]
    InternalServerError,
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Bad request: {0}")]
    BadRequest(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            ApiError::InternalServerError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal Server Error".to_string(),
            ),
            ApiError::NotFound(message) => (StatusCode::NOT_FOUND, message),
            ApiError::BadRequest(message) => (StatusCode::BAD_REQUEST, message),
        };

        let body = Json(json!({ "error": error_message }));

        (status, body).into_response()
    }
}

impl From<ChainError> for ApiError {
    fn from(err: ChainError) -> Self {
        match err {
            ChainError::TriangleNotFound(msg) => ApiError::NotFound(msg),
            ChainError::InvalidTransaction(msg) => ApiError::BadRequest(msg),
            _ => ApiError::InternalServerError,
        }
    }
}
