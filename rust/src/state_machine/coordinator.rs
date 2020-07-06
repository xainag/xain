//! Coordinator state and round parameter types.
use std::collections::HashMap;

use sodiumoxide::{self, crypto::box_, randombytes::randombytes};

use crate::{
    crypto::{encrypt::EncryptKeyPair, ByteObject},
    mask::{config::MaskConfig, object::MaskObject},
    settings::{MaskSettings, PetSettings},
    state_machine::events::{EventPublisher, EventSubscriber, PhaseEvent},
    CoordinatorPublicKey,
};

/// The round parameters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoundParameters {
    /// The public key of the coordinator used for encryption.
    pub pk: CoordinatorPublicKey,
    /// Fraction of participants to be selected for the sum task.
    pub sum: f64,
    /// Fraction of participants to be selected for the update task.
    pub update: f64,
    /// The random round seed.
    pub seed: RoundSeed,
}

/// The coordinator state.
pub struct CoordinatorState {
    /// The credentials of the coordinator.
    pub keys: EncryptKeyPair,
    /// The round parameters.
    pub round_params: RoundParameters,
    /// The minimum of required sum/sum2 messages.
    pub min_sum: usize,
    /// The minimum of required update messages.
    pub min_update: usize,
    /// The number of expected participants.
    pub expected_participants: usize,
    /// The masking configuration.
    pub mask_config: MaskConfig,
    /// The event publisher.
    pub events: EventPublisher,
}

impl CoordinatorState {
    pub fn new(pet_settings: PetSettings, mask_settings: MaskSettings) -> (Self, EventSubscriber) {
        let keys = EncryptKeyPair::generate();
        let round_params = RoundParameters {
            pk: keys.public,
            sum: pet_settings.sum,
            update: pet_settings.update,
            seed: RoundSeed::zeroed(),
        };
        let phase = PhaseEvent::Idle;

        let (publisher, subscriber) =
            EventPublisher::init(keys.clone(), round_params.clone(), phase);

        let coordinator_state = Self {
            keys,
            round_params,
            events: publisher,
            min_sum: pet_settings.min_sum,
            min_update: pet_settings.min_update,
            expected_participants: pet_settings.expected_participants,
            mask_config: mask_settings.into(),
        };
        (coordinator_state, subscriber)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// A seed for a round.
pub struct RoundSeed(box_::Seed);

impl ByteObject for RoundSeed {
    /// Creates a round seed from a slice of bytes.
    ///
    /// # Errors
    /// Fails if the length of the input is invalid.
    fn from_slice(bytes: &[u8]) -> Option<Self> {
        box_::Seed::from_slice(bytes).map(Self)
    }

    /// Creates a round seed initialized to zero.
    fn zeroed() -> Self {
        Self(box_::Seed([0_u8; Self::LENGTH]))
    }

    /// Gets the round seed as a slice.
    fn as_slice(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl RoundSeed {
    /// Gets the number of bytes of a round seed.
    pub const LENGTH: usize = box_::SEEDBYTES;

    /// Generates a random round seed.
    pub fn generate() -> Self {
        // Safe unwrap: length of slice is guaranteed by constants
        Self::from_slice_unchecked(randombytes(Self::LENGTH).as_slice())
    }
}

/// A dictionary created during the sum2 phase of the protocol. It counts the model masks
/// represented by their hashes.
pub type MaskDict = HashMap<MaskObject, usize>;
