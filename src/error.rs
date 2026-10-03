use thiserror::Error;

/// Errors that can occur during state machine operations.
#[derive(Debug, Clone, PartialEq, Error)]
pub enum FsmError<S>
where
    S: std::fmt::Debug + Clone,
{
    /// Returned when attempting to start the machine in a state that is not listed
    /// in [`StateMachineConfig::initial_states`](crate::config::StateMachineConfig::initial_states).
    #[error("Invalid initial state `{state:?}`. Valid initial states are: {valid:?}")]
    InvalidInitialState { state: S, valid: Vec<S> },

    /// Returned when [`StateMachine::transition`](crate::machine::StateMachine::transition)
    /// is called with an action/from/to combination that matches no configured transition.
    #[error("No matching transition: action `{action}` from state `{from:?}` to state `{to:?}`")]
    NoMatchingTransition { action: String, from: S, to: S },
}
