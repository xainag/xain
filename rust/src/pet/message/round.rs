#![allow(dead_code)] // temporary

use std::ops::Range;

use sodiumoxide::crypto::{box_, sealedbox, sign};

use super::{ROUND_TAG, TAG_RANGE};
use crate::pet::PetError;

// round box field ranges
const ENCR_PK_RANGE: Range<usize> = 1..33; // 32 bytes
const SIGN_PK_RANGE: Range<usize> = 33..65; // 32 bytes

/// Mutable and immutable buffer access to round box fields.
struct RoundBoxBuffer<B> {
    bytes: B,
}

impl RoundBoxBuffer<Vec<u8>> {
    /// Create an empty round box buffer of size `len`.
    fn new(len: usize) -> Self {
        Self {
            bytes: vec![0_u8; len],
        }
    }
}

impl<B: AsRef<[u8]>> RoundBoxBuffer<B> {
    /// Create a round box buffer from `bytes`. Fails if the `bytes` don't conform to the expected
    /// round box length `exp_len`.
    fn from(bytes: B, exp_len: usize) -> Result<Self, PetError> {
        (bytes.as_ref().len() == exp_len)
            .then_some(Self { bytes })
            .ok_or(PetError::InvalidMessage)
    }
}

impl<'b, B: AsRef<[u8]> + ?Sized> RoundBoxBuffer<&'b B> {
    /// Access the tag field of the round box buffer by reference.
    fn tag(&self) -> &'b [u8] {
        &self.bytes.as_ref()[TAG_RANGE]
    }

    /// Access the public encryption key field of the round box buffer by reference.
    fn encr_pk(&self) -> &'b [u8] {
        &self.bytes.as_ref()[ENCR_PK_RANGE]
    }

    /// Access the public signature key field of the round box buffer by reference.
    fn sign_pk(&self) -> &'b [u8] {
        &self.bytes.as_ref()[SIGN_PK_RANGE]
    }
}

impl<B: AsMut<[u8]>> RoundBoxBuffer<B> {
    /// Access the tag field of the round box buffer by mutable reference.
    fn tag_mut(&mut self) -> &mut [u8] {
        &mut self.bytes.as_mut()[TAG_RANGE]
    }

    /// Access the public encryption key field of the round box buffer by mutable reference.
    fn encr_pk_mut(&mut self) -> &mut [u8] {
        &mut self.bytes.as_mut()[ENCR_PK_RANGE]
    }

    /// Access the public signature key field of the round box buffer by mutable reference.
    fn sign_pk_mut(&mut self) -> &mut [u8] {
        &mut self.bytes.as_mut()[SIGN_PK_RANGE]
    }
}

/// Encryption and decryption of round boxes.
pub struct RoundBox<E, S> {
    encr_pk: E,
    sign_pk: S,
}

impl<'b> RoundBox<&'b box_::PublicKey, &'b sign::PublicKey> {
    /// Create a round box.
    pub fn new(encr_pk: &'b box_::PublicKey, sign_pk: &'b sign::PublicKey) -> Self {
        Self { encr_pk, sign_pk }
    }

    /// Get the length of the serialized round box.
    pub fn len() -> usize {
        1 + box_::PUBLICKEYBYTES + sign::PUBLICKEYBYTES // 65 bytes
    }

    /// Serialize the round box to bytes.
    fn serialize(&self) -> Vec<u8> {
        let mut buffer = RoundBoxBuffer::new(Self::len());
        buffer.tag_mut().copy_from_slice([ROUND_TAG; 1].as_ref());
        buffer.encr_pk_mut().copy_from_slice(self.encr_pk.as_ref());
        buffer.sign_pk_mut().copy_from_slice(self.sign_pk.as_ref());
        buffer.bytes
    }

    /// Encrypt the round box.
    pub fn seal(&self, pk: &box_::PublicKey) -> Vec<u8> {
        let bytes = self.serialize();
        sealedbox::seal(&bytes, pk)
    }
}

impl RoundBox<box_::PublicKey, sign::PublicKey> {
    /// Get the expected length of a serialized round box.
    pub fn exp_len() -> usize {
        1 + box_::PUBLICKEYBYTES + sign::PUBLICKEYBYTES // 65 bytes
    }

    /// Deserialize a round box from bytes. Fails if the `bytes` don't conform to the expected
    /// round box length.
    fn deserialize(bytes: &[u8]) -> Result<Self, PetError> {
        let buffer = RoundBoxBuffer::from(bytes, Self::exp_len())?;
        (buffer.tag() == [ROUND_TAG])
            .then_some(())
            .ok_or(PetError::InvalidMessage)?;
        let encr_pk = box_::PublicKey::from_slice(buffer.encr_pk()).unwrap();
        let sign_pk = sign::PublicKey::from_slice(buffer.sign_pk()).unwrap();
        Ok(Self { encr_pk, sign_pk })
    }

    /// Decrypt a round box. Fails if the `bytes` don't conform to a valid encrypted round box.
    pub fn open(
        bytes: &[u8],
        pk: &box_::PublicKey,
        sk: &box_::SecretKey,
    ) -> Result<Self, PetError> {
        let bytes = sealedbox::open(bytes, pk, sk).or(Err(PetError::InvalidMessage))?;
        Self::deserialize(&bytes)
    }

    /// Get a reference to the public encryption key.
    pub fn encr_pk(&self) -> &box_::PublicKey {
        &self.encr_pk
    }

    /// Get a reference to the public signature key.
    pub fn sign_pk(&self) -> &sign::PublicKey {
        &self.sign_pk
    }
}
