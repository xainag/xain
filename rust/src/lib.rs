#![allow(dead_code)]
#![allow(unused_imports)]
#![feature(or_patterns)]
#![feature(const_fn)]
#![feature(stmt_expr_attributes)]

#[macro_use]
extern crate tracing;

#[macro_use]
extern crate serde;

pub mod certificate;
pub mod coordinator;
pub mod crypto;
pub mod mask;
pub mod message;
pub mod participant;
pub mod service;
pub mod utils;

use std::collections::HashMap;

use crypto::{PublicEncryptKey, PublicSigningKey, SecretEncryptKey, SecretSigningKey, Signature};
use thiserror::Error;

use crate::mask::seed::EncryptedMaskSeed;

#[derive(Error, Debug)]
#[error("initialization failed: insufficient system entropy to generate secrets")]
pub struct InitError;

#[derive(Debug, PartialEq)]
/// PET protocol errors.
pub enum PetError {
    InvalidMessage,
}

/// A public encryption key that identifies a coordinator.
pub type CoordinatorPublicKey = PublicEncryptKey;

/// A secret encryption key that belongs to the public key of a
/// coordinator.
pub type CoordinatorSecretKey = SecretEncryptKey;

/// A public signature key that identifies a participant.
pub type ParticipantPublicKey = PublicSigningKey;

/// A secret signature key that belongs to the public key of a
/// participant.
pub type ParticipantSecretKey = SecretSigningKey;

/// A public signature key that identifies a sum participant.
pub type SumParticipantPublicKey = ParticipantPublicKey;

/// A secret signature key that belongs to the public key of a sum
/// participant.
pub type SumParticipantSecretKey = ParticipantSecretKey;

/// A public encryption key generated by a sum participant. It is used
/// by the update participants to encrypt their masking seed for each
/// sum participant.
pub type SumParticipantEphemeralPublicKey = PublicEncryptKey;

/// The secret counterpart of [`SumParticipantEphemeralPublicKey`]
pub type SumParticipantEphemeralSecretKey = SecretEncryptKey;

/// A public signature key that identifies an update participant.
pub type UpdateParticipantPublicKey = ParticipantPublicKey;

/// A secret signature key that belongs to the public key of an update
/// participant.
pub type UpdateParticipantSecretKey = ParticipantSecretKey;

/// A signature to prove a participant's eligibility for a task.
pub type ParticipantTaskSignature = Signature;

/// A dictionary created during the sum phase of the protocol. It maps the public key of every sum
/// participant to the ephemeral public key generated by that sum participant.
type SumDict = HashMap<SumParticipantPublicKey, SumParticipantEphemeralPublicKey>;

/// Local seed dictionaries are sent by update participants. They contain the participant's masking
/// seed, encrypted with the ephemeral public key of each sum participant.
type LocalSeedDict = HashMap<SumParticipantPublicKey, EncryptedMaskSeed>;

/// A dictionary created during the update phase of the protocol. The global seed dictionary is
/// built from the local seed dictionaries sent by the update participants. It maps each sum
/// participant to the encrypted masking seeds of all the update participants.
type SeedDict =
    HashMap<SumParticipantPublicKey, HashMap<UpdateParticipantPublicKey, EncryptedMaskSeed>>;

/// A 32-byte hash that identifies a model mask computed by a sum participant.
pub type MaskHash = sodiumoxide::crypto::hash::sha256::Digest;
