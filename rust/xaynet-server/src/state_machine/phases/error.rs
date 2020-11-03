use crate::{
    state_machine::{
        phases::{Idle, Phase, PhaseName, PhaseState, Shared, Shutdown},
        StateMachine,
        UnmaskGlobalModelError,
    },
    storage::api::Storage,
};
use std::time::Duration;
use tokio::time::delay_for;

#[cfg(feature = "metrics")]
use crate::metrics;

use thiserror::Error;

/// Error that can occur during the execution of the [`StateMachine`].
#[derive(Error, Debug)]
pub enum PhaseStateError {
    #[error("channel error: {0}")]
    Channel(&'static str),
    #[error("unmask global model error: {0}")]
    UnmaskGlobalModel(#[from] UnmaskGlobalModelError),
    #[error("phase timeout")]
    Timeout(#[from] tokio::time::Elapsed),
    #[cfg(feature = "model-persistence")]
    #[error("saving the global model failed: {0}")]
    SaveGlobalModel(crate::storage::s3::S3Error),
    #[error("saving the global model failed: {0}")]
    Storage(#[from] crate::storage::api::StorageError),
}

impl<Store: Storage> PhaseState<PhaseStateError, Store> {
    /// Creates a new error state.
    pub fn new(shared: Shared<Store>, error: PhaseStateError) -> Self {
        Self {
            inner: error,
            shared,
        }
    }
}

#[async_trait]
impl<Store: Storage> Phase<Store> for PhaseState<PhaseStateError, Store> {
    const NAME: PhaseName = PhaseName::Error;

    async fn run(&mut self) -> Result<(), PhaseStateError> {
        error!("state failed: {}", self.inner);

        metrics!(
            self.shared.metrics_tx,
            metrics::phase::error::emit(&self.inner)
        );

        if let PhaseStateError::Storage(_) = self.inner {
            // a simple loop that stops as soon as the redis client has reconnected to a redis
            // instance. Reconnecting a lost connection is handled internally by
            // redis::aio::ConnectionManager

            while self.shared.store.get_coordinator_state().await.is_err() {
                info!("try to reconnect to Redis in 5 sec");
                delay_for(Duration::from_secs(5)).await;
            }
        }

        Ok(())
    }

    /// Moves from the error state to the next state.
    ///
    /// See the [module level documentation](../index.html) for more details.
    fn next(self) -> Option<StateMachine<Store>> {
        Some(match self.inner {
            PhaseStateError::Channel(_) => PhaseState::<Shutdown, _>::new(self.shared).into(),
            _ => PhaseState::<Idle, _>::new(self.shared).into(),
        })
    }
}
