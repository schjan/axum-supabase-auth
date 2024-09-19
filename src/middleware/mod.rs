mod decoder;
mod extractor;
mod state;

pub use decoder::*;
pub use extractor::*;
use serde::{Deserialize, Serialize};
pub use state::{AuthState, CookieConfig};
use std::fmt::Debug;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims<A, U, T>
where
    A: Debug,
    U: Debug,
    T: Debug,
{
    pub sub: String,
    pub email: String,
    pub phone: String,
    pub exp: usize,
    pub role: String,
    pub app_metadata: AppMetadata<A>,
    pub user_metadata: U,
    #[serde(flatten)]
    pub additional: T,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AppMetadata<A> {
    pub provider: String,
    pub providers: Vec<String>,
    #[serde(flatten)]
    pub additional: A,
}

pub type Empty = serde_json::Value;
