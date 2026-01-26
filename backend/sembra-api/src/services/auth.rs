use argon2::{
    password_hash::{
        rand_core::OsRng,
        PasswordHash, PasswordHasher, PasswordVerifier, SaltString
    },
    Argon2
};
use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey};
use serde::{Deserialize, Serialize};
use chrono::{Utc, Duration};
use anyhow::Result;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
    pub role: String,
}

pub struct AuthService {
    secret: String,
}

impl AuthService {
    pub fn new(secret: String) -> Self {
        Self { secret }
    }

    pub fn hash_password(&self, password: &str) -> Result<String> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        Ok(argon2.hash_password(password.as_bytes(), &salt)
            .map_err(|e| anyhow::anyhow!("Argon2 Error: {}", e))?
            .to_string())
    }

    pub fn verify_password(&self, hash: &str, password: &str) -> bool {
        let parsed_hash = match PasswordHash::new(hash) {
            Ok(h) => h,
            Err(_) => return false,
        };
        Argon2::default().verify_password(password.as_bytes(), &parsed_hash).is_ok()
    }

    pub fn generate_token(&self, user_id: &str, role: &str) -> Result<String> {
        let expiration = Utc::now()
            .checked_add_signed(Duration::hours(24))
            .expect("valid timestamp")
            .timestamp();

        let claims = Claims {
            sub: user_id.to_owned(),
            exp: expiration as usize,
            role: role.to_owned(),
        };

        Ok(encode(&Header::default(), &claims, &EncodingKey::from_secret(self.secret.as_bytes()))?)
    }
    
    pub fn verify_token(&self, token: &str) -> Result<Claims> {
         Ok(decode::<Claims>(token, &DecodingKey::from_secret(self.secret.as_bytes()), &Validation::default())?.claims)
    }
}
