use crate::protocol::ReceiverPayload;

pub async fn target(
    axum::extract::Json(body): axum::extract::Json<ReceiverPayload>,
) -> Result<http::StatusCode, (http::StatusCode, String)> {
    let ReceiverPayload {
        sender_client_id,
        envelope,
    } = body;

    info!("Received envelope from sender={}", sender_client_id);

    let receiver_client_id = std::env::var("CLIENT_ID").unwrap_or_default();
    let receiver_client_secret = std::env::var("CLIENT_SECRET").unwrap_or_default();

    let checkout =
        crate::protocol::checkout_key(&receiver_client_id, &receiver_client_secret, envelope.hash)
            .await
            .map_err(|_| {
                (
                    http::StatusCode::INTERNAL_SERVER_ERROR,
                    "Failed to checkout key".to_string(),
                )
            })?;

    info!("Received keys for sender: {:?}", checkout.seller);
    let data = crate::encryption::decrypt_envelope::<crate::model::SenderReceiptHeader>(
        &envelope,
        &checkout.transaction.decryption_key,
    );

    info!(
        "DATA RECEIVED FROM SENDER_CLIENT_ID={}: {:?}",
        sender_client_id, data
    );

    Ok(http::StatusCode::ACCEPTED)
}
