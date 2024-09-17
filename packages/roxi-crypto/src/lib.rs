pub(crate) mod error;
use ring::{
    agreement::{self, EphemeralPrivateKey},
    signature::ED25519_PUBLIC_KEY_LEN,
};

pub use crate::error::CryptoError;

pub type CryptoResult<T> = core::result::Result<T, CryptoError>;

type PublicKey = [u8; 32];

pub struct KeyPair {
    pubkey: PublicKey,
    #[allow(unused)]
    privkey: EphemeralPrivateKey,
}

impl KeyPair {
    pub fn pubkey(&self) -> &PublicKey {
        &self.pubkey
    }
}

pub fn gen_keypair() -> CryptoResult<KeyPair> {
    let rng = ring::rand::SystemRandom::new();
    let privkey = agreement::EphemeralPrivateKey::generate(&agreement::X25519, &rng)?;
    let mut pubkey_bytes = [0u8; ED25519_PUBLIC_KEY_LEN];
    let pubkey = privkey.compute_public_key()?;

    pubkey_bytes.copy_from_slice(&pubkey.as_ref()[0..32]);

    Ok(KeyPair {
        privkey,
        pubkey: pubkey_bytes,
    })
}
