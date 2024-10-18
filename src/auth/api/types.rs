use crate::{Session, User};
use either::Either;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct HealthCheckResponse {
    pub version: String,
    pub name: String,
    pub description: String,
}

#[derive(Serialize)]
pub struct SignInUpBody<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone: Option<&'a str>,
    pub password: &'a str,
}

#[derive(Deserialize, Debug)]
#[serde(transparent)]
pub struct SignUpResponse {
    #[serde(with = "either::serde_untagged")]
    inner: Either<User, Session>,
}

impl SignUpResponse {
    pub fn session(self) -> Option<Session> {
        self.into()
    }

    pub fn user(self) -> Option<User> {
        self.into()
    }
}

impl AsRef<User> for SignUpResponse {
    fn as_ref(&self) -> &User {
        match self.inner {
            Either::Left(ref user) => user,
            Either::Right(ref session) => &session.user,
        }
    }
}

impl From<SignUpResponse> for Option<User> {
    fn from(val: SignUpResponse) -> Self {
        val.inner.left()
    }
}

impl From<SignUpResponse> for Option<Session> {
    fn from(val: SignUpResponse) -> Self {
        val.inner.right()
    }
}
