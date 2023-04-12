#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;
// extern crate libsecp256k1;

// pink_extension is short for Phala ink! extension
use pink_extension as pink;

// // Custom crates
pub mod error;
// pub mod ec;

#[pink::contract(env=PinkEnvironment)]
mod phat_crypto {

    use super::pink;

    use pink::{
        PinkEnvironment,
        chain_extension::{signing, SigType},
    };

    // No std crate
    use alloc::{vec, vec::Vec, string::String, format};

    use aes_gcm_siv::aead::{Nonce, KeyInit, Aead};
    use aes_gcm_siv::{Aes256GcmSiv};
    use cipher::{consts::{U12, U32}, generic_array::GenericArray};

    use crate::error::CryptoError;
    // use crate::ec::secp256k1::recover_key;

    pub type CustomResult<T> = core::result::Result<T, CryptoError>;

    #[ink(storage)]
    pub struct PhatCrypto {
        private_key: Vec<u8>,
        salt: Vec<u8>
    }

    impl PhatCrypto {
        #[ink(constructor)]
        pub fn new() -> Self {
            // TODO: Just some generic salt - randomize
            let salt = b"981781668367";
            // TODO: Private key generation -> Now it's just dummy, but salted still
            let private_key = vec![0; 32];

            // Return SELF with parameters
            Self { private_key, salt: salt.to_vec() }
        }

        #[ink(message)]
        pub fn aes_gcm_encrypt(&self, plaintext: String) -> CustomResult<Vec<u8>> {
            let key: &GenericArray<u8, U32> = GenericArray::from_slice(&self.private_key);
            let nonce: &GenericArray<u8, U12> = Nonce::<Aes256GcmSiv>::from_slice(&self.salt);
        
            // Encrypt payload
            let cipher = Aes256GcmSiv::new(key.into());
            let encrypted_text: Vec<u8> = cipher.encrypt(nonce, plaintext.as_bytes().as_ref()).unwrap();
        
            Ok(encrypted_text)
        }

        #[ink(message)]
        pub fn aes_gcm_decrypt(&self, cipher_text: Vec<u8>) -> CustomResult<String> {        
            let key: &GenericArray<u8, U32> = GenericArray::from_slice(&self.private_key);
            let nonce: &GenericArray<u8, U12> = Nonce::<Aes256GcmSiv>::from_slice(&self.salt);
        
            // Decrypt payload
            let cipher = Aes256GcmSiv::new(key.into());
            let decrypted_text = cipher.decrypt(&nonce, cipher_text.as_ref()).unwrap();
            let result = format!("{}", String::from_utf8_lossy(&decrypted_text));

            Ok(result)
        }

        #[ink(message)]
        pub fn get_public_key(&self) -> CustomResult<Vec<u8>> {
            Ok(signing::get_public_key(&self.private_key, SigType::Sr25519))
        }

        #[ink(message)]
        pub fn get_private_key(&self) -> CustomResult<Vec<u8>> {
            Ok(self.private_key.clone())
        }

        #[ink(message)]
        pub fn get_salt(&self) -> CustomResult<Vec<u8>> {
            Ok(self.salt.clone())
        }

        #[ink(message)]
        pub fn recover_pubkey(&self, v: u8, r: Vec<u8>, s: Vec<u8>, message: Vec<u8>) -> CustomResult<()> {
            // let rA = r[..].try_into().unwrap();
            // let sA = s[..].try_into().unwrap();
            // let pubkey = recover_key(v, rA, sA, &message[..]).unwrap();
            // let result = format!("{}", String::from_utf8_lossy(&message));
            Ok(())
            // Ok(result.clone())
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
    
        #[ink::test]
        fn crypto() {
            let payload = "test";

            let key: &GenericArray<u8, U32> = GenericArray::from_slice(&self.private_key);
            let nonce: &GenericArray<u8, U12> = Nonce::<Aes256GcmSiv>::from_slice(&self.salt);
        
            // Encrypt payload
            let cipher = Aes256GcmSiv::new(key.into());
            let encrypted_text: Vec<u8> = cipher.encrypt(nonce, plaintext.as_bytes().as_ref()).unwrap();
        
            // Generate key and nonce
            let key_bytes: Vec<u8> = vec![0; 32];
            let key: &GenericArray<u8, U32> = GenericArray::from_slice(&key_bytes);
            let nonce_bytes: Vec<u8> = vec![0; 12];
            let nonce: &GenericArray<u8, U12> = Nonce::<Aes256GcmSiv>::from_slice(&nonce_bytes);
        
            // Encrypt payload
            let cipher = Aes256GcmSiv::new(key.into());
            let encrypted_text: Vec<u8> = cipher.encrypt(nonce, payload.as_bytes().as_ref()).unwrap();
        
            // Generate key and nonce
            let key_bytes: Vec<u8> = vec![0; 32];
            let key: &GenericArray<u8, U32> = GenericArray::from_slice(&key_bytes);
            let nonce_bytes: Vec<u8> = vec![0; 12];
            let nonce: &GenericArray<u8, U12> = Nonce::<Aes256GcmSiv>::from_slice(&nonce_bytes);
        
            // Decrypt payload
            let cipher = Aes256GcmSiv::new(key.into());
            let decrypted_text = cipher.decrypt(&nonce, encrypted_text.as_ref()).unwrap();

            assert_eq!(payload.as_bytes(), decrypted_text);
            assert_eq!(payload, String::from_utf8_lossy(&decrypted_text));
        }
    }
        
        // #[ink::test]
        // fn verify_signature() {
        //     let message = "Test String";
        //     let account = "0xC9c7731FAB51730224d5a9Ec433F59433Eb35166".to_string();
        //     let message = eth_message(message);
        //     let signature = hex::decode("b6fa05b690d2f27599b6f8e015e1a1bba4b8800392ae3cb34960c7334451481955f38ccfd1115b69350890f3663cc9ed0ae5257902a955a854793e2fcc7672e81b").unwrap();
        // }
}

