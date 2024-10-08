use base64::prelude::*;
use hmac::Mac;
use http::HeaderMap;
use serde_json::Value;

use crate::protocol::ReceiverPayload;

async fn verify_with_secret(
    body: axum::body::Body,
    secret: String,
    token: &str,
) -> (bool, hyper::body::Bytes) {
    let mut mac = hmac::Hmac::<sha1::Sha1>::new_from_slice(&secret.as_bytes()).unwrap();
    let body_bytes = axum::body::to_bytes(body, 512_000_000).await.unwrap();
    mac.update(body_bytes.as_ref());
    let code_bytes = mac.finalize().into_bytes();
    let encoded = BASE64_STANDARD.encode(&code_bytes.to_vec());
    (encoded == token, body_bytes)
}

pub async fn target(
    headers: HeaderMap,
    raw_body: axum::body::Body,
) -> Result<http::StatusCode, (http::StatusCode, String)> {
    let receiver_secret = std::env::var("RECEIVER_SECRET").expect("RECEIVER_SECRET must be set");

    let Some(request_signature) = headers.get("X-Request-Signature") else {
        return Err((
            http::StatusCode::BAD_REQUEST,
            "Missing X-Request-Signature header".to_string(),
        ));
    };
    let Ok(request_token) = request_signature.to_str() else {
        return Err((
            http::StatusCode::BAD_REQUEST,
            "Malformed X-Request-Signature header".to_string(),
        ));
    };
    let (verified, body_bytes) = verify_with_secret(raw_body, receiver_secret, request_token).await;
    if !verified {
        return Err((
            http::StatusCode::UNAUTHORIZED,
            "Failed to verify request signature".to_string(),
        ));
    }
    info!("Successfully verified hmac request signature");
    let body: ReceiverPayload = match serde_json::from_slice(&body_bytes) {
        Ok(val) => val,
        Err(e) => {
            return Err((
                http::StatusCode::BAD_REQUEST,
                format!("Failed to parse body: {}", e),
            ));
        }
    };

    let ReceiverPayload {
        sender_client_id,
        receipt_id,
        envelope,
    } = body;

    info!("Received envelope from sender={}", sender_client_id);

    let receiver_client_id = std::env::var("CLIENT_ID").unwrap_or_default();
    let receiver_client_secret = std::env::var("CLIENT_SECRET").unwrap_or_default();

    let checkout =
        crate::protocol::checkout_key(&receiver_client_id, &receiver_client_secret, receipt_id)
            .await
            .map_err(|_| {
                (
                    http::StatusCode::INTERNAL_SERVER_ERROR,
                    "Failed to checkout key".to_string(),
                )
            })?;

    info!("Received keys for sender: {:?}", checkout.sender);
    let data = crate::encryption::decrypt_envelope::<Value>(envelope, checkout.key);

    info!(
        "DATA RECEIVED FROM SENDER_CLIENT_ID={}: {:?}",
        sender_client_id,
        serde_json::to_string(&data).unwrap()
    );

    Ok(http::StatusCode::ACCEPTED)
}
