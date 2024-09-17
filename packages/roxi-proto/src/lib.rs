pub(crate) mod error;
pub(crate) mod message;
pub(crate) mod wireguard;

pub type ProtoResult<T> = core::result::Result<T, error::ProtoError>;

pub use error::ProtoError;
pub use message::{Message, MessageKind, MessageStatus};

use ring::aead::{Aad, LessSafeKey, Nonce, UnboundKey, AES_256_GCM};

pub struct Protocol {
    encryption_key: LessSafeKey,
}

impl Protocol {
    pub fn new() -> ProtoResult<Self> {
        // TODO: Replace simulated key exchange with authentic key exchange
        let bytes = [0u8; 32];
        let unbound_key = UnboundKey::new(&AES_256_GCM, &bytes)?;
        let encryption_key = LessSafeKey::new(unbound_key);

        Ok(Self { encryption_key })
    }

    pub async fn encrypt_egress(&self, data: &[u8]) -> ProtoResult<Vec<u8>> {
        let nonce = Nonce::assume_unique_for_key([0u8; 12]);
        let aad = Aad::from([0u8; 0]);
        let mut buff = data.to_vec();
        self.encryption_key
            .seal_in_place_append_tag(nonce, aad, &mut buff)?;
        Ok(buff)
    }

    pub async fn decrypt_ingress(&self, data: &[u8]) -> ProtoResult<Vec<u8>> {
        let nonce = Nonce::assume_unique_for_key([0u8; 12]);
        let aad = Aad::from([0u8; 0]);
        let mut buff = data.to_vec();
        let result = self.encryption_key.open_in_place(nonce, aad, &mut buff)?;
        Ok(result.to_vec())
    }
}
