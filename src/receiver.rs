use http::HeaderMap;
use serde_json::Value;
use versa::{client::VersaClient, client_receiver::VersaReceiver, protocol::ReceiverPayload};

pub async fn target(
    headers: HeaderMap,
    raw_body: axum::body::Body,
) -> Result<http::StatusCode, (http::StatusCode, String)> {
    let receiver_client_id = std::env::var("CLIENT_ID").expect("CLIENT_ID must be set");
    let receiver_client_secret = std::env::var("CLIENT_SECRET").expect("CLIENT_SECRET must be set");
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

    let versa_client = VersaClient::new(receiver_client_id, receiver_client_secret)
        .receiving_client(receiver_secret);

    let body_bytes = versa_client
        .verify_event(
            axum::body::to_bytes(raw_body, 512_000_000).await.unwrap(),
            request_token,
        )
        .map_err(|_| {
            (
                http::StatusCode::UNAUTHORIZED,
                "Failed to verify request signature".to_string(),
            )
        })?;

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

    let checkout = versa_client.checkout_key(receipt_id).await.map_err(|_| {
        (
            http::StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to checkout key".to_string(),
        )
    })?;

    info!("Received keys for sender: {:?}", checkout.sender);
    let data = versa_client.decrypt_envelope::<Value>(envelope, checkout.key);

    info!(
        "DATA RECEIVED FROM SENDER_CLIENT_ID={}: {:?}",
        sender_client_id,
        serde_json::to_string(&data).unwrap()
    );

    Ok(http::StatusCode::ACCEPTED)
}
