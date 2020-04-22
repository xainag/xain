use std::{
    borrow::Borrow,
    convert::{TryFrom, TryInto},
    ops::Range,
};

use sodiumoxide::crypto::{sealedbox, sign};

use super::{MessageBuffer, Tag, LEN_BYTES};
use crate::{
    certificate::Certificate,
    mask::Mask,
    CoordinatorPublicKey,
    CoordinatorSecretKey,
    ParticipantTaskSignature,
    PetError,
    SumParticipantPublicKey,
    SumParticipantSecretKey,
};

#[derive(Clone, Debug)]
/// Access to sum2 message buffer fields.
struct Sum2MessageBuffer<B> {
    bytes: B,
    certificate_range: Range<usize>,
    mask_range: Range<usize>,
}

impl Sum2MessageBuffer<Vec<u8>> {
    /// Create an empty sum2 message buffer.
    fn new(certificate_len: usize, mask_len: usize) -> Self {
        let bytes = [
            vec![0_u8; Self::SUM_SIGNATURE_RANGE.end],
            certificate_len.to_le_bytes().to_vec(),
            mask_len.to_le_bytes().to_vec(),
            vec![0_u8; certificate_len + mask_len],
        ]
        .concat();
        let certificate_range =
            Self::MASK_LEN_RANGE.end..Self::MASK_LEN_RANGE.end + certificate_len;
        let mask_range = certificate_range.end..certificate_range.end + mask_len;
        Self {
            bytes,
            certificate_range,
            mask_range,
        }
    }
}

impl TryFrom<Vec<u8>> for Sum2MessageBuffer<Vec<u8>> {
    type Error = PetError;

    /// Create a sum2 message buffer from `bytes`. Fails if the length of the input is invalid.
    fn try_from(bytes: Vec<u8>) -> Result<Self, Self::Error> {
        let mut buffer = Self {
            bytes,
            certificate_range: 0..0,
            mask_range: 0..0,
        };
        if buffer.len() >= Self::MASK_LEN_RANGE.end {
            // safe unwraps: lengths of slices are guaranteed by constants
            buffer.certificate_range = Self::MASK_LEN_RANGE.end
                ..Self::MASK_LEN_RANGE.end
                    + usize::from_le_bytes(buffer.certificate_len().try_into().unwrap());
            buffer.mask_range = buffer.certificate_range.end
                ..buffer.certificate_range.end
                    + usize::from_le_bytes(buffer.mask_len().try_into().unwrap());
        } else {
            return Err(PetError::InvalidMessage);
        }
        if buffer.len() == buffer.mask_range.end {
            Ok(buffer)
        } else {
            Err(PetError::InvalidMessage)
        }
    }
}

impl<B: AsRef<[u8]> + AsMut<[u8]>> MessageBuffer for Sum2MessageBuffer<B> {
    /// Get a reference to the message buffer.
    fn bytes(&'_ self) -> &'_ [u8] {
        self.bytes.as_ref()
    }

    /// Get a mutable reference to the message buffer.
    fn bytes_mut(&mut self) -> &mut [u8] {
        self.bytes.as_mut()
    }
}

impl<B: AsRef<[u8]> + AsMut<[u8]>> Sum2MessageBuffer<B> {
    /// Get the range of the certificate length field.
    const CERTIFICATE_LEN_RANGE: Range<usize> =
        Self::SUM_SIGNATURE_RANGE.end..Self::SUM_SIGNATURE_RANGE.end + LEN_BYTES;

    /// Get the range of the masked model length field.
    const MASK_LEN_RANGE: Range<usize> =
        Self::CERTIFICATE_LEN_RANGE.end..Self::CERTIFICATE_LEN_RANGE.end + LEN_BYTES;

    /// Get a reference to the certificate length field.
    fn certificate_len(&'_ self) -> &'_ [u8] {
        &self.bytes()[Self::CERTIFICATE_LEN_RANGE]
    }

    /// Get a reference to the certificate field.
    fn certificate(&'_ self) -> &'_ [u8] {
        &self.bytes()[self.certificate_range.clone()]
    }

    /// Get a mutable reference to the certificate field.
    fn certificate_mut(&mut self) -> &mut [u8] {
        let range = self.certificate_range.clone();
        &mut self.bytes_mut()[range]
    }

    /// Get a reference to the mask length field.
    fn mask_len(&'_ self) -> &'_ [u8] {
        &self.bytes()[Self::MASK_LEN_RANGE]
    }

    /// Get a reference to the mask field.
    fn mask(&'_ self) -> &'_ [u8] {
        &self.bytes()[self.mask_range.clone()]
    }

    /// Get a mutable reference to the mask field.
    fn mask_mut(&mut self) -> &mut [u8] {
        let range = self.mask_range.clone();
        &mut self.bytes_mut()[range]
    }
}

#[derive(Clone, Debug, PartialEq)]
/// Encryption and decryption of sum2 messages.
pub struct Sum2Message<K, S, C, M>
where
    K: Borrow<SumParticipantPublicKey>,
    S: Borrow<ParticipantTaskSignature>,
    C: Borrow<Certificate>,
    M: Borrow<Mask>,
{
    pk: K,
    sum_signature: S,
    certificate: C,
    mask: M,
}

impl<K, S, C, M> Sum2Message<K, S, C, M>
where
    K: Borrow<SumParticipantPublicKey>,
    S: Borrow<ParticipantTaskSignature>,
    C: Borrow<Certificate>,
    M: Borrow<Mask>,
{
    /// Create a sum2 message from its parts.
    pub fn from_parts(pk: K, sum_signature: S, certificate: C, mask: M) -> Self {
        Self {
            pk,
            sum_signature,
            certificate,
            mask,
        }
    }

    /// Serialize the sum2 message into a buffer.
    fn serialize<B: AsRef<[u8]> + AsMut<[u8]>>(
        &self,
        buffer: &mut Sum2MessageBuffer<B>,
        pk: &CoordinatorPublicKey,
    ) {
        buffer.tag_mut().copy_from_slice([Tag::Sum2 as u8].as_ref());
        buffer.coord_pk_mut().copy_from_slice(pk.borrow().as_ref());
        buffer
            .part_pk_mut()
            .copy_from_slice(self.pk.borrow().as_ref());
        buffer
            .sum_signature_mut()
            .copy_from_slice(self.sum_signature.borrow().as_ref());
        buffer
            .certificate_mut()
            .copy_from_slice(self.certificate.borrow().as_ref());
        buffer
            .mask_mut()
            .copy_from_slice(self.mask.borrow().as_ref());
    }

    /// Sign and encrypt the sum2message.
    pub fn seal(&self, sk: &SumParticipantSecretKey, pk: &CoordinatorPublicKey) -> Vec<u8> {
        let mut buffer =
            Sum2MessageBuffer::new(self.certificate.borrow().len(), self.mask.borrow().len());
        self.serialize(&mut buffer, pk);
        let signature = sign::sign_detached(buffer.message(), sk);
        buffer.signature_mut().copy_from_slice(signature.as_ref());
        sealedbox::seal(buffer.bytes(), pk)
    }
}

impl Sum2Message<SumParticipantPublicKey, ParticipantTaskSignature, Certificate, Mask> {
    /// Deserialize a sum2 message from a buffer.
    fn deserialize(buffer: Sum2MessageBuffer<Vec<u8>>) -> Self {
        // safe unwraps: lengths of slices are guaranteed by constants
        let pk = sign::PublicKey::from_slice(buffer.part_pk()).unwrap();
        let sum_signature = sign::Signature::from_slice(buffer.sum_signature()).unwrap();
        let certificate = buffer.certificate().into();
        let mask = buffer.mask().into();
        Self {
            pk,
            sum_signature,
            certificate,
            mask,
        }
    }

    /// Decrypt and verify a sum2 message. Fails if decryption or validation fails.
    pub fn open(
        bytes: &[u8],
        pk: &CoordinatorPublicKey,
        sk: &CoordinatorSecretKey,
    ) -> Result<Self, PetError> {
        let buffer = Sum2MessageBuffer::try_from(
            sealedbox::open(bytes, pk, sk).or(Err(PetError::InvalidMessage))?,
        )?;
        if buffer.tag() == [Tag::Sum2 as u8]
            && buffer.coord_pk() == pk.as_ref()
            && sign::verify_detached(
                // safe unwraps: lengths of slices are guaranteed by constants
                &sign::Signature::from_slice(buffer.signature()).unwrap(),
                buffer.message(),
                &sign::PublicKey::from_slice(buffer.part_pk()).unwrap(),
            )
        {
            Ok(Self::deserialize(buffer))
        } else {
            Err(PetError::InvalidMessage)
        }
    }

    /// Get a reference to the public signature key.
    pub fn pk(&self) -> &SumParticipantPublicKey {
        &self.pk
    }

    /// Get a reference to the sum signature.
    pub fn sum_signature(&self) -> &ParticipantTaskSignature {
        &self.sum_signature
    }

    /// Get a reference to the certificate.
    pub fn certificate(&self) -> &Certificate {
        &self.certificate
    }

    /// Get a reference to the mask.
    pub fn mask(&self) -> &Mask {
        &self.mask
    }
}

#[cfg(test)]
mod tests {
    use sodiumoxide::{crypto::box_, randombytes::randombytes};

    use super::*;
    use crate::message::{PK_BYTES, SIGNATURE_BYTES, TAG_BYTES};

    fn auxiliary_bytes() -> Vec<u8> {
        [
            randombytes(193),
            (32 as usize).to_le_bytes().to_vec(),
            (32 as usize).to_le_bytes().to_vec(),
            randombytes(64),
        ]
        .concat()
    }

    type MB = Sum2MessageBuffer<Vec<u8>>;

    #[test]
    fn test_sum2messagebuffer_ranges() {
        assert_eq!(MB::SIGNATURE_RANGE, ..SIGNATURE_BYTES);
        assert_eq!(MB::MESSAGE_RANGE, SIGNATURE_BYTES..);
        assert_eq!(MB::TAG_RANGE, 64..64 + TAG_BYTES);
        assert_eq!(MB::COORD_PK_RANGE, 65..65 + PK_BYTES);
        assert_eq!(MB::PART_PK_RANGE, 97..97 + PK_BYTES);
        assert_eq!(MB::SUM_SIGNATURE_RANGE, 129..129 + SIGNATURE_BYTES);
        assert_eq!(MB::CERTIFICATE_LEN_RANGE, 193..193 + LEN_BYTES);
        assert_eq!(MB::MASK_LEN_RANGE, 193 + LEN_BYTES..193 + 2 * LEN_BYTES);
        let buffer = Sum2MessageBuffer::new(32, 32);
        assert_eq!(
            buffer.certificate_range,
            193 + 2 * LEN_BYTES..193 + 2 * LEN_BYTES + 32,
        );
        assert_eq!(
            buffer.mask_range,
            193 + 2 * LEN_BYTES + 32..193 + 2 * LEN_BYTES + 32 + 32,
        );
    }

    #[test]
    fn test_sum2messagebuffer_fields() {
        // new
        assert_eq!(
            Sum2MessageBuffer::new(32, 32).bytes,
            [
                vec![0_u8; 193],
                (32 as usize).to_le_bytes().to_vec(),
                (32 as usize).to_le_bytes().to_vec(),
                vec![0_u8; 64],
            ]
            .concat(),
        );

        // try from
        let mut bytes = auxiliary_bytes();
        let mut buffer = Sum2MessageBuffer::try_from(bytes.clone()).unwrap();
        assert_eq!(buffer.bytes, bytes);
        assert_eq!(
            Sum2MessageBuffer::try_from(vec![0_u8; 0]).unwrap_err(),
            PetError::InvalidMessage,
        );

        // length
        assert_eq!(buffer.len(), 257 + 2 * LEN_BYTES);

        // signature
        assert_eq!(buffer.signature(), &bytes[MB::SIGNATURE_RANGE]);
        assert_eq!(buffer.signature_mut(), &mut bytes[MB::SIGNATURE_RANGE]);

        // message
        assert_eq!(buffer.message(), &bytes[MB::MESSAGE_RANGE]);

        // tag
        assert_eq!(buffer.tag(), &bytes[MB::TAG_RANGE]);
        assert_eq!(buffer.tag_mut(), &mut bytes[MB::TAG_RANGE]);

        // coordinator pk
        assert_eq!(buffer.coord_pk(), &bytes[MB::COORD_PK_RANGE]);
        assert_eq!(buffer.coord_pk_mut(), &mut bytes[MB::COORD_PK_RANGE]);

        // participant pk
        assert_eq!(buffer.part_pk(), &bytes[MB::PART_PK_RANGE]);
        assert_eq!(buffer.part_pk_mut(), &mut bytes[MB::PART_PK_RANGE]);

        // sum signature
        assert_eq!(buffer.sum_signature(), &bytes[MB::SUM_SIGNATURE_RANGE]);
        assert_eq!(
            buffer.sum_signature_mut(),
            &mut bytes[MB::SUM_SIGNATURE_RANGE],
        );

        // certificate
        assert_eq!(buffer.certificate_len(), &bytes[MB::CERTIFICATE_LEN_RANGE]);
        let range = buffer.certificate_range.clone();
        assert_eq!(buffer.certificate(), &bytes[range.clone()]);
        assert_eq!(buffer.certificate_mut(), &mut bytes[range]);

        // mask
        assert_eq!(buffer.mask_len(), &bytes[MB::MASK_LEN_RANGE]);
        let range = buffer.mask_range.clone();
        assert_eq!(buffer.mask(), &bytes[range.clone()]);
        assert_eq!(buffer.mask_mut(), &mut bytes[range]);
    }

    #[test]
    fn test_sum2message_serialize() {
        // from parts
        let pk = &sign::PublicKey::from_slice(&randombytes(32)).unwrap();
        let sum_signature = &sign::Signature::from_slice(&randombytes(64)).unwrap();
        let certificate = &Certificate::new();
        let mask = &randombytes(32).into();
        let msg = Sum2Message::from_parts(pk, sum_signature, certificate, mask);
        assert_eq!(
            msg.pk as *const sign::PublicKey,
            pk as *const sign::PublicKey,
        );
        assert_eq!(
            msg.sum_signature as *const sign::Signature,
            sum_signature as *const sign::Signature,
        );
        assert_eq!(
            msg.certificate as *const Certificate,
            certificate as *const Certificate,
        );
        assert_eq!(msg.mask as *const Mask, mask as *const Mask);

        // serialize
        let mut buffer = Sum2MessageBuffer::new(32, 32);
        let coord_pk = box_::PublicKey::from_slice(&randombytes(32)).unwrap();
        msg.serialize(&mut buffer, &coord_pk);
        assert_eq!(buffer.tag(), [Tag::Sum2 as u8].as_ref());
        assert_eq!(buffer.coord_pk(), coord_pk.as_ref());
        assert_eq!(buffer.part_pk(), pk.as_ref());
        assert_eq!(buffer.sum_signature(), sum_signature.as_ref());
        assert_eq!(
            buffer.certificate_len(),
            certificate.len().to_le_bytes().as_ref(),
        );
        assert_eq!(buffer.certificate(), certificate.as_ref());
        assert_eq!(buffer.mask_len(), mask.len().to_le_bytes().as_ref());
        assert_eq!(buffer.mask(), mask.as_ref());
    }

    #[test]
    fn test_sum2message_deserialize() {
        // deserialize
        let bytes = auxiliary_bytes();
        let buffer = Sum2MessageBuffer::try_from(bytes.clone()).unwrap();
        let msg = Sum2Message::deserialize(buffer.clone());
        assert_eq!(
            msg.pk(),
            &sign::PublicKey::from_slice(&bytes[MB::PART_PK_RANGE]).unwrap(),
        );
        assert_eq!(
            msg.sum_signature(),
            &sign::Signature::from_slice(&bytes[MB::SUM_SIGNATURE_RANGE]).unwrap(),
        );
        assert_eq!(
            msg.certificate(),
            &bytes[buffer.certificate_range.clone()].into(),
        );
        assert_eq!(msg.mask(), &bytes[buffer.mask_range.clone()].into());
    }

    #[test]
    fn test_sum2message() {
        // seal
        let (pk, sk) = sign::gen_keypair();
        let sum_signature = sign::Signature::from_slice(&randombytes(64)).unwrap();
        let certificate = Certificate::new();
        let mask = randombytes(32).into();
        let (coord_pk, coord_sk) = box_::gen_keypair();
        let bytes =
            Sum2Message::from_parts(&pk, &sum_signature, &certificate, &mask).seal(&sk, &coord_pk);

        // open
        let msg = Sum2Message::open(&bytes, &coord_pk, &coord_sk).unwrap();
        assert_eq!(msg.pk(), &pk);
        assert_eq!(msg.sum_signature(), &sum_signature);
        assert_eq!(msg.certificate(), &certificate);
        assert_eq!(msg.mask(), &mask);

        // wrong signature
        let bytes = auxiliary_bytes();
        let mut buffer = Sum2MessageBuffer::try_from(bytes).unwrap();
        let msg = Sum2Message::from_parts(&pk, &sum_signature, &certificate, &mask);
        msg.serialize(&mut buffer, &coord_pk);
        let bytes = sealedbox::seal(buffer.bytes(), &coord_pk);
        assert_eq!(
            Sum2Message::open(&bytes, &coord_pk, &coord_sk).unwrap_err(),
            PetError::InvalidMessage,
        );

        // wrong receiver
        msg.serialize(
            &mut buffer,
            &box_::PublicKey::from_slice(&randombytes(32)).unwrap(),
        );
        let bytes = sealedbox::seal(buffer.bytes(), &coord_pk);
        assert_eq!(
            Sum2Message::open(&bytes, &coord_pk, &coord_sk).unwrap_err(),
            PetError::InvalidMessage,
        );

        // wrong tag
        buffer.tag_mut().copy_from_slice([Tag::None as u8].as_ref());
        let bytes = sealedbox::seal(buffer.bytes(), &coord_pk);
        assert_eq!(
            Sum2Message::open(&bytes, &coord_pk, &coord_sk).unwrap_err(),
            PetError::InvalidMessage,
        );

        // wrong length
        assert_eq!(
            Sum2Message::open([0_u8; 0].as_ref(), &coord_pk, &coord_sk).unwrap_err(),
            PetError::InvalidMessage,
        );
    }
}
