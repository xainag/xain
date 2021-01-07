use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

use crate::{
    state_machine::{IntoPhase, Phase, PhaseIo, Progress, State, Step, Sum2, TransitionOutcome},
    MessageEncoder,
};
use xaynet_core::{
    crypto::{EncryptKeyPair, Signature},
    message::Sum as SumMessage,
};

#[derive(Serialize, Deserialize, Debug)]
pub struct Sum {
    pub ephm_keys: EncryptKeyPair,
    pub sum_signature: Signature,
    pub message: Option<MessageEncoder>,
}

impl Sum {
    pub fn new(sum_signature: Signature) -> Self {
        Sum {
            ephm_keys: EncryptKeyPair::generate(),
            sum_signature,
            message: None,
        }
    }
}

impl IntoPhase<Sum> for State<Sum> {
    fn into_phase(self, mut io: PhaseIo) -> Phase<Sum> {
        io.notify_sum();
        Phase::<_>::new(self, io)
    }
}

#[async_trait]
impl Step for Phase<Sum> {
    async fn step(mut self) -> TransitionOutcome {
        info!("sum task");

        self = try_progress!(self.compose_sum_message());

        // FIXME: currently if sending fails, we lose the message,
        // thus wasting all the work we've done in this phase
        let message = self.state.private.message.take().unwrap();
        match self.send_message(message).await {
            Ok(_) => {
                info!("sent sum message, going to sum2 phase");
                TransitionOutcome::Complete(self.into_sum2().into())
            }
            Err(e) => {
                warn!("failed to send sum message: {}", e);
                warn!("sum phase failed, going back to awaiting phase");
                TransitionOutcome::Complete(self.into_awaiting().into())
            }
        }
    }
}

impl Phase<Sum> {
    pub fn compose_sum_message(mut self) -> Progress<Sum> {
        if self.state.private.message.is_some() {
            return Progress::Continue(self);
        }

        let sum = SumMessage {
            sum_signature: self.state.private.sum_signature,
            ephm_pk: self.state.private.ephm_keys.public,
        };
        self.state.private.message = Some(self.message_encoder(sum.into()));
        Progress::Updated(self.into())
    }

    pub fn into_sum2(self) -> Phase<Sum2> {
        let sum2 = Box::new(Sum2::new(
            self.state.private.ephm_keys,
            self.state.private.sum_signature,
        ));
        let state = State::new(self.state.shared, sum2);
        state.into_phase(self.io)
    }
}
