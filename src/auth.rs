// server/backend_vibrance/src/auth.rs
use anyhow::Result;
use jsonwebtoken::{decode, DecodingKey, Validation, decode_header};
use serde::{Serialize, Deserialize};
use std::env;
use reqwest;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // UUID del usuario
    pub exp: usize,  // Expiración del token (si es necesario)
}

/// Esta función valida y decodifica el token emitido por Auth0 usando su JWKS.
pub async fn decode_jwt(token: &str) -> Result<Claims> {
    let auth0_domain = env::var("VITE_AUTH0_DOMAIN")
        .expect("VITE_AUTH0_DOMAIN debe estar definido en el .env");
    let jwks_url = format!("https://{}/.well-known/jwks.json", auth0_domain);
    
    let jwks: serde_json::Value = reqwest::get(&jwks_url).await?.json().await?;
    
    let header = decode_header(token)?;
    let kid = header.kid.ok_or_else(|| anyhow::anyhow!("No se encontró 'kid' en el header del token"))?;
    
    let jwk = jwks["keys"].as_array()
        .and_then(|keys| keys.iter().find(|k| k["kid"].as_str() == Some(&kid)))
        .ok_or_else(|| anyhow::anyhow!("No se encontró la clave (JWK) para el kid: {}", kid))?;
    
    let n = jwk["n"].as_str().ok_or_else(|| anyhow::anyhow!("Falta el parámetro 'n' en el JWK"))?;
    let e = jwk["e"].as_str().ok_or_else(|| anyhow::anyhow!("Falta el parámetro 'e' en el JWK"))?;
    
    let decoding_key = DecodingKey::from_rsa_components(n, e)
        .map_err(|err| anyhow::anyhow!("Error construyendo la clave de decodificación: {}", err))?;
    
    let mut validation = Validation::new(jsonwebtoken::Algorithm::RS256);
    if let Ok(audience) = env::var("VITE_AUTH0_AUDIENCE") {
        if !audience.is_empty() {
            validation.set_audience(&[&audience]);
        }
    }
    let issuer = format!("https://{}/", auth0_domain);
    validation.set_issuer(&[&issuer]);
    
    let token_data = decode::<Claims>(token, &decoding_key, &validation)?;
    Ok(token_data.claims)
}
