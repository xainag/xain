use super::{Participant, ParticipantState};
use xaynet_core::{
    mask::{Aggregation, MaskObject, MaskSeed},
    message::{Payload, Sum2 as Sum2Message},
    ParticipantPublicKey,
    ParticipantTaskSignature,
    SumParticipantEphemeralPublicKey,
    SumParticipantEphemeralSecretKey,
    UpdateSeedDict,
};

use crate::PetError;

#[derive(Serialize, Deserialize, Clone)]
pub struct Sum2 {
    ephm_pk: SumParticipantEphemeralPublicKey,
    ephm_sk: SumParticipantEphemeralSecretKey,
    sum_signature: ParticipantTaskSignature,
}

impl Participant<Sum2> {
    pub fn new(
        state: ParticipantState,
        sum_signature: ParticipantTaskSignature,
        ephm_pk: SumParticipantEphemeralPublicKey,
        ephm_sk: SumParticipantEphemeralSecretKey,
    ) -> Self {
        Self {
            inner: Sum2 {
                sum_signature,
                ephm_pk,
                ephm_sk,
            },
            state,
        }
    }

    /// Compose a sum2 message given the coordinator public key, seed dictionary
    /// and mask length.
    ///
    /// # Errors
    ///
    /// Returns a [`PetError`] if there is a problem extracting the
    /// seed dictionary, or computing the global mask.
    pub fn compose_sum2_message(
        &self,
        seed_dict: &UpdateSeedDict,
        mask_len: usize,
    ) -> Result<Payload, PetError> {
        let mask_seeds = self.get_seeds(seed_dict)?;
        let mask = self.compute_global_mask(mask_seeds, mask_len)?;
        let sum2 = Sum2Message {
            sum_signature: self.inner.sum_signature,
            model_mask: mask,
        };
        Ok(sum2.into())
    }

    pub fn get_participant_pk(&self) -> ParticipantPublicKey {
        self.state.keys.public
    }

    /// Get the mask seeds from the local seed dictionary.
    fn get_seeds(&self, seed_dict: &UpdateSeedDict) -> Result<Vec<MaskSeed>, PetError> {
        seed_dict
            .values()
            .map(|seed| {
                seed.decrypt(&self.inner.ephm_pk, &self.inner.ephm_sk)
                    .map_err(|_| PetError::InvalidMask)
            })
            .collect()
    }

    /// Compute a global mask from local mask seeds.
    fn compute_global_mask(
        &self,
        mask_seeds: Vec<MaskSeed>,
        mask_len: usize,
    ) -> Result<MaskObject, PetError> {
        if mask_seeds.is_empty() {
            return Err(PetError::InvalidMask);
        }

        // HACK reuse config for both
        let config = self.state.aggregation_config.mask;
        let mut mask_agg = Aggregation::new(config, config, mask_len);
        for seed in mask_seeds.into_iter() {
            let mask = seed.derive_mask(mask_len, config, config);
            mask_agg
                .validate_aggregation(&mask)
                .map_err(|_| PetError::InvalidMask)?;

            mask_agg.aggregate(mask);
        }
        Ok(mask_agg.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mobile_client::participant::AggregationConfig;
    use sodiumoxide::randombytes::{randombytes, randombytes_uniform};
    use std::{collections::HashSet, iter};
    use xaynet_core::{
        crypto::{ByteObject, EncryptKeyPair, Signature, SigningKeyPair},
        mask::{BoundType, DataType, GroupType, MaskConfig, ModelType},
        UpdateParticipantPublicKey,
    };

    fn participant_state() -> ParticipantState {
        sodiumoxide::init().unwrap();

        let aggregation_config = AggregationConfig {
            mask: MaskConfig {
                group_type: GroupType::Prime,
                data_type: DataType::F32,
                bound_type: BoundType::B0,
                model_type: ModelType::M3,
            },

            scalar: 1_f64,
        };
        ParticipantState {
            keys: SigningKeyPair::generate(),
            aggregation_config,
        }
    }

    #[test]
    fn test_get_seeds() {
        let EncryptKeyPair { public, secret } = EncryptKeyPair::generate();
        let part =
            Participant::<Sum2>::new(participant_state(), Signature::zeroed(), public, secret);
        let mask_seeds: Vec<MaskSeed> = iter::repeat_with(MaskSeed::generate)
            .take(1 + randombytes_uniform(10) as usize)
            .collect::<Vec<_>>();
        let upd_seed_dict = mask_seeds
            .iter()
            .map(|seed| {
                (
                    UpdateParticipantPublicKey::from_slice(&randombytes(32)).unwrap(),
                    seed.encrypt(&part.inner.ephm_pk),
                )
            })
            .collect();
        assert_eq!(
            part.get_seeds(&upd_seed_dict)
                .unwrap()
                .into_iter()
                .map(|seed| seed.as_array())
                .collect::<HashSet<_>>(),
            mask_seeds
                .into_iter()
                .map(|seed| seed.as_array())
                .collect::<HashSet<_>>(),
        );
    }
}
