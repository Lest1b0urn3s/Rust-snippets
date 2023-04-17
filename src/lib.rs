#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;
// extern crate libsecp256k1;

// pink_extension is short for Phala ink! extension
use pink_extension as pink;

// // Custom crates
pub mod error;
pub mod ec;

#[pink::contract(env=PinkEnvironment)]
mod phat_crypto {

    use super::pink;

    use pink::{
        PinkEnvironment,
        chain_extension::{signing, SigType},
    };

    use pink_web3::types::{Address, U256, H160};
    use pink_web3::api::{Eth, Namespace};
    use pink_web3::transports::pink_http::{resolve_ready, PinkHttp};
    use pink_web3::contract::{Contract, Options};
    use pink_web3::Web3;

    use ink_env::ecdsa_recover;

    // No std crate
    use alloc::{vec, vec::Vec, string::String, format, str::FromStr};

    use aes_gcm_siv::aead::{Nonce, KeyInit, Aead};
    use aes_gcm_siv::Aes256GcmSiv;
    use cipher::{consts::{U12, U32}, generic_array::GenericArray};

    use crate::error::CryptoError;

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

        // #[ink(message)]
        // pub fn validate_ownership(&self) -> CustomResult<U256> {
        //     use pink_web3::api::{Eth, Namespace};
        //     use pink_web3::transports::pink_http::PinkHttp;
        //     let phttp = PinkHttp::new("https://moonbeam-alpha.api.onfinality.io/public");
        //     let eth = Eth::new(phttp);
        //     let result = eth.gas_price().resolve().unwrap();
        //     Ok(result)
        // }

        #[ink(message)]
        pub fn get_gas_price(&self) -> CustomResult<u128> {
            let phttp = PinkHttp::new("https://moonbeam-alpha.api.onfinality.io/public");
            let eth = Eth::new(phttp);
            let result = eth.gas_price().resolve().unwrap();
            let result1 = result.as_u128();
            Ok(result1)
        }
        
        // #[ink(message)]
        // pub fn validate_ownership(&self, account: String) -> CustomResult<u128> {
        //     let phttp = PinkHttp::new("https://moonbeam-alpha.api.onfinality.io/public");
        //     let eth = Eth::new(phttp);
        //     let addr = String::from(account).as_bytes().to_vec();
        //     // TO H160, which is just the first 20 bytes of the Substrate address
        //     let addrs = Address::from_slice(&addr[..20]);
        //     let contract = Contract::from_json(eth, addrs, include_bytes!("erc721_abi.json")).unwrap();
        //     let query = "ownerOf";

        //     let token_id = 2;
        //     let params = (token_id);
            
        //     let result: u128 = resolve_ready(contract.query(&query, params, addrs, Options::default(), None)).unwrap();
        //     Ok(result)
        // }

        #[ink(message)]
        pub fn recovery(&self, sig: Vec<u8>, msg: Vec<u8>) -> CustomResult<[u8; 33]> {
            let mut signature = [0u8; 65];
            hex::decode_to_slice(sig, &mut signature as &mut [u8; 65]).unwrap();

            let mut message = [0u8; 32];
            hex::decode_to_slice(msg, &mut message as &mut [u8; 32]).unwrap();
            
            let mut output = [0; 33];
            ecdsa_recover(&signature, &message, &mut output).unwrap();

            Ok(output)
        }

    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use core::future::ready;
        use pink::debug;
        use pink_web3::types::H160;

        // #[ink::test]
        // fn crypto(self) {
        //     let payload = "test";

        //     let key: &GenericArray<u8, U32> = GenericArray::from_slice(&self.private_key);
        //     let nonce: &GenericArray<u8, U12> = Nonce::<Aes256GcmSiv>::from_slice(&self.salt);
        
        //     // Encrypt payload
        //     let cipher = Aes256GcmSiv::new(key.into());
        //     let encrypted_text: Vec<u8> = cipher.encrypt(nonce, plaintext.as_bytes().as_ref()).unwrap();
        
        //     // Generate key and nonce
        //     let key_bytes: Vec<u8> = vec![0; 32];
        //     let key: &GenericArray<u8, U32> = GenericArray::from_slice(&key_bytes);
        //     let nonce_bytes: Vec<u8> = vec![0; 12];
        //     let nonce: &GenericArray<u8, U12> = Nonce::<Aes256GcmSiv>::from_slice(&nonce_bytes);
        
        //     // Encrypt payload
        //     let cipher = Aes256GcmSiv::new(key.into());
        //     let encrypted_text: Vec<u8> = cipher.encrypt(nonce, payload.as_bytes().as_ref()).unwrap();
        
        //     // Generate key and nonce
        //     let key_bytes: Vec<u8> = vec![0; 32];
        //     let key: &GenericArray<u8, U32> = GenericArray::from_slice(&key_bytes);
        //     let nonce_bytes: Vec<u8> = vec![0; 12];
        //     let nonce: &GenericArray<u8, U12> = Nonce::<Aes256GcmSiv>::from_slice(&nonce_bytes);
        
        //     // Decrypt payload
        //     let cipher = Aes256GcmSiv::new(key.into());
        //     let decrypted_text = cipher.decrypt(&nonce, encrypted_text.as_ref()).unwrap();

        //     assert_eq!(payload.as_bytes(), decrypted_text);
        //     assert_eq!(payload, String::from_utf8_lossy(&decrypted_text));
        // }

        // struct Foo {
        //     arr: [u8; 65],
        // }
        // struct Bar {
        //     arr: [u8; 32],
        // }

        #[test]
        fn recovery_signature() {
            let sig = "83ee96fd0047083b3c302cd2de3a5c5eb87e56903486d10f410f94cb17137e0c69a94113361aeb2f958fc211709da2983824fb2247ba3a8773354df94d8b16921b";
            let msg = "5240a32aab803873141d1df3dbfce4213d56415caae2cdd234a8549836b6c96e";
            let expected_pub_key = "02b9e72dfd423bcf95b3801ac93f4392be5ff22143f9980eb78b3a860c4843bfd0";

            let mut signature = [0u8; 65];
            hex::decode_to_slice(sig, &mut signature as &mut [u8; 65]).unwrap();

            let mut message = [0u8; 32];
            hex::decode_to_slice(msg, &mut message as &mut [u8; 32]).unwrap();
            
            let mut output = [0; 33];
            ecdsa_recover(&signature, &message, &mut output).unwrap();

            let res = hex::encode(output).to_string();

            assert_eq!(res, expected_pub_key);
        }
        
       
        #[test]
        fn test_ownership_validation() {
            pink_extension_runtime::mock_ext::mock_all_ext();

            let phttp = PinkHttp::new("https://moonbase-alpha.public.blastapi.io");
            let eth = Eth::new(phttp);
            let address = Address::from_str("1b63b10dc015bbcac201490ca286e844bf7c0ff1").unwrap();
            // let from: H160 = Address::from_str("0x1b63b10dc015bbcac201490ca286e844bf7c0ff1").unwrap();
            let contract = Contract::from_json(
                eth, address, include_bytes!("erc721_abi.json")).unwrap();
            let query = "ownerOf";
            let tokenId = 2;
            let result1: Address = resolve_ready(contract.query(&query, (tokenId, ), None, Options::default(), None)).unwrap();
            // let x: String = resolve_ready(
            //     contract
            //     .query("name", (), None, Options::default(), None)
            // ).unwrap();

            debug!("to: {:#?}", address);
            // let result: u128 = resolve_ready(contract.query(&query, (owner, ), address, Options::default(), None)).unwrap();
    
            assert_eq!(true, true);
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