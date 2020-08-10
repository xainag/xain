use crate::state_machine::{
    phases::{Idle, Phase, PhaseName, PhaseState, Shared, Shutdown},
    RoundFailed,
    StateMachine,
};
use thiserror::Error;

/// Error that can occur during the execution of the [`StateMachine`].
#[derive(Error, Debug)]
pub enum StateError {
    #[error("state failed: channel error: {0}")]
    ChannelError(&'static str),
    #[error("state failed: round error: {0}")]
    RoundError(#[from] RoundFailed),
    #[error("state failed: phase timeout: {0}")]
    TimeoutError(#[from] tokio::time::Elapsed),
}

impl PhaseState<StateError> {
    /// Creates a new error state.
    pub fn new(shared: Shared, error: StateError) -> Self {
        info!("state transition");
        Self {
            inner: error,
            shared,
        }
    }
}

#[async_trait]
impl Phase for PhaseState<StateError> {
    const NAME: PhaseName = PhaseName::Error;

    async fn run(&mut self) -> Result<(), StateError> {
        error!("state transition failed! error: {:?}", self.inner);

        info!("broadcasting error phase event");
        self.shared.io.events.broadcast_phase(PhaseName::Error);

        Ok(())
    }

    /// Moves from the error state to the next state.
    ///
    /// See the [module level documentation](../index.html) for more details.
    fn next(self) -> Option<StateMachine> {
        Some(match self.inner {
            StateError::ChannelError(_) => PhaseState::<Shutdown>::new(self.shared).into(),
            _ => PhaseState::<Idle>::new(self.shared).into(),
        })
    }
}
