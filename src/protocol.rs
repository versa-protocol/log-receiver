use versa::client::VersaClient;
use versa::client_receiver::VersaReceiver;
use versa::protocol::Checkout;

pub async fn checkout_key(
    client_id: &str,
    client_secret: &str,
    receipt_id: String,
) -> Result<Checkout, ()> {
    let receiver_secret = std::env::var("RECEIVER_SECRET").expect("RECEIVER_SECRET must be set");

    let versa_client =
        VersaClient::new(client_id.into(), client_secret.into()).receiving_client(receiver_secret);

    match versa_client.checkout_key(receipt_id).await {
        Ok(checkout) => Ok(checkout),
        Err(_) => Err(()),
    }
}
