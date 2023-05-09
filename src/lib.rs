#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

// // Custom crates
pub mod error;
pub mod ec;

// pink_extension is short for Phala ink! extension
use pink_extension as pink;
use serde::{Deserialize, Serialize};
use alloc::{vec, vec::Vec, string::String};
use pink_json;

#[derive(Serialize, Deserialize)]
pub struct IPFSSync { directSync: bool }

pub fn create_ipfs_sync(direct_sync: bool) -> String {
    // Some data structure.
    let sync = IPFSSync { directSync: direct_sync };

    // Serialize it to a JSON string.
    let s = pink_json::to_string(&sync);

    s.unwrap()
}

#[derive(Serialize, Deserialize)]
pub struct RequestContent {
    data: String,
}

// This is a trait, which is used to serialize / deserialize data in the struct
#[derive(Serialize, Deserialize)]
pub struct UploadedFile {
    fileName: String,
    contentType: String
}

#[derive(Serialize, Deserialize)]
pub struct PendingFile {
    fileName: String,
    url: String,
    fileUuid: String
}

#[derive(Serialize, Deserialize)]
pub struct StorageResponseData {
    sessionUuid: String,
    files: Vec<PendingFile>,
}

#[derive(Serialize, Deserialize)]
pub struct StorageResponse {
    id: String,
    status: u8,
    data: StorageResponseData
}

#[derive(Serialize, Deserialize)]
pub struct FileContent {
    id: String,
}

#[derive(Serialize, Deserialize)]
pub struct Files { files: Vec<UploadedFile> }

pub fn create_file_payload(file: UploadedFile) -> String {
    // Some data structure.
    let files = Files { files: vec![file]};

    // Serialize it to a JSON string.
    let v = pink_json::to_string(&files);

    v.unwrap()
}


#[pink::contract(env=PinkEnvironment)]
mod phat_crypto {

    use super::pink;

    use binascii::b64encode;
    use pink::{
        PinkEnvironment,
        chain_extension::{signing, SigType},
        http_get, http_post, http_put
    };
    use pink_json;
    use crate::{
        create_file_payload,
        create_ipfs_sync,
        UploadedFile,
        StorageResponse,
    };
    use alloc::{vec, vec::Vec, string::String, format};

    use pink_web3::types::{Address, H160, U256};
    use pink_web3::contract::{Contract, Options};
    use pink_web3::api::{Eth, Namespace};
    use pink_web3::transports::{
        pink_http::{PinkHttp},
        resolve_ready
    };

    use ink_env::ecdsa_recover;

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
        
        #[ink(message)]
        pub fn validate_ownership(&self, token_id: u8) -> CustomResult<String> {
            let phttp = PinkHttp::new("https://moonbase-alpha.public.blastapi.io");
            let eth = Eth::new(phttp);

            let addrs_hex = hex_literal::hex!("1b63b10dC015bbcaC201490CA286e844bf7c0ff1");
            let address: Address = Address::from_slice(&addrs_hex);
            let contract = Contract::from_json(eth, address, include_bytes!("../abi/erc721_abi.json")).unwrap();

            let query = "ownerOf";
            let result1: Address = resolve_ready(contract.query(&query, (U256::from(token_id), ), None, Options::default(), None)).unwrap();

            Ok(format!("{:?}", result1))
        }

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

        #[ink(message)]
        pub fn download_file_from_decentralized(&self, url: String) -> CustomResult<String> {
            let response = http_get!(url);
            let key: &GenericArray<u8, U32> = GenericArray::from_slice(&self.private_key);
            let nonce: &GenericArray<u8, U12> = Nonce::<Aes256GcmSiv>::from_slice(&self.salt);
        
            // Decrypt payload
            let cipher = Aes256GcmSiv::new(key.into());
            let decrypted_text = cipher.decrypt(&nonce, response.body.as_ref()).unwrap();
            let result = format!("{}", String::from_utf8_lossy(&decrypted_text));

            Ok(result)
        }

        #[ink(message)]
        pub fn upload_file_to_decentralized(&self, bucket_uuid: String, file_name: String, file_content: String) -> CustomResult<String> {
            let key: &GenericArray<u8, U32> = GenericArray::from_slice(&self.private_key);
            let nonce: &GenericArray<u8, U12> = Nonce::<Aes256GcmSiv>::from_slice(&self.salt);
        
            // Encrypt payload
            let cipher = Aes256GcmSiv::new(key.into());
            let cncrypted_content: Vec<u8> = cipher.encrypt(nonce, file_content.as_bytes().as_ref()).unwrap();

            let content_type = String::from("text/html");
            let bucket_uuid: String = String::from(bucket_uuid);

            // ** UPLOAD FILE TO APILLON STORAGE ** //
            let url_f_upload = format!("https://api-dev.apillon.io/storage/{}/upload", bucket_uuid);
            let url_get_content: String = format!("https://api-dev.apillon.io/storage/{}/content", bucket_uuid);
            
            let file = UploadedFile { fileName: String::from(file_name), contentType: content_type };
            let json_data = create_file_payload(file);

            let mut output_buffer = [0u8; 68];
            let message = "44f2b448-b89c-42df-afd2-c487d9a7b4a4:@AUAY0DHm86P";
            let encoded = b64encode(&message.as_bytes(), &mut output_buffer).ok().unwrap();
            let authorization = format!("Basic {}", String::from_utf8_lossy(encoded));
            let content_type = format!("application/json");

            let headers: Vec<(String, String)> = vec![
                ("Authorization".into(), authorization),
                ("Content-Type".into(), content_type)
            ];

            // assert_ne!(headers, headers);
            let response = http_get!(url_get_content, headers.clone());
            assert_eq!(response.status_code, 200);

            let response = http_post!(url_f_upload, json_data, headers.clone());

            let resp_body_str = match String::from_utf8(response.body) {
                Ok(r) => r,
                Err(e) => panic!("Mja, error, kaj ces {}", e),
            };

            assert_eq!(response.status_code, 201);
            let resp: StorageResponse = pink_json::from_str(&resp_body_str).unwrap();
            let file = &resp.data.files[0];
            let url_upload_s3: String = format!("{}", file.url);
            let content = cncrypted_content;
            let origin = String::from("https://app-dev.apillon.io/");

            let content_type = format!("text/plain");
            let headers_s3: Vec<(String, String)> = vec![
                ("Content-Type".into(), content_type),
                ("Referer".into(), origin.clone()),
                ("Origin".into(), origin.clone())
            ];

            let response = http_put!(url_upload_s3, *content, headers_s3);
            assert_eq!(response.status_code, 200);

            // ** TRIGGER UPLOAD TO IPFS (From Apillon storage) ** //
            let url_sync_ipfs = format!(
                "https://api-dev.apillon.io/storage/{}/upload/{}/end", 
                bucket_uuid, resp.data.sessionUuid);

            let ipfs_sync_json = create_ipfs_sync(true);

            let response = http_post!(url_sync_ipfs, ipfs_sync_json, headers.clone());
            assert_eq!(response.status_code, 200);

            Ok(String::from("DONE"))

        }

    }

    #[cfg(test)]
    mod tests {
        use core::panic;

        use super::*;

        #[test]
        fn recovery_signature() {
            // R+S actually
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
            use pink_web3::transports::{
                pink_http::{PinkHttp},
                resolve_ready
            };

            let phttp = PinkHttp::new("https://moonbase-alpha.public.blastapi.io");
            let eth = Eth::new(phttp);

            let addrs_hex = hex_literal::hex!("1b63b10dC015bbcaC201490CA286e844bf7c0ff1");
            let address: Address = Address::from_slice(&addrs_hex);
            let contract = Contract::from_json(eth, address, include_bytes!("../abi/erc721_abi.json")).unwrap();

            let query = "ownerOf";
            let result1: Address = resolve_ready(contract.query(&query, (U256::from(1), ), None, Options::default(), None)).unwrap();
            assert_eq!("0xa257f4ef17c81eb4d15a741a8d09e1ebb3953202", format!("{:?}", result1));

            let result2: Address = resolve_ready(contract.query(&query, (U256::from(2), ), None, Options::default(), None)).unwrap();
            assert_eq!("0xc9c7731fab51730224d5a9ec433f59433eb35166", format!("{:?}", result2));
        }
        
        #[test]
        fn test_download_files_ipfs() {
            pink_extension_runtime::mock_ext::mock_all_ext();
            let response = http_get!("https://ipfs2.apillon.io/ipfs/QmQLuYkRADePaWJtG6vxXaxurMgeuhcbEkRfC4xW6ceDFQ");
            assert_eq!(response.status_code, 200);
        }

        #[test]
        
        fn test_upload_files_ipfs() {
            pink_extension_runtime::mock_ext::mock_all_ext();
            use crate::{
                create_file_payload,
                create_ipfs_sync,
                UploadedFile,
                StorageResponse
            };
            
            let content_type = String::from("text/html");
            let bucket_uuid: String = String::from("10268b28-684e-42a1-a037-5ce3663e7827");
            let file_name: String = String::from("IamTheFilerus.txt");
            let file_content: String = String::from("Goo goo g'joob, goo goo goo g'joob");

            // ** UPLOAD FILE TO APILLON STORAGE ** //  
            let url_f_upload = format!("https://api-dev.apillon.io/storage/{}/upload", bucket_uuid);
            let url_get_content: String = format!("https://api-dev.apillon.io/storage/{}/content", bucket_uuid);
            
            let file = UploadedFile { fileName: String::from(file_name), contentType: content_type };
            let json_data = create_file_payload(file);

            let mut output_buffer = [0u8; 68];
            let message = "44f2b448-b89c-42df-afd2-c487d9a7b4a4:@AUAY0DHm86P";
            let encoded = b64encode(&message.as_bytes(), &mut output_buffer).ok().unwrap();
            let authorization = format!("Basic {}", String::from_utf8_lossy(encoded));
            let content_type = format!("application/json");

            let headers: Vec<(String, String)> = vec![
                ("Authorization".into(), authorization),
                ("Content-Type".into(), content_type)
            ];

            // assert_ne!(headers, headers);
            let response = http_get!(url_get_content, headers.clone());
            assert_eq!(response.status_code, 200);

            let response = http_post!(url_f_upload, json_data, headers.clone());

            let resp_body_str = match String::from_utf8(response.body) {
                Ok(r) => r,
                Err(e) => panic!("Mja, error, kaj ces {}", e),
            };

            assert_eq!(response.status_code, 201);
            let resp: StorageResponse = pink_json::from_str(&resp_body_str).unwrap();
            let file = &resp.data.files[0];
            let url_upload_s3: String = format!("{}", file.url);
            let content = file_content.as_bytes();
            let origin = String::from("https://app-dev.apillon.io/");

            let content_type = format!("text/plain");
            let headers_s3: Vec<(String, String)> = vec![
                ("Content-Type".into(), content_type),
                ("Referer".into(), origin.clone()),
                ("Origin".into(), origin.clone())
            ];

            let response = http_put!(url_upload_s3, *content, headers_s3);
            assert_eq!(response.status_code, 200);

            // ** TRIGGER UPLOAD TO IPFS (From Apillon storage) ** //
            let url_sync_ipfs = format!(
                "https://api-dev.apillon.io/storage/{}/upload/{}/end", 
                bucket_uuid, resp.data.sessionUuid);

            let ipfs_sync_json = create_ipfs_sync(true);

            let response = http_post!(url_sync_ipfs, ipfs_sync_json, headers.clone());
            assert_eq!(response.status_code, 200);

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