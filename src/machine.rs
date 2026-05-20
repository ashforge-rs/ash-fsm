use crate::{
    config::StateMachineConfig,
    error::FsmError,
    state::State,
    transition::{current_timestamp, TransitionRecord},
};

// ── StateMachine trait ────────────────────────────────────────────────────────

/// Core trait for implementing a finite state machine.
///
/// # Required methods
///
/// Implementors must provide:
/// - [`config`](StateMachine::config) — the static machine definition.
/// - [`current_state`](StateMachine::current_state) — a reference to the field
///   holding the current state.
/// - [`set_state`](StateMachine::set_state) — mutates that field.
/// - [`history`](StateMachine::history) — a reference to the history `Vec`.
/// - [`history_mut`](StateMachine::history_mut) — a mutable reference to the
///   same `Vec`.
///
/// All other methods have default implementations built on top of those five.
///
/// # Example
///
/// ```rust
/// use ash_fsm::{State, StateMachine};
/// use ash_fsm::config::StateMachineConfig;
/// use ash_fsm::transition::{Transition, TransitionRecord};
///
/// #[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// enum TicketState { Open, InProgress, Resolved, Closed }
/// impl State for TicketState {}
///
/// struct Ticket {
///     state: TicketState,
///     history: Vec<TransitionRecord<TicketState>>,
/// }
///
/// impl StateMachine for Ticket {
///     type State = TicketState;
///
///     fn config() -> StateMachineConfig<TicketState> {
///         StateMachineConfig::new(TicketState::Open)
///             .initial_states(vec![TicketState::Open])
///             .transitions(vec![
///                 Transition::new("start").from(vec![TicketState::Open]).to(vec![TicketState::InProgress]),
///                 Transition::new("resolve").from(vec![TicketState::InProgress]).to(vec![TicketState::Resolved]),
///                 Transition::new("close").from_any().to(vec![TicketState::Closed]),
///             ])
///     }
///
///     fn current_state(&self) -> &TicketState { &self.state }
///     fn set_state(&mut self, s: TicketState) { self.state = s; }
///     fn history(&self) -> &[TransitionRecord<TicketState>] { &self.history }
///     fn history_mut(&mut self) -> &mut Vec<TransitionRecord<TicketState>> { &mut self.history }
/// }
///
/// let mut ticket = Ticket { state: TicketState::Open, history: Vec::new() };
/// ticket.transition("start", TicketState::InProgress).unwrap();
/// assert_eq!(ticket.current_state(), &TicketState::InProgress);
/// assert_eq!(ticket.history().len(), 1);
/// ```
pub trait StateMachine {
    /// The state type used by this machine.
    type State: State;

    // ── required ──────────────────────────────────────────────────────────────

    /// Returns the static configuration for this machine type.
    ///
    /// Called on every operation that needs to consult transition rules.
    fn config() -> StateMachineConfig<Self::State>;

    /// Returns a reference to the machine's current state.
    fn current_state(&self) -> &Self::State;

    /// Overwrites the current state. Called only by the transition machinery;
    /// prefer [`transition`](StateMachine::transition) for normal use.
    fn set_state(&mut self, state: Self::State);

    /// Returns the history of all recorded transitions (oldest first).
    fn history(&self) -> &[TransitionRecord<Self::State>];

    /// Returns a mutable reference to the history buffer.
    fn history_mut(&mut self) -> &mut Vec<TransitionRecord<Self::State>>;

    // ── provided — transitions ────────────────────────────────────────────────

    /// Attempts to transition the machine to `target` via the named `action`.
    ///
    /// - Validates that a matching transition rule exists.
    /// - Appends a [`TransitionRecord`] to the history, evicting the oldest
    ///   entry if the history limit is exceeded.
    /// - Updates the current state.
    ///
    /// Returns [`FsmError::NoMatchingTransition`] if no rule covers the
    /// `(action, current_state → target)` triple.
    fn transition(
        &mut self,
        action: &str,
        target: Self::State,
    ) -> Result<(), FsmError<Self::State>> {
        let config = Self::config();
        let from = self.current_state().clone();

        if !config.is_transition_allowed(action, &from, &target) {
            return Err(FsmError::NoMatchingTransition {
                action: action.to_string(),
                from,
                to: target,
            });
        }

        let record = TransitionRecord {
            from: from.clone(),
            to: target.clone(),
            action: action.to_string(),
            timestamp: current_timestamp(),
        };

        let history = self.history_mut();
        if history.len() >= config.history_limit {
            history.remove(0);
        }
        history.push(record);

        self.set_state(target);
        Ok(())
    }

    // ── provided — validation ─────────────────────────────────────────────────

    /// Validates that the machine's current state is a legal initial state.
    ///
    /// Call this after constructing the machine to catch misconfiguration early.
    fn validate_initial_state(&self) -> Result<(), FsmError<Self::State>> {
        let config = Self::config();
        let state = self.current_state().clone();
        if config.is_valid_initial_state(&state) {
            Ok(())
        } else {
            Err(FsmError::InvalidInitialState {
                state,
                valid: config.initial_states.clone(),
            })
        }
    }

    // ── provided — introspection ──────────────────────────────────────────────

    /// Returns every state reachable from the current state via any action.
    fn possible_next_states(&self) -> Vec<Self::State> {
        Self::config().possible_next_states(self.current_state())
    }

    /// Returns every state reachable from the current state via the given action.
    fn possible_next_states_for_action(&self, action: &str) -> Vec<Self::State> {
        Self::config().possible_next_states_for_action(self.current_state(), action)
    }

    /// Returns `true` if transitioning to `target` via `action` would succeed.
    fn can_transition(&self, action: &str, target: &Self::State) -> bool {
        Self::config().is_transition_allowed(action, self.current_state(), target)
    }

    /// Returns all states known to this machine (explicit + extra + deprecated).
    fn all_states() -> Vec<Self::State> {
        Self::config().all_states()
    }

    /// Returns only the non-deprecated states.
    fn active_states() -> Vec<Self::State> {
        Self::config().active_states()
    }

    /// Returns every transition rule whose `from` endpoint covers the current state.
    fn available_transitions(&self) -> Vec<crate::transition::Transition<Self::State>> {
        Self::config()
            .transitions_from(self.current_state())
            .into_iter()
            .cloned()
            .collect()
    }

    // ── provided — serde ──────────────────────────────────────────────────────

    /// Serialises the transition history to a JSON string.
    ///
    /// Requires the `serde` feature (enabled by default) and that `Self::State`
    /// implements `serde::Serialize`.
    #[cfg(feature = "serde")]
    fn serialize_history(&self) -> serde_json::Result<String>
    where
        Self::State: serde::Serialize,
    {
        serde_json::to_string(self.history())
    }

    /// Serialises the transition history to a pretty-printed JSON string.
    #[cfg(feature = "serde")]
    fn serialize_history_pretty(&self) -> serde_json::Result<String>
    where
        Self::State: serde::Serialize,
    {
        serde_json::to_string_pretty(self.history())
    }

    /// Deserialises a history buffer from a JSON string and replaces the
    /// current history with it (up to the configured limit).
    #[cfg(feature = "serde")]
    fn deserialize_history(&mut self, json: &str) -> serde_json::Result<()>
    where
        Self::State: serde::de::DeserializeOwned,
    {
        let mut records: Vec<TransitionRecord<Self::State>> = serde_json::from_str(json)?;
        let limit = Self::config().history_limit;
        if records.len() > limit {
            records.drain(0..records.len() - limit);
        }
        *self.history_mut() = records;
        Ok(())
    }
}
