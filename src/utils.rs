use alloc::{vec, vec::Vec, string::String, borrow::ToOwned};

use ink_env::{ecdsa_recover, ecdsa_to_eth_address};
use pink_web3::types::Address;
use pink_web3::api::{Eth, Namespace};
use pink_web3::contract::Contract;
use pink_web3::transports::pink_http::PinkHttp;


pub mod utils {

    use super::*;

    pub fn recover_acc_address(sig: Vec<u8>, msg: Vec<u8>) -> String {
        let mut signature = [0u8; 65];
        hex::decode_to_slice(sig, &mut signature as &mut [u8; 65]).unwrap();
    
        let mut message = [0u8; 32];
        hex::decode_to_slice(msg, &mut message as &mut [u8; 32]).unwrap();
    
        let mut pub_key = [0; 33];
        let mut address = [0; 20];
        ecdsa_recover(&signature, &message, &mut pub_key).unwrap();
        ecdsa_to_eth_address(&mut pub_key, &mut address).unwrap();

        let addrs = String::from_utf8_lossy(&address).as_ref().to_owned();
        hex::encode(addrs)
    }

    // pub fn create_contract_interface(contract_address: Vec<u8>) {
    //     let phttp = PinkHttp::new("https://moonbase-alpha.public.blastapi.io");
    //     let eth = Eth::new(phttp);
        
    //     let mut addrs_hex = [0u8; 20];
    //     hex::decode_to_slice(contract_address, &mut addrs_hex).unwrap();
    //     let address: Address = Address::from_slice(&addrs_hex);
    //     let contract = Contract::from_json(eth, address, include_bytes!("../abi/erc721_abi.json")).unwrap();
    // }
}
