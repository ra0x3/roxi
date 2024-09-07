use crate::ServerResult;
#[allow(unused_imports)]
use ring::{
    aead::{Aad, LessSafeKey, Nonce, OpeningKey, SealingKey, UnboundKey, AES_256_GCM},
    agreement,
    rand::SystemRandom,
};

pub struct Protocol {
    encryption_key: LessSafeKey,
}

impl Protocol {
    pub fn new() -> ServerResult<Self> {
        // TODO: Replace simulated key exchange with authentic key exchange
        let bytes = [0u8; 32];
        let unbound_key = UnboundKey::new(&AES_256_GCM, &bytes)?;
        let encryption_key = LessSafeKey::new(unbound_key);

        Ok(Self { encryption_key })
    }

    pub async fn encrypt_egress(&self, data: &[u8]) -> ServerResult<Vec<u8>> {
        let nonce = Nonce::assume_unique_for_key([0u8; 12]);
        let aad = Aad::from([0u8; 0]);
        let mut buff = data.to_vec();
        self.encryption_key
            .seal_in_place_append_tag(nonce, aad, &mut buff)?;
        Ok(buff)
    }

    pub async fn decrypt_ingress(&self, data: &[u8]) -> ServerResult<Vec<u8>> {
        let nonce = Nonce::assume_unique_for_key([0u8; 12]);
        let aad = Aad::from([0u8; 0]);
        let mut buff = data.to_vec();
        let result = self.encryption_key.open_in_place(nonce, aad, &mut buff)?;
        Ok(result.to_vec())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_protocol_encryption_decryption() {
        let proto = Protocol::new().unwrap();
        let data = b"Hello, world!";
        let encrypted = proto.encrypt_egress(data).await.unwrap();
        let decrypted = proto.decrypt_ingress(&encrypted).await.unwrap();
        assert_eq!(decrypted, data);
    }
}
