use crate::{
    client::{
        mobile_client::participant::{
            Participant,
            ParticipantSettings,
            Sum,
            Sum2,
            Type,
            Undefined,
            Update,
        },
        ClientError,
        Proxy,
    },
    crypto::ByteObject,
    mask::model::Model,
    state_machine::coordinator::RoundParameters,
    InitError,
    PetError,
};
use derive_more::From;
use std::{cell::RefCell, rc::Rc};

#[derive(From)]
pub enum ClientStateMachine {
    Undefined(ClientState<Undefined>),
    Sum(ClientState<Sum>),
    Update(ClientState<Update>),
    Sum2(ClientState<Sum2>),
}

impl ClientStateMachine {
    pub fn new(
        proxy: Proxy,
        participant_settings: ParticipantSettings,
        local_model: Rc<RefCell<Option<Model>>>,
        global_model: Rc<RefCell<Option<Model>>>,
    ) -> Result<Self, InitError> {
        // crucial: init must be called before anything else in this module
        sodiumoxide::init().or(Err(InitError))?;

        Ok(ClientState::<Undefined>::new(
            proxy,
            Participant::<Undefined>::new(participant_settings.into()),
            local_model,
            global_model,
        )
        .into())
    }

    pub async fn next(self) -> Self {
        match self {
            ClientStateMachine::Undefined(state) => state.next().await,
            ClientStateMachine::Sum(state) => state.next().await,
            ClientStateMachine::Update(state) => state.next().await,
            ClientStateMachine::Sum2(state) => state.next().await,
        }
    }
}

pub struct ClientState<Type> {
    proxy: Proxy,
    round_params: RoundParameters,
    participant: Participant<Type>,
    local_model: Rc<RefCell<Option<Model>>>,
    global_model: Rc<RefCell<Option<Model>>>,
}

impl<Type> ClientState<Type> {
    async fn check_round_freshness(&self) -> Result<(), ClientError> {
        debug!("fetching round parameters");
        let round_params = self.proxy.get_round_params().await?;
        if round_params.seed != self.round_params.seed {
            info!("new round parameters");
            Err(ClientError::RoundOutdated)
        } else {
            Ok(())
        }
    }

    fn reset(self) -> ClientState<Undefined> {
        warn!("reset client");
        ClientState::<Undefined>::new(
            self.proxy,
            self.participant.reset(),
            self.local_model,
            self.global_model,
        )
    }
}

impl ClientState<Undefined> {
    fn new(
        proxy: Proxy,
        participant: Participant<Undefined>,
        local_model: Rc<RefCell<Option<Model>>>,
        global_model: Rc<RefCell<Option<Model>>>,
    ) -> Self {
        Self {
            proxy,
            round_params: RoundParameters::default(),
            participant,
            local_model,
            global_model,
        }
    }

    async fn next(mut self) -> ClientStateMachine {
        info!("new participant with undefined task");
        if let Err(err) = self.fetch_round_params().await {
            error!("{:?}", err);
            return self.reset().into();
        };

        let Self {
            proxy,
            round_params,
            participant,
            local_model,
            global_model,
        } = self;

        let participant_type = participant.determine_type(
            round_params.seed.as_slice(),
            round_params.sum,
            round_params.update,
        );

        match participant_type {
            Type::Unselected(unsel_par) => {
                info!("unselected");
                ClientState::<Undefined>::new(proxy, unsel_par.reset(), local_model, global_model)
                    .into()
            }
            Type::Summer(sum_par) => {
                ClientState::<Sum>::new(proxy, round_params, sum_par, local_model, global_model)
                    .into()
            }
            Type::Updater(upt_pat) => {
                ClientState::<Update>::new(proxy, round_params, upt_pat, local_model, global_model)
                    .into()
            }
        }
    }

    async fn fetch_round_params(&mut self) -> Result<(), ClientError> {
        self.round_params = self.proxy.get_round_params().await?;
        self.fetch_global_model().await;
        Ok(())
    }

    async fn fetch_global_model(&mut self) {
        if let Ok(model) = self.proxy.get_model().await {
            //update our global model where necessary
            let mut global_model = self.global_model.borrow_mut();

            match (model, global_model.as_ref()) {
                (Some(new_model), None) => {
                    info!("new global model");
                    *global_model = Some(new_model);
                }
                (Some(new_model), Some(old_model)) if &new_model != old_model => {
                    debug!("updating global model");
                    *global_model = Some(new_model);
                }
                (None, _) => debug!("global model not ready yet"),
                _ => debug!("global model still fresh"),
            }
        }
    }
}

impl ClientState<Sum> {
    fn new(
        proxy: Proxy,
        round_params: RoundParameters,
        participant: Participant<Sum>,
        local_model: Rc<RefCell<Option<Model>>>,
        global_model: Rc<RefCell<Option<Model>>>,
    ) -> Self {
        Self {
            proxy,
            round_params,
            participant,
            local_model,
            global_model,
        }
    }

    async fn next(mut self) -> ClientStateMachine {
        info!("selected to sum");

        match self.run().await {
            Ok(_) => self.move_into_sum2().into(),
            Err(ClientError::RoundOutdated) => self.reset().into(),
            Err(err) => {
                error!("{:?}", err);
                self.into()
            }
        }
    }

    async fn run(&mut self) -> Result<(), ClientError> {
        self.check_round_freshness().await?;

        let sum_msg = self.participant.compose_sum_message(&self.round_params.pk);
        let sealed_msg = self
            .participant
            .seal_message(&self.round_params.pk, &sum_msg);

        debug!("sending sum message");
        self.proxy.post_message(sealed_msg).await?;
        debug!("sum message sent");
        Ok(())
    }

    fn move_into_sum2(self) -> ClientState<Sum2> {
        ClientState::<Sum2>::new(
            self.proxy,
            self.round_params,
            self.participant.next(),
            self.local_model,
            self.global_model,
        )
    }
}

impl ClientState<Update> {
    fn new(
        proxy: Proxy,
        round_params: RoundParameters,
        participant: Participant<Update>,
        local_model: Rc<RefCell<Option<Model>>>,
        global_model: Rc<RefCell<Option<Model>>>,
    ) -> Self {
        Self {
            proxy,
            round_params,
            participant,
            local_model,
            global_model,
        }
    }

    async fn next(mut self) -> ClientStateMachine {
        info!("selected to update");

        match self.run().await {
            Ok(_) | Err(ClientError::RoundOutdated) => self.reset().into(),
            Err(err) => {
                error!("{:?}", err);
                self.into()
            }
        }
    }

    async fn run(&mut self) -> Result<(), ClientError> {
        self.check_round_freshness().await?;

        debug!("polling for local model");
        let local_model = self
            .local_model
            .borrow_mut()
            .take()
            .ok_or(ClientError::TooEarly("local model"))?
            .clone();

        debug!("polling for model scalar");
        let scalar = self
            .proxy
            .get_scalar()
            .await?
            .ok_or(ClientError::TooEarly("scalar"))?;

        debug!("polling for sum dict");
        let sums = self
            .proxy
            .get_sums()
            .await?
            .ok_or(ClientError::TooEarly("sum dict"))?;

        let upd_msg = self.participant.compose_update_message(
            self.round_params.pk,
            &sums,
            scalar,
            local_model,
        );
        let sealed_msg = self
            .participant
            .seal_message(&self.round_params.pk, &upd_msg);

        debug!("sending update message");
        self.proxy.post_message(sealed_msg).await?;
        info!("update participant completed a round");
        Ok(())
    }
}

impl ClientState<Sum2> {
    fn new(
        proxy: Proxy,
        round_params: RoundParameters,
        participant: Participant<Sum2>,
        local_model: Rc<RefCell<Option<Model>>>,
        global_model: Rc<RefCell<Option<Model>>>,
    ) -> Self {
        Self {
            proxy,
            round_params,
            participant,
            local_model,
            global_model,
        }
    }

    async fn next(mut self) -> ClientStateMachine {
        info!("selected to sum2");

        match self.run().await {
            Ok(_) | Err(ClientError::RoundOutdated) => self.reset().into(),
            Err(err) => {
                error!("{:?}", err);
                self.into()
            }
        }
    }

    async fn run(&mut self) -> Result<(), ClientError> {
        self.check_round_freshness().await?;

        debug!("polling for model/mask length");
        let length = self
            .proxy
            .get_mask_length()
            .await?
            .ok_or(ClientError::TooEarly("length"))?;
        if length > usize::MAX as u64 {
            return Err(ClientError::ParticipantErr(PetError::InvalidModel));
        };

        debug!("polling for seed dict");
        let seeds = self
            .proxy
            .get_seeds(self.participant.get_participant_pk())
            .await?
            .ok_or(ClientError::TooEarly("seeds"))?;

        let sum2_msg = self
            .participant
            .compose_sum2_message(self.round_params.pk, &seeds, length as usize)
            .map_err(|e| {
                error!("failed to compose sum2 message with seeds: {:?}", &seeds);
                ClientError::ParticipantErr(e)
            })?;
        let sealed_msg = self
            .participant
            .seal_message(&self.round_params.pk, &sum2_msg);

        debug!("sending sum2 message");
        self.proxy.post_message(sealed_msg).await?;
        info!("sum participant completed a round");
        Ok(())
    }
}
