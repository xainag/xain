use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info, warn};
use xaynet_core::{
    crypto::{EncryptKeyPair, Signature},
    mask::{Aggregation, MaskObject, MaskSeed},
    message::Sum2 as Sum2Message,
    UpdateSeedDict,
};

use crate::{
    state_machine::{
        IntoPhase,
        Phase,
        PhaseIo,
        Progress,
        SendingSum2,
        State,
        Step,
        TransitionOutcome,
        IO,
    },
    MessageEncoder,
};

use super::Awaiting;

/// The state of the sum2 phase.
#[derive(Serialize, Deserialize, Debug)]
pub struct Sum2 {
    /// The sum participant ephemeral keys. They are used to decrypt
    /// the encrypted mask seeds.
    pub ephm_keys: EncryptKeyPair,
    /// Signature that proves that the participant has been selected
    /// for the sum task.
    pub sum_signature: Signature,
    /// Dictionary containing the encrypted mask seed of every update
    /// participants.
    pub seed_dict: Option<UpdateSeedDict>,
    /// The decrypted mask seeds
    pub seeds: Option<Vec<MaskSeed>>,
    /// The global mask, obtained by aggregating the masks derived
    /// from the mask seeds.
    pub mask: Option<MaskObject>,
}

impl Sum2 {
    /// Creates a new sum2 state.
    pub fn new(ephm_keys: EncryptKeyPair, sum_signature: Signature) -> Self {
        Self {
            ephm_keys,
            sum_signature,
            seed_dict: None,
            seeds: None,
            mask: None,
        }
    }

    /// Checks if the seed dict has already been fetched.
    fn has_fetched_seed_dict(&self) -> bool {
        self.seed_dict.is_some() || self.has_decrypted_seeds()
    }

    /// Checks if the seeds have already been decrypted.
    fn has_decrypted_seeds(&self) -> bool {
        self.seeds.is_some() || self.has_aggregated_masks()
    }

    /// Checks if the masks have already been aggregated.
    fn has_aggregated_masks(&self) -> bool {
        self.mask.is_some()
    }
}

impl IntoPhase<Sum2> for State<Sum2> {
    fn into_phase(self, io: PhaseIo) -> Phase<Sum2> {
        Phase::<_>::new(self, io)
    }
}

#[async_trait]
impl Step for Phase<Sum2> {
    async fn step(mut self) -> TransitionOutcome {
        info!("sum2 task");
        self = try_progress!(self.fetch_seed_dict().await);
        self = try_progress!(self.decrypt_seeds());
        self = try_progress!(self.aggregate_masks());
        let sending: Phase<SendingSum2> = self.into();
        TransitionOutcome::Complete(sending.into())
    }
}

impl From<Phase<Sum2>> for Phase<SendingSum2> {
    fn from(mut sum2: Phase<Sum2>) -> Self {
        debug!("composing sum2 message");
        let message = sum2.compose_message();

        debug!("going to sending phase");
        let sending = Box::new(SendingSum2::new(message, Awaiting));
        let state = State::new(sum2.state.shared, sending);
        state.into_phase(sum2.io)
    }
}

impl From<Phase<Sum2>> for Phase<Awaiting> {
    fn from(sum2: Phase<Sum2>) -> Self {
        State::new(sum2.state.shared, Box::new(Awaiting)).into_phase(sum2.io)
    }
}

impl Phase<Sum2> {
    /// Retrieve the encrypted mask seeds.
    pub(crate) async fn fetch_seed_dict(mut self) -> Progress<Sum2> {
        if self.state.private.has_fetched_seed_dict() {
            return Progress::Continue(self);
        }
        debug!("polling for update seeds");
        match self.io.get_seeds(self.state.shared.keys.public).await {
            Err(e) => {
                warn!("failed to fetch seeds: {}", e);
                Progress::Stuck(self)
            }
            Ok(None) => {
                debug!("seeds not available yet");
                Progress::Stuck(self)
            }
            Ok(Some(seeds)) => {
                self.state.private.seed_dict = Some(seeds);
                Progress::Updated(self.into())
            }
        }
    }

    /// Decrypt the mask seeds that the update participants generated.
    pub(crate) fn decrypt_seeds(mut self) -> Progress<Sum2> {
        if self.state.private.has_decrypted_seeds() {
            return Progress::Continue(self);
        }

        let keys = &self.state.private.ephm_keys;
        // UNWRAP_SAFE: the seed dict is set in
        // `self.fetch_seed_dict()` which is called before this method
        let seeds: Result<Vec<MaskSeed>, ()> = self
            .state
            .private
            .seed_dict
            .take()
            .unwrap()
            .into_iter()
            .map(|(_, seed)| seed.decrypt(&keys.public, &keys.secret).map_err(|_| ()))
            .collect();

        match seeds {
            Ok(seeds) => {
                self.state.private.seeds = Some(seeds);
                Progress::Updated(self.into())
            }
            Err(_) => {
                warn!("failed to decrypt mask seeds, going back to waiting phase");
                self.io.notify_idle();
                let awaiting: Phase<Awaiting> = self.into();
                Progress::Updated(awaiting.into())
            }
        }
    }

    /// Derive the masks from the decrypted mask seeds, and aggregate
    /// them. The resulting mask will later be added to the sum2
    /// message to be sent to the coordinator.
    pub(crate) fn aggregate_masks(mut self) -> Progress<Sum2> {
        if self.state.private.has_aggregated_masks() {
            return Progress::Continue(self);
        }

        info!("aggregating masks");
        let config = self.state.shared.round_params.mask_config;
        let mask_len = self.state.shared.round_params.model_length;
        let mut mask_agg = Aggregation::new(config, mask_len as usize);
        // UNWRAP_SAFE: the seeds are set in `decrypt_seeds()` which is called before this method
        for seed in self.state.private.seeds.take().unwrap().into_iter() {
            let mask = seed.derive_mask(mask_len as usize, config);
            if let Err(e) = mask_agg.validate_aggregation(&mask) {
                error!("sum2 phase failed: cannot aggregate masks: {}", e);
                error!("going to awaiting phase");
                let awaiting: Phase<Awaiting> = self.into();
                return Progress::Updated(awaiting.into());
            } else {
                mask_agg.aggregate(mask);
            }
        }
        self.state.private.mask = Some(mask_agg.into());
        Progress::Updated(self.into())
    }

    /// Creates and encodes the sum2 message from the sum2 state.
    pub fn compose_message(&mut self) -> MessageEncoder {
        let sum2 = Sum2Message {
            sum_signature: self.state.private.sum_signature,
            // UNWRAP_SAFE: the mask set in `aggregate_masks()` which is called before this method
            model_mask: self.state.private.mask.take().unwrap(),
        };
        self.message_encoder(sum2.into())
    }
}
