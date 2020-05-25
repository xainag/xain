use crate::{
    mask::{MaskObject, MaskObjectBuffer},
    message::{utils::range, DecodeError, FromBytes, LengthValueBuffer, ToBytes},
    LocalSeedDict,
    ParticipantTaskSignature,
};
use anyhow::{anyhow, Context};
use std::{borrow::Borrow, ops::Range};

const SUM_SIGNATURE_RANGE: Range<usize> = range(0, ParticipantTaskSignature::LENGTH);
const UPDATE_SIGNATURE_RANGE: Range<usize> =
    range(SUM_SIGNATURE_RANGE.end, ParticipantTaskSignature::LENGTH);

#[derive(Clone, Debug)]
/// Wrapper around a buffer that contains an update message.
pub struct UpdateBuffer<T> {
    inner: T,
}

impl<T: AsRef<[u8]>> UpdateBuffer<T> {
    /// Perform bound checks on `bytes` to ensure its fields can be
    /// accessed without panicking, and return an `UpdateBuffer`.
    pub fn new(bytes: T) -> Result<Self, DecodeError> {
        let buffer = Self { inner: bytes };
        buffer
            .check_buffer_length()
            .context("invalid UpdateBuffer")?;
        Ok(buffer)
    }

    /// Return an `UpdateBuffer` without performing any bound
    /// check. This means accessing the various fields may panic if
    /// the data is invalid.
    pub fn new_unchecked(bytes: T) -> Self {
        Self { inner: bytes }
    }

    /// Perform bound checks to ensure the fields can be accessed
    /// without panicking.
    pub fn check_buffer_length(&self) -> Result<(), DecodeError> {
        let len = self.inner.as_ref().len();
        // First, check the fixed size portion of the
        // header. UPDATE_SIGNATURE_RANGE is the last field
        if len < UPDATE_SIGNATURE_RANGE.end {
            return Err(anyhow!(
                "invalid buffer length: {} < {}",
                len,
                UPDATE_SIGNATURE_RANGE.end
            ));
        }

        // Check the length of the length of the masked model field
        let _ = MaskObjectBuffer::new(&self.inner.as_ref()[self.masked_model_offset()..])
            .context("invalid masked model field")?;

        // Check the length of the local seed dictionary field
        let _ = LengthValueBuffer::new(&self.inner.as_ref()[self.local_seed_dict_offset()..])
            .context("invalid local seed dictionary length")?;

        Ok(())
    }

    /// Get the offset of the masked model field
    fn masked_model_offset(&self) -> usize {
        UPDATE_SIGNATURE_RANGE.end
    }

    /// Get the offset of the local seed dictionary field
    fn local_seed_dict_offset(&self) -> usize {
        let masked_model =
            MaskObjectBuffer::new_unchecked(&self.inner.as_ref()[self.masked_model_offset()..]);
        self.masked_model_offset() + masked_model.len()
    }
}

impl<'a, T: AsRef<[u8]> + ?Sized> UpdateBuffer<&'a T> {
    /// Get the sum signature field
    ///
    /// # Panic
    ///
    /// This may panic if the underlying buffer does not represent a
    /// valid update. If `self.check_buffer_length()` returned
    /// `Ok(())` this method is guaranteed not to panic.
    pub fn sum_signature(&self) -> &'a [u8] {
        &self.inner.as_ref()[SUM_SIGNATURE_RANGE]
    }

    /// Get the update signature field
    ///
    /// # Panic
    ///
    /// This may panic if the underlying buffer does not represent a
    /// valid update. If `self.check_buffer_length()` returned
    /// `Ok(())` this method is guaranteed not to panic.
    pub fn update_signature(&self) -> &'a [u8] {
        &self.inner.as_ref()[UPDATE_SIGNATURE_RANGE]
    }

    /// Get a slice that starts at the beginning of the masked model
    /// field
    ///
    /// # Panic
    ///
    /// This may panic if the underlying buffer does not represent a
    /// valid update. If `self.check_buffer_length()` returned
    /// `Ok(())` this method is guaranteed not to panic.
    pub fn masked_model(&self) -> &'a [u8] {
        let offset = self.masked_model_offset();
        &self.inner.as_ref()[offset..]
    }

    /// Get a slice that starts at the beginning og the local seed
    /// dictionary field
    ///
    /// # Panic
    ///
    /// This may panic if the underlying buffer does not represent a
    /// valid update. If `self.check_buffer_length()` returned
    /// `Ok(())` this method is guaranteed not to panic.
    pub fn local_seed_dict(&self) -> &'a [u8] {
        let offset = self.local_seed_dict_offset();
        &self.inner.as_ref()[offset..]
    }
}

impl<T: AsRef<[u8]> + AsMut<[u8]>> UpdateBuffer<T> {
    /// Get a mutable reference to the sum signature field
    ///
    /// # Panic
    ///
    /// This may panic if the underlying buffer does not represent a
    /// valid update. If `self.check_buffer_length()` returned
    /// `Ok(())` this method is guaranteed not to panic.
    pub fn sum_signature_mut(&mut self) -> &mut [u8] {
        &mut self.inner.as_mut()[SUM_SIGNATURE_RANGE]
    }

    /// Get a mutable reference to the update signature field
    ///
    /// # Panic
    ///
    /// This may panic if the underlying buffer does not represent a
    /// valid update. If `self.check_buffer_length()` returned
    /// `Ok(())` this method is guaranteed not to panic.
    pub fn update_signature_mut(&mut self) -> &mut [u8] {
        &mut self.inner.as_mut()[UPDATE_SIGNATURE_RANGE]
    }

    /// Get a mutable slice that starts at the beginning of the masked
    /// model field
    ///
    /// # Panic
    ///
    /// This may panic if the underlying buffer does not represent a
    /// valid update. If `self.check_buffer_length()` returned
    /// `Ok(())` this method is guaranteed not to panic.
    pub fn masked_model_mut(&mut self) -> &mut [u8] {
        let offset = self.masked_model_offset();
        &mut self.inner.as_mut()[offset..]
    }

    /// Get a mutable slice that starts at the beginning of the local
    /// seed dictionary field
    ///
    /// # Panic
    ///
    /// This may panic if the underlying buffer does not represent a
    /// valid update. If `self.check_buffer_length()` returned
    /// `Ok(())` this method is guaranteed not to panic.
    pub fn local_seed_dict_mut(&mut self) -> &mut [u8] {
        let offset = self.local_seed_dict_offset();
        &mut self.inner.as_mut()[offset..]
    }
}

#[derive(Debug, Eq, PartialEq, Clone)]
/// High level representation of an update message. These messages are
/// sent by update partipants during the update phase.
pub struct Update<D, M> {
    /// Signature of the round seed and the word "sum", used to
    /// determine whether a participant is selected for the sum task
    pub sum_signature: ParticipantTaskSignature,
    /// Signature of the round seed and the word "update", used to
    /// determine whether a participant is selected for the update
    /// task
    pub update_signature: ParticipantTaskSignature,
    /// Model trained by an update participant, masked with randomness
    /// derived from the participant seed
    pub masked_model: M,
    /// A dictionary that contains the seed used to mask
    /// `masked_model`, encrypted with the ephemeral public key of
    /// each sum participant
    pub local_seed_dict: D,
}

impl<D, M> ToBytes for Update<D, M>
where
    D: Borrow<LocalSeedDict>,
    M: Borrow<MaskObject>,
{
    fn buffer_length(&self) -> usize {
        UPDATE_SIGNATURE_RANGE.end
            + self.masked_model.borrow().buffer_length()
            + self.local_seed_dict.borrow().buffer_length()
    }

    fn to_bytes<T: AsMut<[u8]>>(&self, buffer: &mut T) {
        let mut writer = UpdateBuffer::new_unchecked(buffer.as_mut());
        self.sum_signature.to_bytes(&mut writer.sum_signature_mut());
        self.update_signature
            .to_bytes(&mut writer.update_signature_mut());
        self.masked_model
            .borrow()
            .to_bytes(&mut writer.masked_model_mut());
        self.local_seed_dict
            .borrow()
            .to_bytes(&mut writer.local_seed_dict_mut());
    }
}

/// Owned version of a [`Update`]
pub type UpdateOwned = Update<LocalSeedDict, MaskObject>;

impl FromBytes for UpdateOwned {
    fn from_bytes<T: AsRef<[u8]>>(buffer: &T) -> Result<Self, DecodeError> {
        let reader = UpdateBuffer::new(buffer.as_ref())?;
        Ok(Self {
            sum_signature: ParticipantTaskSignature::from_bytes(&reader.sum_signature())
                .context("invalid sum signature")?,
            update_signature: ParticipantTaskSignature::from_bytes(&reader.update_signature())
                .context("invalid update signature")?,
            masked_model: MaskObject::from_bytes(&reader.masked_model())
                .context("invalid masked model")?,
            local_seed_dict: LocalSeedDict::from_bytes(&reader.local_seed_dict())
                .context("invalid local seed dictionary")?,
        })
    }
}

#[cfg(test)]
pub(crate) mod tests_helpers {
    use super::*;
    use crate::{
        crypto::ByteObject,
        mask::{EncryptedMaskSeed, MaskObject},
        SumParticipantPublicKey,
    };
    use std::convert::TryFrom;

    pub fn sum_signature() -> (ParticipantTaskSignature, Vec<u8>) {
        let bytes = vec![0x33; 64];
        let signature = ParticipantTaskSignature::from_slice(&bytes[..]).unwrap();
        (signature, bytes)
    }

    pub fn update_signature() -> (ParticipantTaskSignature, Vec<u8>) {
        let bytes = vec![0x44; 64];
        let signature = ParticipantTaskSignature::from_slice(&bytes[..]).unwrap();
        (signature, bytes)
    }

    pub fn masked_model() -> (MaskObject, Vec<u8>) {
        use crate::mask::object::serialization::tests::{bytes, object};
        (object(), bytes())
    }

    pub fn local_seed_dict() -> (LocalSeedDict, Vec<u8>) {
        let mut local_seed_dict = LocalSeedDict::new();
        let mut bytes = vec![];

        // Length (32+80) * 2 + 4 = 228
        bytes.extend(vec![0x00, 0x00, 0x00, 0xe4]);

        bytes.extend(vec![0x55; SumParticipantPublicKey::LENGTH]);
        bytes.extend(vec![0x66; EncryptedMaskSeed::LENGTH]);
        local_seed_dict.insert(
            SumParticipantPublicKey::from_slice(vec![0x55; 32].as_slice()).unwrap(),
            EncryptedMaskSeed::try_from(vec![0x66; EncryptedMaskSeed::LENGTH]).unwrap(),
        );

        // Second entry
        bytes.extend(vec![0x77; SumParticipantPublicKey::LENGTH]);
        bytes.extend(vec![0x88; EncryptedMaskSeed::LENGTH]);
        local_seed_dict.insert(
            SumParticipantPublicKey::from_slice(vec![0x77; 32].as_slice()).unwrap(),
            EncryptedMaskSeed::try_from(vec![0x88; EncryptedMaskSeed::LENGTH]).unwrap(),
        );

        (local_seed_dict, bytes)
    }

    pub fn update() -> (UpdateOwned, Vec<u8>) {
        let mut bytes = sum_signature().1;
        bytes.extend(update_signature().1);
        bytes.extend(masked_model().1);
        bytes.extend(local_seed_dict().1);

        let update = UpdateOwned {
            sum_signature: sum_signature().0,
            update_signature: update_signature().0,
            masked_model: masked_model().0,
            local_seed_dict: local_seed_dict().0,
        };
        (update, bytes)
    }
}

#[cfg(test)]
pub(crate) mod tests {
    pub(crate) use super::tests_helpers as helpers;
    use super::*;

    #[test]
    fn buffer_read() {
        let bytes = helpers::update().1;
        let buffer = UpdateBuffer::new(&bytes).unwrap();
        assert_eq!(
            buffer.sum_signature(),
            helpers::sum_signature().1.as_slice()
        );
        assert_eq!(
            buffer.update_signature(),
            helpers::update_signature().1.as_slice()
        );
        let expected = helpers::masked_model().1;
        assert_eq!(&buffer.masked_model()[..expected.len()], &expected[..]);
        assert_eq!(buffer.local_seed_dict(), &helpers::local_seed_dict().1[..]);
    }

    #[test]
    fn decode_invalid_seed_dict() {
        let mut invalid = helpers::local_seed_dict().1;
        // This truncates the last entry of the seed dictionary
        invalid[3] = 0xe3;
        let mut bytes = vec![];
        bytes.extend(helpers::sum_signature().1);
        bytes.extend(helpers::update_signature().1);
        bytes.extend(helpers::masked_model().1);
        bytes.extend(invalid);

        let e = UpdateOwned::from_bytes(&bytes).unwrap_err();
        let cause = e.source().unwrap().to_string();
        assert_eq!(
            cause,
            "invalid local seed dictionary: trailing bytes".to_string()
        );
    }

    #[test]
    fn decode() {
        let (update, bytes) = helpers::update();
        let parsed = UpdateOwned::from_bytes(&bytes).unwrap();
        assert_eq!(parsed, update);
    }

    #[test]
    fn encode() {
        let (update, bytes) = helpers::update();
        assert_eq!(update.buffer_length(), bytes.len());
        let mut buf = vec![0xff; update.buffer_length()];
        update.to_bytes(&mut buf);
        // The order in which the hashmap is serialized is not
        // guaranteed, but we chose our key/values such that they are
        // sorted.
        //
        // First compute the offset at which the local seed dict value
        // starts: two signature (64 bytes), the masked model (32
        // bytes), the length field (4 bytes)
        let offset = 64 * 2 + 32 + 4;
        // Sort the end of the buffer
        (&mut buf[offset..]).sort();
        assert_eq!(buf, bytes);
    }
}
