use anyhow::bail;
use anyhow::Result;
use ring::aead::Aad;
use ring::aead::BoundKey;
use ring::aead::Nonce;
use ring::aead::NonceSequence;
use ring::aead::OpeningKey;
use ring::aead::SealingKey;
use ring::aead::UnboundKey;
use ring::aead::AES_256_GCM;
use ring::aead::NONCE_LEN;
use ring::error::Unspecified;
use ring::rand::SecureRandom;
use ring::rand::SystemRandom;

struct ArrayNonceSequence<'a>(&'a [u8]);

impl<'a> NonceSequence for ArrayNonceSequence<'a> {
    fn advance(&mut self) -> Result<Nonce, Unspecified> {
        Nonce::try_assume_unique_for_key(self.0)
    }
}

pub struct EncryptionResult {
    /// The encrypted data.
    pub ciphertext: Vec<u8>,
    /// A nonce generated during encryption.
    pub nonce: Vec<u8>,
    pub tag: Vec<u8>,
}

pub fn encrypt(data: &[u8], key: &str) -> Result<EncryptionResult> {
    let key = get_key(key)?;

    // Create a new AEAD key without a designated role or nonce sequence
    let unbound_key = match UnboundKey::new(&AES_256_GCM, key.as_ref()) {
        Ok(key) => key,
        Err(err) => bail!("{}", err),
    };

    // Generate nonce
    let rand = SystemRandom::new();
    let mut nonce_bytes = vec![0; NONCE_LEN];
    let nonce = match rand.fill(&mut nonce_bytes) {
        Ok(_) => nonce_bytes,
        Err(err) => bail!("error generating key: {}", err),
    };

    let nonce_sequence = ArrayNonceSequence(&nonce[..]);

    // Create a new AEAD key for encrypting and signing ("sealing"), bound to a nonce sequence
    // The SealingKey can be used multiple times, each time a new nonce will be used
    let mut sealing_key = SealingKey::new(unbound_key, nonce_sequence);

    // Create a mutable copy of the data that will be encrypted in place
    let mut ciphertext = Vec::new();
    ciphertext.extend_from_slice(data);

    // Encrypt the data with AEAD using the AES_256_GCM algorithm
    match sealing_key.seal_in_place_separate_tag(Aad::empty(), &mut ciphertext) {
        Ok(tag) => {
            let mut t = Vec::new();
            t.extend_from_slice(tag.as_ref());
            Ok(EncryptionResult {
                ciphertext,
                nonce,
                tag: t,
            })
        }
        Err(err) => bail!("error encrypting: {}", err),
    }
}

pub fn decrypt(key: &str, nonce: &[u8], tag: &[u8], data: &[u8]) -> Result<Vec<u8>> {
    let key = get_key(key)?;

    let nonce_sequence = ArrayNonceSequence(nonce);
    let unbound_key = match UnboundKey::new(&AES_256_GCM, &key) {
        Ok(key) => key,
        Err(err) => bail!("{}", err),
    };

    let mut opening_key = OpeningKey::new(unbound_key, nonce_sequence);
    let mut plaintext = [data, tag].concat();

    match opening_key.open_in_place(Aad::empty(), &mut plaintext) {
        Ok(plaintext) => {
            let mut p = Vec::with_capacity(plaintext.len());
            p.extend_from_slice(plaintext);
            Ok(p)
        }
        Err(err) => bail!("{}", err),
    }
}

const KEY_LEN: usize = 32;

fn get_key(key_str: &str) -> Result<Vec<u8>> {
    match key_str.bytes().len() {
        0 => bail!("empty key"),
        n if n < 8 => bail!("key must not be shorter than 8 characters"),
        n if n > 32 => bail!("key must not be longer than 32 characters"),
        _ => (),
    }

    let mut key = Vec::with_capacity(KEY_LEN);
    key.extend_from_slice(key_str.as_bytes());

    if key_str.len() < KEY_LEN {
        key.resize_with(KEY_LEN, Default::default);
    }

    Ok(key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_and_decrypt() {
        // Encrypt
        let key = "testing-encryption";
        let data = "Journals";
        let res = encrypt(data.as_bytes(), key).unwrap();

        // Decrypt
        let plaintext = decrypt(key, &res.nonce[..], &res.tag[..], &res.ciphertext[..]).unwrap();
        let plaintext = String::from_utf8(plaintext).unwrap();
        assert_eq!(plaintext, data);
    }

    #[test]
    fn test_encrypt_short_key() {
        // Encrypt
        let key = "testing";
        let res = encrypt(b"journals", key);
        assert!(res.is_err());
    }

    #[test]
    fn test_encrypt_long_key() {
        // Encrypt
        let key = "testing-testing-testing-testing-testing-testing";
        let res = encrypt(b"journals", key);
        assert!(res.is_err());
    }
}
