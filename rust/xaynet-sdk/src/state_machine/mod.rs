// Important the macro_use modules must be declared first for the
// macro to be used in the other modules (until declarative macros are stable)
#[macro_use]
mod phase;
mod io;
mod phases;
#[allow(clippy::module_inception)]
mod state_machine;

// It is useful to re-export everything within this module because
// there are lot of interdependencies between all the sub-modules
pub use self::{
    io::PassiveNotifier,
    state_machine::{StateMachine, TransitionOutcome},
};
use self::{
    io::{boxed_io, IO},
    phase::{Phase, Progress, SerializableState, SharedState, State, Step},
    phases::{Awaiting, NewRound, Sum, Sum2, Update},
};
