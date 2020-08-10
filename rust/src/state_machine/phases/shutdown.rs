use crate::state_machine::{
    phases::{Phase, PhaseName, PhaseState, Shared},
    StateError,
    StateMachine,
};

/// Shutdown state
#[derive(Debug)]
pub struct Shutdown;

#[async_trait]
impl Phase for PhaseState<Shutdown> {
    const NAME: PhaseName = PhaseName::Shutdown;

    /// Shuts down the [`StateMachine`].
    ///
    /// See the [module level documentation](../index.html) for more details.
    async fn run(&mut self) -> Result<(), StateError> {
        // clear the request channel
        self.shared.io.request_rx.close();
        while self.shared.io.request_rx.recv().await.is_some() {}
        Ok(())
    }

    fn next(self) -> Option<StateMachine> {
        None
    }
}

impl PhaseState<Shutdown> {
    /// Creates a new shutdown state.
    pub fn new(shared: Shared) -> Self {
        info!("state transition");
        Self {
            inner: Shutdown,
            shared,
        }
    }
}
