//! Coordinator state and round parameter types.

use serde::{Deserialize, Serialize};

use crate::settings::{MaskSettings, ModelSettings, PetSettings};
use xaynet_core::{
    common::{RoundParameters, RoundSeed},
    crypto::{ByteObject, EncryptKeyPair},
    mask::MaskConfig,
};

/// The coordinator state.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct CoordinatorState {
    /// The credentials of the coordinator.
    pub keys: EncryptKeyPair,
    /// Internal ID used to identify a round
    pub round_id: u64,
    /// The round parameters.
    pub round_params: RoundParameters,
    /// The minimum of required sum/sum2 messages.
    pub min_sum_count: u64,
    /// The minimum of required update messages.
    pub min_update_count: u64,
    /// The maximum of accepted sum/sum2 messages.
    pub max_sum_count: u64,
    /// The maximum of accepted update messages.
    pub max_update_count: u64,
    /// The minimum time (in seconds) reserved for processing sum/sum2 messages.
    pub min_sum_time: u64,
    /// The minimum time (in seconds) reserved for processing update messages.
    pub min_update_time: u64,
    /// The maximum time (in seconds) permitted for processing sum/sum2 messages.
    pub max_sum_time: u64,
    /// The maximum time (in seconds) permitted for processing update messages.
    pub max_update_time: u64,
}

impl CoordinatorState {
    pub fn new(
        pet_settings: PetSettings,
        mask_settings: MaskSettings,
        model_settings: ModelSettings,
    ) -> Self {
        let keys = EncryptKeyPair::generate();
        let mask_config: MaskConfig = mask_settings.into();
        let round_params = RoundParameters {
            pk: keys.public,
            sum: pet_settings.sum,
            update: pet_settings.update,
            seed: RoundSeed::zeroed(),
            mask_config: mask_config.clone().into(),
            model_length: model_settings.length,
        };
        let round_id = 0;
        Self {
            keys,
            round_params,
            round_id,
            min_sum_count: pet_settings.min_sum_count,
            min_update_count: pet_settings.min_update_count,
            max_sum_count: pet_settings.max_sum_count,
            max_update_count: pet_settings.max_update_count,
            min_sum_time: pet_settings.min_sum_time,
            min_update_time: pet_settings.min_update_time,
            max_sum_time: pet_settings.max_sum_time,
            max_update_time: pet_settings.max_update_time,
        }
    }
}
