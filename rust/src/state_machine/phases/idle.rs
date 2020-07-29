use crate::{
    crypto::{encrypt::EncryptKeyPair, sign::SigningKeySeed, ByteObject},
    state_machine::{
        coordinator::{CoordinatorState, RoundSeed},
        events::{DictionaryUpdate, MaskLengthUpdate, ScalarUpdate},
        phases::{reject_request, Handler, Phase, PhaseName, PhaseState, Sum},
        requests::{Request, RequestReceiver},
        StateError,
        StateMachine,
    },
};

use sodiumoxide::crypto::hash::sha256;

/// Idle state
#[derive(Debug)]
pub struct Idle;

impl<R> Handler<Request> for PhaseState<R, Idle> {
    /// Reject all the request with a [`PetError::InvalidMessage`]
    fn handle_request(&mut self, req: Request) {
        reject_request(req);
    }
}

#[async_trait]
impl<R> Phase<R> for PhaseState<R, Idle>
where
    R: Send,
{
    const NAME: PhaseName = PhaseName::Idle;

    /// Moves from the idle state to the next state.
    ///
    /// See the [module level documentation](../index.html) for more details.
    async fn run(&mut self) -> Result<(), StateError> {
        info!("updating the keys");
        self.gen_round_keypair();

        info!("updating round thresholds");
        self.update_round_thresholds();

        info!("updating round seeds");
        self.update_round_seed();

        let events = &mut self.coordinator_state.events;

        info!("broadcasting new keys");
        events.broadcast_keys(self.coordinator_state.keys.clone());

        info!("broadcasting invalidation of sum dictionary from previous round");
        events.broadcast_sum_dict(DictionaryUpdate::Invalidate);

        info!("broadcasting invalidation of seed dictionary from previous round");
        events.broadcast_seed_dict(DictionaryUpdate::Invalidate);

        info!("broadcasting invalidation of scalar from previous round");
        events.broadcast_scalar(ScalarUpdate::Invalidate);

        info!("broadcasting invalidation of mask length from previous round");
        events.broadcast_mask_length(MaskLengthUpdate::Invalidate);

        info!("broadcasting new round parameters");
        events.broadcast_params(self.coordinator_state.round_params.clone());

        // TODO: add a delay to prolongate the idle phase
        Ok(())
    }

    fn next(self) -> Option<StateMachine<R>> {
        info!("going to sum phase");
        Some(PhaseState::<R, Sum>::new(self.coordinator_state, self.request_rx).into())
    }
}

impl<R> PhaseState<R, Idle> {
    /// Creates a new idle state.
    pub fn new(mut coordinator_state: CoordinatorState, request_rx: RequestReceiver<R>) -> Self {
        // Since some events are emitted very early, the round id must
        // be correct when the idle phase starts. Therefore, we update
        // it here, when instantiating the idle PhaseState.
        coordinator_state.set_round_id(coordinator_state.round_id() + 1);
        debug!("new round ID = {}", coordinator_state.round_id());
        Self {
            inner: Idle,
            coordinator_state,
            request_rx,
        }
    }

    fn update_round_thresholds(&mut self) {}

    /// Updates the seed round parameter.
    fn update_round_seed(&mut self) {
        // Safe unwrap: `sk` and `seed` have same number of bytes
        let (_, sk) =
            SigningKeySeed::from_slice_unchecked(self.coordinator_state.keys.secret.as_slice())
                .derive_signing_key_pair();
        let signature = sk.sign_detached(
            &[
                self.coordinator_state.round_params.seed.as_slice(),
                &self.coordinator_state.round_params.sum.to_le_bytes(),
                &self.coordinator_state.round_params.update.to_le_bytes(),
            ]
            .concat(),
        );
        // Safe unwrap: the length of the hash is 32 bytes
        self.coordinator_state.round_params.seed =
            RoundSeed::from_slice_unchecked(sha256::hash(signature.as_slice()).as_ref());
    }

    /// Generates fresh round credentials.
    fn gen_round_keypair(&mut self) {
        self.coordinator_state.keys = EncryptKeyPair::generate();
        self.coordinator_state.round_params.pk = self.coordinator_state.keys.public;
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::state_machine::{
        events::Event,
        tests::{builder::StateMachineBuilder, utils},
    };

    #[tokio::test]
    async fn round_id_is_updated_when_idle_phase_runs() {
        let (coordinator_state, event_subscriber) = CoordinatorState::new(
            utils::pet_settings(),
            utils::mask_settings(),
            utils::model_settings(),
        );
        let keys = event_subscriber.keys_listener();
        let id = keys.get_latest().round_id;
        assert_eq!(id, 0);

        let (request_rx, _request_tx) = RequestReceiver::<Request>::new();
        let mut idle_phase = PhaseState::<Request, Idle>::new(coordinator_state, request_rx);
        idle_phase.run().await.unwrap();

        let id = keys.get_latest().round_id;
        assert_eq!(id, 1);
    }

    #[tokio::test]
    async fn idle_to_sum() {
        let (state_machine, _request_tx, events) =
            StateMachineBuilder::new().with_round_id(2).build();
        assert!(state_machine.is_idle());

        let initial_round_params = events.params_listener().get_latest().event;
        let initial_keys = events.keys_listener().get_latest().event;
        let initial_seed = initial_round_params.seed.clone();

        let state_machine = state_machine.next().await.unwrap();
        assert!(state_machine.is_sum());

        let PhaseState {
            inner: sum_state,
            coordinator_state,
            ..
        } = state_machine.into_sum_phase_state();

        assert!(sum_state.sum_dict().is_empty());

        let new_round_params = coordinator_state.round_params.clone();
        let new_keys = coordinator_state.keys.clone();

        // Make sure the seed and keys have updated
        assert_ne!(initial_seed, new_round_params.seed.clone());
        assert_ne!(initial_keys, new_keys);

        fn expected_event<T>(event: T) -> Event<T> {
            Event { round_id: 2, event }
        }

        // Check all the events that should be emitted during the idle
        // phase
        assert_eq!(
            events.phase_listener().get_latest(),
            expected_event(PhaseName::Idle)
        );

        assert_eq!(
            events.keys_listener().get_latest(),
            expected_event(new_keys),
        );

        assert_eq!(
            events.params_listener().get_latest(),
            expected_event(new_round_params)
        );

        assert_eq!(
            events.sum_dict_listener().get_latest(),
            expected_event(DictionaryUpdate::Invalidate)
        );

        assert_eq!(
            events.seed_dict_listener().get_latest(),
            expected_event(DictionaryUpdate::Invalidate)
        );

        assert_eq!(
            events.scalar_listener().get_latest(),
            expected_event(ScalarUpdate::Invalidate)
        );

        assert_eq!(
            events.mask_length_listener().get_latest(),
            expected_event(MaskLengthUpdate::Invalidate)
        );
    }
}
