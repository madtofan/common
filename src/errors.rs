use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use bytes::Bytes;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{borrow::Cow, collections::HashMap, fmt::Debug};
use thiserror::Error;
use tonic::{Code, Status};
use validator::{ValidationErrors, ValidationErrorsKind};

#[derive(Debug, Deserialize, Serialize)]
pub struct ApiError {
    pub errors: HashMap<String, Vec<String>>,
}

impl ApiError {
    pub fn new(error: String) -> Self {
        let mut error_map: HashMap<String, Vec<String>> = HashMap::new();
        error_map.insert("message".to_owned(), vec![error]);
        Self { errors: error_map }
    }
}

pub type ServiceResult<T> = Result<T, ServiceError>;
pub type ServiceErrorMap = HashMap<Cow<'static, str>, Vec<Cow<'static, str>>>;

#[derive(Error, Debug)]
pub enum ServiceError {
    #[error("authentication is required to access this resource")]
    Unauthorized,
    #[error("username or password is incorrect")]
    InvalidLoginAttempt,
    #[error("user does not have privilege to access this resource")]
    Forbidden,
    #[error("Not Found Error: {0}")]
    NotFound(String),
    #[error("App Startup Error: {0}")]
    ApplicationStartup(String),
    #[error("Bad Request: {0}")]
    BadRequest(String),
    #[error("Unexpected error has occurred")]
    InternalServerError,
    #[error("Unexpected error has occurred: {0}")]
    InternalServerErrorWithContext(String),
    #[error("Object Conflict: {0}")]
    ObjectConflict(String),
    #[error("Unprocessable request has occured")]
    Unprocessable { errors: ServiceErrorMap },
    #[error(transparent)]
    ValidationError(#[from] ValidationErrors),
    #[error(transparent)]
    AxumJsonRejection(#[from] axum::extract::rejection::JsonRejection),
    #[error(transparent)]
    AnyhowError(#[from] anyhow::Error),
}

impl ServiceError {
    pub fn unprocessable_entity(errors: ValidationErrors) -> Response {
        let mut validation_errors = ServiceErrorMap::new();

        for (_, error_kind) in errors.into_errors() {
            if let ValidationErrorsKind::Struct(meta) = error_kind {
                for (struct_property, struct_error_kind) in meta.into_errors() {
                    if let ValidationErrorsKind::Field(field_meta) = struct_error_kind {
                        for error in field_meta.into_iter() {
                            validation_errors
                                .entry(Cow::from(struct_property))
                                .or_insert_with(Vec::new)
                                .push(error.message.unwrap_or_else(|| {
                                    Cow::from(format!("{} is required", struct_property))
                                }));
                        }
                    }
                }
            }
        }
        let body = Json(json!({
            "error": validation_errors,
        }));

        (StatusCode::UNPROCESSABLE_ENTITY, body).into_response()
    }
}

impl IntoResponse for ServiceError {
    fn into_response(self) -> Response {
        if let Self::ValidationError(e) = self {
            return Self::unprocessable_entity(e);
        }

        let (status, error_message) = match self {
            Self::InternalServerErrorWithContext(err) => (StatusCode::INTERNAL_SERVER_ERROR, err),
            Self::NotFound(err) => (StatusCode::NOT_FOUND, err),
            Self::ObjectConflict(err) => (StatusCode::CONFLICT, err),
            Self::Unauthorized => (
                StatusCode::UNAUTHORIZED,
                String::from("Bearer token not available or expired"),
            ),
            Self::InvalidLoginAttempt => (
                StatusCode::BAD_REQUEST,
                Self::InvalidLoginAttempt.to_string(),
            ),
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                String::from("unexpected error occured"),
            ),
        };

        let body = Json(ApiError::new(error_message));

        (status, body).into_response()
    }
}

impl From<ServiceError> for Status {
    fn from(service_error: ServiceError) -> Self {
        match service_error {
            ServiceError::Unauthorized => Status::unauthenticated(service_error.to_string()),
            ServiceError::BadRequest(_) => Status::invalid_argument(service_error.to_string()),
            ServiceError::NotFound(message) => {
                Status::with_details(Code::NotFound, message, Bytes::new())
            }
            ServiceError::InternalServerErrorWithContext(message) => {
                Status::with_details(Code::Internal, message, Bytes::new())
            }
            _ => Status::internal(service_error.to_string()),
        }
    }
}
