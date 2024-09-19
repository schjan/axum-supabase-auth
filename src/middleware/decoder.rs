use super::AuthClaims;
use crate::AuthTypes;
use jsonwebtoken::{DecodingKey, Validation};
use std::marker::PhantomData;

pub struct Decoder<T>
where
    T: AuthTypes,
{
    keys: Keys,
    validation: Validation,

    phantom: PhantomData<T>,
}

impl<T> Decoder<T>
where
    T: AuthTypes,
{
    pub fn new(secret: &str) -> Self {
        let mut validation = Validation::default();
        validation.set_audience(&["authenticated"]);

        Self::new_with_validation(secret, validation)
    }

    pub fn new_with_validation(secret: &str, validation: Validation) -> Self {
        Self {
            keys: Keys::new(secret.as_bytes()),
            validation,
            phantom: PhantomData,
        }
    }

    pub fn decode(&self, token: &str) -> Result<AuthClaims<T>, jsonwebtoken::errors::Error> {
        jsonwebtoken::decode::<AuthClaims<T>>(token, &self.keys.decoding, &self.validation)
            .map(|data| data.claims)
    }
}

struct Keys {
    decoding: DecodingKey,
}

impl Keys {
    fn new(secret: &[u8]) -> Self {
        Self {
            decoding: DecodingKey::from_secret(secret),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::middleware::Empty;
    use jsonwebtoken::Validation;
    use serde::{Deserialize, Serialize};
    use std::sync::LazyLock;

    #[derive(Debug, Deserialize, Serialize)]
    struct AppMetadata {
        groups: Vec<String>,
    }

    const JWT: &str = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJodHRwOi8vMTI3LjAuMC4xOjU0MzIxL2F1dGgvdjEiLCJzdWIiOiIzNGFiYzFmNy1lMzQ2LTRiMzAtYmMyNi0xYjUzZjcwN2JmNTQiLCJhdWQiOiJhdXRoZW50aWNhdGVkIiwiZXhwIjoxNzIzOTc2ODY5LCJpYXQiOjE3MjM5NzMyNjksImVtYWlsIjoidGVzdHVzZXJAdGVzdC5jb20iLCJwaG9uZSI6IiIsImFwcF9tZXRhZGF0YSI6eyJncm91cHMiOlsiZmsiXSwicHJvdmlkZXIiOiJlbWFpbCIsInByb3ZpZGVycyI6WyJlbWFpbCJdfSwidXNlcl9tZXRhZGF0YSI6e30sInJvbGUiOiJhdXRoZW50aWNhdGVkIiwiYWFsIjoiYWFsMSIsImFtciI6W3sibWV0aG9kIjoicGFzc3dvcmQiLCJ0aW1lc3RhbXAiOjE3MjM5NzMyNjl9XSwic2Vzc2lvbl9pZCI6Ijk5OTRhNTk4LTMyNjMtNDBlNC1iMWMwLTU5YTE0ODNhODRlMiIsImlzX2Fub255bW91cyI6ZmFsc2V9.1U6LglCXAD1yJwnXndgYuvQ6muCn2MUb_ivwlKIirgk";
    const SECRET: &str = "super-secret-jwt-token-with-at-least-32-characters-long";

    static VALIDATION: LazyLock<Validation> = LazyLock::new(|| {
        let mut validation = Validation::default();
        validation.validate_exp = false;
        validation.set_audience(&["authenticated"]);
        validation
    });

    struct MyAuthTypes;

    impl AuthTypes for MyAuthTypes {
        type AppData = AppMetadata;
        type UserData = Empty;
        type AdditionalData = Empty;
    }

    struct EmptyAuthTypes;

    impl AuthTypes for EmptyAuthTypes {
        type AppData = Empty;
        type UserData = Empty;
        type AdditionalData = Empty;
    }

    #[test]
    fn test_decode_additional_app_metadata() {
        let decoder = Decoder::<MyAuthTypes>::new_with_validation(SECRET, VALIDATION.clone());
        let claims = decoder.decode(JWT).unwrap();

        assert_eq!(claims.app_metadata.additional.groups, vec!["fk"]);
        assert_eq!(claims.app_metadata.provider, "email");
    }

    #[test]
    fn test_decode_no_additional_app_metadata() {
        let decoder = Decoder::<EmptyAuthTypes>::new_with_validation(SECRET, VALIDATION.clone());
        let claims = decoder.decode(JWT).unwrap();

        assert_eq!(claims.app_metadata.provider, "email");
    }
}
