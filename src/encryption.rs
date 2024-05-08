use aes_siv::{
    aead::{Aead, KeyInit, Payload},
    Aes256SivAead,
};
use serde::Deserialize;

use crate::model::Envelope;

pub fn decrypt_envelope<T>(envelope: &Envelope, key: &Vec<u8>) -> T
where
    T: for<'a> Deserialize<'a>,
{
    let cipher = Aes256SivAead::new(key[..].into());
    let decrypted = match cipher.decrypt(
        envelope.nonce[..].into(),
        Payload::from(&envelope.encrypted[..]),
    ) {
        Ok(decrypted) => decrypted,
        Err(e) => panic!("Failed to decrypt envelope: {:?}", e),
    };
    let canonical_json = String::from_utf8(decrypted).expect("Works");
    serde_json::from_str::<T>(&canonical_json).expect("Deserialization should work")
}
