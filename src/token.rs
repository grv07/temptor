use jsonwebtokens::{Algorithm, Verifier, VerifierBuilder, encode};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Serialize, Deserialize)]
struct Claims {
    username: String,
    roles: Vec<String>,
    token_version: String,
}

pub fn generate_token(
    username: String,
    token_version: String,
) -> Result<String, jsonwebtokens::error::Error> {
    encode_token(username, token_version)
}

pub fn verify_token(
    token: String,
    username: String,
    token_version: String,
) -> Result<serde_json::value::Value, jsonwebtokens::error::Error> {
    let alg = Algorithm::new_hmac(jsonwebtokens::AlgorithmID::HS512, "secret")?;

    let verifier = Verifier::create()
        .leeway(5)
        .string_equals("username", username)
        .string_equals("token_version", token_version)
        .build()?;

    verifier.verify(token, &alg)
}

fn encode_token(
    username: String,
    token_version: String,
) -> Result<String, jsonwebtokens::error::Error> {
    let alg = Algorithm::new_hmac(jsonwebtokens::AlgorithmID::HS512, "secret")?;

    let header = json!({
        "alg": alg.name(),
        "typ": "JWT"
    });

    let claims = Claims {
        username,
        roles: vec!["admin".to_string()],
        token_version,
    };
    encode(&header, &claims, &alg)
}
