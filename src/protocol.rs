use serde::{Deserialize, Serialize};
use versa::protocol::Checkout;

use versa::protocol::Envelope;

#[derive(Serialize)]
pub struct CheckoutRequest {
    pub receipt_id: String,
}

#[derive(Deserialize, Serialize)]
pub struct ReceiverPayload {
    pub sender_client_id: String,
    pub receipt_id: String,
    pub envelope: Envelope,
}

pub async fn checkout_key(
    client_id: &str,
    client_secret: &str,
    receipt_id: String,
) -> Result<Checkout, ()> {
    let registry_url = std::env::var("REGISTRY_URL").unwrap_or_default();
    let credential = format!("Basic {}:{}", client_id, client_secret);

    let payload = CheckoutRequest { receipt_id };

    let payload_json = serde_json::to_string(&payload).unwrap();

    let client = reqwest::Client::new();
    info!(
        "Sending checkout request to: {}",
        format!("{}/checkout", registry_url)
    );
    let response_result = client
        .post(format!("{}/checkout", registry_url))
        .header("Accept", "application/json")
        .header("Authorization", credential)
        .header("Content-Type", "application/json")
        .body(payload_json)
        .send()
        .await;

    let res = match response_result {
        Ok(res) => res,
        Err(e) => {
            info!("Error placing request: {:?}", e);
            return Err(());
        }
    };

    if res.status().is_success() {
        info!("Successfully received data from registry");
        let data: Checkout = match res.json().await {
            Ok(val) => val,
            Err(e) => {
                info!("Failed to deserialize due to error: {}", e);

                return Err(());
            }
        };
        return Ok(data);
    } else {
        let status = res.status();
        let text = res.text().await.unwrap_or_default();
        info!("Received an error from the registry: {} {}", status, text);
    }

    return Err(());
}
