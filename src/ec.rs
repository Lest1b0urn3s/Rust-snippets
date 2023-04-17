

// // #[cfg(feature = "pure-rust")]
// // Only pure-rust feature supported, so no need to set this at compile time
// // use curve::{Affine, ECMultContext, ECMultGenContext, Field},

// // pub static ECMULT_CONTEXT: ECMultContext =
// //     unsafe { ECMultContext::new_from_raw(include!(concat!("../const/const.rs"))) };

// // pub static ECMULT_GEN_CONTEXT: ECMultGenContext =
// //     unsafe { ECMultGenContext::new_from_raw(include!(concat!("../const/const_gen.rs"))) };

// // /// Recover public key from a signed message, using the given context.
// // pub fn recover_with_context(
// //     message: &Message,
// //     signature: &Signature,
// //     recovery_id: &RecoveryId,
// //     context: &ECMultContext,
// // ) -> Result<PublicKey, Error> {
// //     context
// //         .recover_raw(&signature.r, &signature.s, recovery_id.0, &message.0)
// //         .map(PublicKey)
// // }

// // /// Recover public key from a signed message.
// // pub fn recover(
// //     message: &Message,
// //     signature: &Signature,
// //     recovery_id: &RecoveryId,
// // ) -> Result<PublicKey, Error> {
// //     recover_with_context(message, signature, recovery_id, &ECMULT_CONTEXT)
// // }

// // pub struct PublicKey(Affine);

// pub mod secp256k1 {
//     /// `libsecp256k1::Error`
//     use libsecp256k1;
//     use libsecp256k1::Error;

//     // pub fn verify_secret(secret: &[u8]) -> Result<(), Error> {
//     //     libsecp256k1::SecretKey::parse_slice(secret)?;
//     //     Ok(())
//     // }

//     // pub fn secret_to_public(secret: &[u8]) -> Result<[u8; 65], Error> {
//     //     let sec = libsecp256k1::SecretKey::parse_slice(secret)?;
//     //     let pubkey = libsecp256k1::PublicKey::from_secret_key(&sec);

//     //     Ok(pubkey.serialize())
//     // }

//     // /// Sign given 32-byte message hash with the key.
//     // pub fn sign(secret: &[u8], message: &[u8]) -> Result<(u8, [u8; 64]), Error> {
//     //     let sec = libsecp256k1::SecretKey::parse_slice(secret)?;
//     //     let msg = libsecp256k1::Message::parse_slice(message)?;

//     //     let (sig, rec_id) = libsecp256k1::sign(&msg, &sec);

//     //     Ok((rec_id.serialize(), sig.serialize()))
//     // }

//     fn to_signature(r: &[u8; 32], s: &[u8; 32]) -> Result<libsecp256k1::Signature, Error> {
//         let mut data = [0u8; 64];
//         data[0..32].copy_from_slice(r);
//         data[32..64].copy_from_slice(s);

//         Ok(libsecp256k1::Signature::parse_standard(&data)?)
//     }

//     // Result<[u8; 65], Error>
//     /// Recover the signer of the message.
//     pub fn recover_key(v: u8, r: &[u8; 32], s: &[u8; 32], message: &[u8]) -> Result<(), Error> {
//         let rec_id = libsecp256k1::RecoveryId::parse(v)?;
//         let sig = to_signature(r, s)?;
//         let msg = libsecp256k1::Message::parse_slice(message)?;
//         let pubkey = libsecp256k1::recover(&msg, &sig, &rec_id)?;

//         // pubkey.serialize()
//         Ok(())
//     }

//     // fn to_pubkey(public: &[u8]) -> Result<libsecp256k1::PublicKey, Error> {
//     //     let mut pubkey = [4u8; 65];
//     //     pubkey[1..65].copy_from_slice(public);
//     //     libsecp256k1::PublicKey::parse(&pubkey)
//     // }

//     // /// Checks ECDSA validity of `signature(r, s)` for `message` with `public` key.
//     // /// Returns `Ok(true)` on success.
//     // pub fn verify(public: &[u8], _v: u8, r: &[u8; 32], s: &[u8; 32], message: &[u8]) -> Result<bool, Error> {
//     //     let sig = to_signature(r, s)?;
//     //     let msg = libsecp256k1::Message::parse_slice(message)?;

//     //     Ok(libsecp256k1::verify(&msg, &sig, &to_pubkey(public)?))
//     // }
// }