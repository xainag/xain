use crate::state_machine::{
    coordinator::CoordinatorState,
    phases::{Phase, PhaseName, PhaseState},
    requests::RequestReceiver,
    StateError,
    StateMachine,
};

/// Shutdown state
#[derive(Debug)]
pub struct Shutdown;

#[async_trait]
impl<R> Phase<R> for PhaseState<R, Shutdown>
where
    R: Send,
{
    const NAME: PhaseName = PhaseName::Shutdown;

    /// Shuts down the [`StateMachine`].
    ///
    /// See the [module level documentation](../index.html) for more details.
    async fn run(&mut self) -> Result<(), StateError> {
        // clear the request channel
        self.request_rx.close();
        while self.request_rx.recv().await.is_some() {}
        Ok(())
    }

    fn next(self) -> Option<StateMachine<R>> {
        None
    }
}

impl<R> PhaseState<R, Shutdown> {
    /// Creates a new shutdown state.
    pub fn new(coordinator_state: CoordinatorState, request_rx: RequestReceiver<R>) -> Self {
        info!("state transition");
        Self {
            inner: Shutdown,
            coordinator_state,
            request_rx,
        }
    }
}
