use std::collections::HashMap;
use std::fs;
use actix_web::body::{EitherBody, MessageBody};
use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::{post, web, Error, HttpMessage, HttpResponse, Responder};
use actix_web::middleware::Next;
use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm, encode, Header, EncodingKey};
use serde::{Deserialize, Serialize};
use chrono::{Utc, Months};
use sha2::{Digest, Sha256};
use crate::backup::device::Device;
use crate::backup::storage::data_path;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Claims {
    iss: String,
    exp: usize,
}
#[derive(Deserialize)]
struct JwtAuthJson{
    uuid: String,
    password: String,
}
#[derive(Deserialize)]
struct DeviceAuth {
    pub name: String,
    pub password_hash: String,
}

pub type Devices = HashMap<String, DeviceAuth>;

const EXCLUDE_ROUTES: [&str; 2] = ["/login", "/liveliness"];


fn get_decoding_secret() -> DecodingKey{
    DecodingKey::from_base64_secret(std::env::var("JWT_SECRET")
        .expect("Unable to retrieve JWT_SECRET")
        .as_ref()
    ).expect("Unable to retrieve JWT_SECRET")
}

fn get_encoding_secret() -> EncodingKey{
    EncodingKey::from_base64_secret(std::env::var("JWT_SECRET")
        .expect("Unable to retrieve JWT_SECRET")
        .as_ref()
    ).expect("Unable to retrieve JWT_SECRET")
}
pub async fn jwt_auth_middleware(
    req: ServiceRequest,
    next: Next<impl MessageBody>,
) -> Result<ServiceResponse<EitherBody<impl MessageBody>>, Error> {
    if EXCLUDE_ROUTES.contains(&req.path()) {
        return Ok(next.call(req).await?.map_into_left_body())
    }
    let secret = &get_decoding_secret();

    let token = match req.headers()
        .get("Authorization").ok_or("Authorization header missing".to_string())
        .and_then(|h| h.to_str().map_err(|_| "Authorization header invalid".to_string()))
        .and_then(|auth| auth.strip_prefix("Bearer ").ok_or("Authorization header invalid".to_string()))
        .and_then(|token| decode::<Claims>(&token, secret, &Validation::new(Algorithm::HS256)).map_err(|e| e.to_string())) {
        Ok(token_message) => token_message,
        Err(_) => {
            return Ok(req.into_response(
                HttpResponse::Unauthorized()
                    .finish()
                    .map_into_right_body(),
            ));
        }
    };

    req.extensions_mut().insert(Device {
        uuid: token.claims.iss,
    });

    Ok(next.call(req).await?.map_into_left_body())
}

#[post("/login")]
async fn login(auth: web::Json<JwtAuthJson>) -> impl Responder {
    let data = fs::read_to_string(data_path().join("devices.json")).unwrap();
    let devices: Devices = serde_json::from_str(&data).unwrap();
    match devices.get(auth.uuid.as_str())
        .map(|device_auth| hex::decode(device_auth.password_hash.clone()).unwrap() == Sha256::digest(auth.password.clone()).as_slice()) {
        Some(true) => {},
        _ => { return HttpResponse::Unauthorized().finish(); }
    }

    let my_claims = Claims {
        iss: auth.uuid.clone(),
        exp: (Utc::now() + Months::new(1)).timestamp() as usize,
    };

    let secret = &get_encoding_secret();
    let token = encode(&Header::default(), &my_claims, secret).unwrap();
    HttpResponse::Ok().body(token)
}
