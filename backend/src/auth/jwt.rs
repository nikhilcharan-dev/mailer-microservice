use anyhow::{Context, Result};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,        // user_id (ObjectId as hex string)
    pub username: String,
    pub exp: usize,
    pub iat: usize,
}

pub struct JwtConfig {
    pub secret: Vec<u8>,
    pub expiry_hours: i64,
}

impl JwtConfig {
    pub fn from_env() -> Result<Self> {
        let secret = std::env::var("JWT_SECRET").context("JWT_SECRET required")?;
        if secret.len() < 32 {
            tracing::warn!("JWT_SECRET is shorter than 32 bytes — consider a longer secret");
        }
        let expiry_hours: i64 = std::env::var("JWT_EXPIRY_HOURS")
            .unwrap_or_else(|_| "24".into())
            .parse()
            .unwrap_or(24);
        Ok(Self {
            secret: secret.into_bytes(),
            expiry_hours,
        })
    }

    pub fn issue(&self, user_id: &str, username: &str) -> Result<String> {
        let now = Utc::now();
        let exp = now + Duration::hours(self.expiry_hours);
        let claims = Claims {
            sub: user_id.to_string(),
            username: username.to_string(),
            iat: now.timestamp() as usize,
            exp: exp.timestamp() as usize,
        };
        let token = encode(
            &Header::new(Algorithm::HS256),
            &claims,
            &EncodingKey::from_secret(&self.secret),
        )?;
        Ok(token)
    }

    pub fn verify(&self, token: &str) -> Result<Claims> {
        let data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(&self.secret),
            &Validation::new(Algorithm::HS256),
        )?;
        Ok(data.claims)
    }
}
