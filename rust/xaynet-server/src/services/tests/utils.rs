use xaynet_core::{
    common::{RoundParameters, RoundSeed},
    crypto::{ByteObject, EncryptKeyPair, PublicEncryptKey, SigningKeyPair},
    message::{Message, Sum},
};

use crate::state_machine::{
    events::{EventPublisher, EventSubscriber},
    phases::PhaseName,
};

/// Create an [`EventPublisher`]/[`EventSubscriber`] pair with default
/// values similar to those produced in practice when instantiating a
/// new coordinator.
pub fn new_event_channels() -> (EventPublisher, EventSubscriber) {
    let keys = EncryptKeyPair::generate();
    let params = RoundParameters {
        pk: keys.public,
        sum: 0.0,
        update: 0.0,
        seed: RoundSeed::generate(),
    };
    let phase = PhaseName::Idle;
    let round_id = 0;
    EventPublisher::init(round_id, keys, params, phase)
}

/// Simulate a participant generating keys and crafting a valid sum
/// message for the given round parameters. The keys generated by the
/// participants are returned along with the message.
pub fn new_sum_message(round_params: &RoundParameters) -> (Message, SigningKeyPair) {
    let signing_keys = SigningKeyPair::generate();
    let sum = Sum {
        sum_signature: signing_keys
            .secret
            .sign_detached(&[round_params.seed.as_slice(), b"sum"].concat()),
        ephm_pk: PublicEncryptKey::generate(),
    };
    let message = Message::new_sum(signing_keys.public, round_params.pk, sum);
    (message, signing_keys)
}

/// Sign and encrypt the given message using the given round
/// parameters and particpant keys.
pub fn encrypt_message(
    message: &Message,
    round_params: &RoundParameters,
    participant_signing_keys: &SigningKeyPair,
) -> Vec<u8> {
    let serialized = serialize_message(message, participant_signing_keys);
    round_params.pk.encrypt(&serialized[..])
}

pub fn serialize_message(message: &Message, participant_signing_keys: &SigningKeyPair) -> Vec<u8> {
    let mut buf = vec![0; message.buffer_length()];
    message.to_bytes(&mut buf, &participant_signing_keys.secret);
    buf
}
