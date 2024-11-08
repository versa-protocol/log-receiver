pub fn validate() {
    std::env::var("CLIENT_ID").expect("Missing environment variable CLIENT_ID");
    std::env::var("CLIENT_SECRET").expect("Missing environment variable CLIENT_SECRET");
    std::env::var("RECEIVER_SECRET").expect("Missing environment variable RECEIVER_SECRET");
}
