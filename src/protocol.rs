use serde::{Deserialize, Serialize};

use crate::model::Envelope;

#[derive(Serialize)]
pub struct CheckoutRequest {
    pub transaction_hash: u64,
}

#[derive(Deserialize)]
pub struct Transaction {
    pub transaction_hash: String,
    pub decryption_key: Vec<u8>,
}

#[derive(Debug, Deserialize)]
pub struct Org {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub website: String,
    pub logo_url: Option<String>,
    pub brand_color: Option<String>,
    pub stock_symbol: Option<String>,
    pub twitter: Option<String>,
    pub isin: Option<String>,
    pub lei: Option<String>,
    pub naics: Option<String>,
    pub created: i64,
}

#[derive(Deserialize)]
pub struct Checkout {
    pub transaction: Transaction,
    pub seller: Option<Org>,
}

#[derive(Deserialize, Serialize)]
pub struct ReceiverPayload {
    pub sender_client_id: String,
    pub envelope: Envelope,
}

pub async fn checkout_key(
    client_id: &str,
    client_secret: &str,
    transaction_hash: u64,
) -> Result<Checkout, ()> {
    let registry_url = std::env::var("REGISTRY_URL").unwrap_or_default();
    let credential = format!("{}:{}", client_id, client_secret);

    let payload = CheckoutRequest { transaction_hash };

    let payload_json = serde_json::to_string(&payload).unwrap();

    let client = reqwest::Client::new();
    let response_result = client
        .post(format!("{registry_url}/http/checkout"))
        .header("Accept", "application/json")
        .header("Authorization", credential)
        .header("Content-Type", "application/json")
        .body(payload_json)
        .send()
        .await;

    let res = match response_result {
        Ok(res) => res,
        Err(_) => return Err(()),
    };

    if res.status().is_success() {
        let data: Checkout = match res.json().await {
            Ok(val) => val,
            Err(e) => {
                info!("Failed to deserialize due to error: {}", e);

                return Err(());
            }
        };
        return Ok(data);
    }

    return Err(());
}
