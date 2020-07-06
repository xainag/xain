use std::task::Poll;

use futures::{
    future::{ready, Ready},
    task::Context,
};
use tower::Service;

use crate::{
    crypto::ByteObject,
    message::{
        header::HeaderOwned,
        message::MessageOwned,
        payload::{update::UpdateOwned, PayloadOwned},
    },
    services::messages::pre_processor::{PreProcessorError, PreProcessorResponse},
    state_machine::coordinator::RoundParameters,
};

/// Request type for [`UpdatePreProcessorService`]
pub type UpdateRequest = (HeaderOwned, UpdateOwned, RoundParameters);

/// A service for performing sanity checks and preparing an update
/// request to be handled by the state machine. At the moment, this is
/// limited to verifying the participant's eligibility for the update
/// task.
#[derive(Clone, Debug)]
pub struct UpdatePreProcessorService;

impl Service<UpdateRequest> for UpdatePreProcessorService {
    type Response = PreProcessorResponse;
    type Error = std::convert::Infallible;
    type Future = Ready<Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, (header, message, params): UpdateRequest) -> Self::Future {
        let pre_processor = UpdatePreProcessor {
            header,
            message,
            params,
        };
        ready(Ok(pre_processor.call()))
    }
}

struct UpdatePreProcessor {
    header: HeaderOwned,
    message: UpdateOwned,
    params: RoundParameters,
}

impl UpdatePreProcessor {
    fn call(self) -> Result<MessageOwned, PreProcessorError> {
        debug!("checking sum signature");
        if !self.has_valid_sum_signature() {
            debug!("invalid sum signature");
            return Err(PreProcessorError::InvalidSumSignature);
        }

        debug!("checking sum task eligibility");
        if self.is_eligible_for_sum_task() {
            debug!("participant is eligible for the sum task, so is not eligible for update task");
            return Err(PreProcessorError::NotUpdateEligible);
        }

        debug!("checking update signature");
        if !self.has_valid_update_signature() {
            debug!("invalid update signature");
            return Err(PreProcessorError::InvalidUpdateSignature);
        }

        debug!("checking update task eligibility");
        if !self.is_eligible_for_update_task() {
            debug!("not eligible for update task");
            return Err(PreProcessorError::NotUpdateEligible);
        }

        let Self {
            header, message, ..
        } = self;
        Ok(MessageOwned {
            header,
            payload: PayloadOwned::Update(message),
        })
    }

    /// Check whether this request contains a valid sum signature
    fn has_valid_sum_signature(&self) -> bool {
        let seed = &self.params.seed;
        let signature = &self.message.sum_signature;
        let pk = &self.header.participant_pk;
        pk.verify_detached(&signature, &[seed.as_slice(), b"sum"].concat())
    }

    /// Check whether this request comes from a participant that is eligible for the sum task.
    fn is_eligible_for_sum_task(&self) -> bool {
        self.message.sum_signature.is_eligible(self.params.sum)
    }

    /// Check whether this request contains a valid update signature
    fn has_valid_update_signature(&self) -> bool {
        let seed = &self.params.seed;
        let signature = &self.message.update_signature;
        let pk = &self.header.participant_pk;
        pk.verify_detached(&signature, &[seed.as_slice(), b"update"].concat())
    }

    /// Check whether this request comes from a participant that is
    /// eligible for the update task.
    fn is_eligible_for_update_task(&self) -> bool {
        self.message
            .update_signature
            .is_eligible(self.params.update)
    }
}
