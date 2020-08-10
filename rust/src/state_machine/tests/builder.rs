use crate::{
    crypto::encrypt::EncryptKeyPair,
    mask::config::MaskConfig,
    state_machine::{
        coordinator::RoundSeed,
        events::EventSubscriber,
        phases::{self, Handler, Phase, PhaseState, Shared},
        requests::RequestSender,
        tests::utils,
        StateMachine,
    },
};

#[derive(Debug)]
pub struct StateMachineBuilder<P> {
    shared: Shared,
    request_tx: RequestSender,
    event_subscriber: EventSubscriber,
    phase_state: P,
}

impl StateMachineBuilder<phases::Idle> {
    pub fn new() -> Self {
        let (shared, event_subscriber, request_tx) = utils::init_shared();

        let phase_state = phases::Idle;
        StateMachineBuilder {
            shared,
            request_tx,
            event_subscriber,
            phase_state,
        }
    }
}

impl<P> StateMachineBuilder<P>
where
    PhaseState<P>: Handler + Phase,
    StateMachine: From<PhaseState<P>>,
{
    pub fn build(self) -> (StateMachine, RequestSender, EventSubscriber) {
        let Self {
            mut shared,
            request_tx,
            event_subscriber,
            phase_state,
        } = self;

        // Make sure the events that the listeners have are up to date
        let events = &mut shared.io.events;
        events.broadcast_keys(shared.state.keys.clone());
        events.broadcast_params(shared.state.round_params.clone());
        events.broadcast_phase(<PhaseState<P> as Phase>::NAME);
        // Also re-emit the other events in case the round ID changed
        let scalar = event_subscriber.scalar_listener().get_latest().event;
        events.broadcast_scalar(scalar);
        let model = event_subscriber.model_listener().get_latest().event;
        events.broadcast_model(model);
        let mask_length = event_subscriber.mask_length_listener().get_latest().event;
        events.broadcast_mask_length(mask_length);
        let sum_dict = event_subscriber.sum_dict_listener().get_latest().event;
        events.broadcast_sum_dict(sum_dict);
        let seed_dict = event_subscriber.seed_dict_listener().get_latest().event;
        events.broadcast_seed_dict(seed_dict);

        let state = PhaseState {
            inner: phase_state,
            shared,
        };

        let state_machine = StateMachine::from(state);

        (state_machine, request_tx, event_subscriber)
    }

    #[allow(dead_code)]
    pub fn with_keys(mut self, keys: EncryptKeyPair) -> Self {
        self.shared.state.round_params.pk = keys.public.clone();
        self.shared.state.keys = keys.clone();
        self
    }

    pub fn with_round_id(mut self, id: u64) -> Self {
        self.shared.set_round_id(id);
        self
    }

    pub fn with_sum_ratio(mut self, sum_ratio: f64) -> Self {
        self.shared.state.round_params.sum = sum_ratio;
        self
    }

    pub fn with_update_ratio(mut self, update_ratio: f64) -> Self {
        self.shared.state.round_params.update = update_ratio;
        self
    }

    pub fn with_expected_participants(mut self, expected_participants: usize) -> Self {
        self.shared.state.expected_participants = expected_participants;
        self
    }

    pub fn with_seed(mut self, seed: RoundSeed) -> Self {
        self.shared.state.round_params.seed = seed;
        self
    }

    pub fn with_min_sum(mut self, min_sum: usize) -> Self {
        self.shared.state.min_sum_count = min_sum;
        self
    }

    pub fn with_mask_config(mut self, mask_config: MaskConfig) -> Self {
        self.shared.state.mask_config = mask_config;
        self
    }

    pub fn with_min_update(mut self, min_update: usize) -> Self {
        self.shared.state.min_update_count = min_update;
        self
    }

    pub fn with_model_size(mut self, model_size: usize) -> Self {
        self.shared.state.model_size = model_size;
        self
    }

    pub fn with_phase<S>(self, phase_state: S) -> StateMachineBuilder<S> {
        let Self {
            shared,
            request_tx,
            event_subscriber,
            ..
        } = self;
        StateMachineBuilder {
            shared,
            request_tx,
            event_subscriber,
            phase_state,
        }
    }
}
