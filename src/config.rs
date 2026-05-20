#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use std::collections::HashSet;

use crate::{
    state::State,
    transition::Transition,
};

/// Default maximum number of [`TransitionRecord`](crate::transition::TransitionRecord)s
/// kept in a machine's history.
pub const DEFAULT_HISTORY_LIMIT: usize = 100;

// ── StateMachineConfig ────────────────────────────────────────────────────────

/// All configuration for a state machine definition.
///
/// Build one with [`StateMachineConfig::new`] and the fluent builder methods,
/// then return it from [`StateMachine::config`](crate::machine::StateMachine::config).
///
/// # Example
///
/// ```rust
/// use ash_fsm::config::StateMachineConfig;
/// use ash_fsm::transition::Transition;
///
/// # #[derive(Debug, Clone, PartialEq, Eq, Hash)] enum Order { Pending, Active, Closed }
/// # impl ash_fsm::State for Order {}
/// let config = StateMachineConfig::new(Order::Pending)
///     .initial_states(vec![Order::Pending])
///     .transitions(vec![
///         Transition::new("activate").from(vec![Order::Pending]).to(vec![Order::Active]),
///         Transition::new("close").from_any().to(vec![Order::Closed]),
///     ]);
/// ```
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct StateMachineConfig<S> {
    /// The set of states that are valid starting states.
    pub initial_states: Vec<S>,

    /// The state assigned automatically when no explicit initial state is
    /// provided. Must be a member of `initial_states`.
    pub default_initial_state: S,

    /// States that were previously valid but are now obsolete. They are still
    /// listed in [`all_states`](StateMachineConfig::all_states) but are
    /// excluded from wildcard (`*`) expansion.
    pub deprecated_states: Vec<S>,

    /// Additional states that are not mentioned in any transition's `from`/`to`
    /// list but must be reachable via a wildcard (`*`) endpoint.
    pub extra_states: Vec<S>,

    /// Ordered list of allowed transitions.
    pub transitions: Vec<Transition<S>>,

    /// Maximum number of history entries to retain. Oldest entries are evicted
    /// once this limit is exceeded. Defaults to [`DEFAULT_HISTORY_LIMIT`].
    pub history_limit: usize,
}

impl<S: State> StateMachineConfig<S> {
    /// Creates a minimal config with `default_initial_state` as the only
    /// valid initial state and no transitions yet.
    pub fn new(default_initial_state: S) -> Self {
        Self {
            initial_states: vec![default_initial_state.clone()],
            default_initial_state,
            deprecated_states: Vec::new(),
            extra_states: Vec::new(),
            transitions: Vec::new(),
            history_limit: DEFAULT_HISTORY_LIMIT,
        }
    }

    // ── builder methods ───────────────────────────────────────────────────────

    /// Overrides the valid initial states.
    pub fn initial_states(mut self, states: Vec<S>) -> Self {
        self.initial_states = states;
        self
    }

    /// Overrides the default initial state.
    pub fn default_initial_state(mut self, state: S) -> Self {
        self.default_initial_state = state;
        self
    }

    /// Sets the deprecated states for this machine.
    pub fn deprecated_states(mut self, states: Vec<S>) -> Self {
        self.deprecated_states = states;
        self
    }

    /// Sets the extra states for this machine (required for wildcard expansion).
    pub fn extra_states(mut self, states: Vec<S>) -> Self {
        self.extra_states = states;
        self
    }

    /// Replaces the transition list with the given transitions.
    pub fn transitions(mut self, transitions: Vec<Transition<S>>) -> Self {
        self.transitions = transitions;
        self
    }

    /// Appends a single transition to the existing list.
    pub fn add_transition(mut self, transition: Transition<S>) -> Self {
        self.transitions.push(transition);
        self
    }

    /// Overrides the history size limit.
    pub fn history_limit(mut self, limit: usize) -> Self {
        self.history_limit = limit;
        self
    }

    // ── introspection ─────────────────────────────────────────────────────────

    /// Returns the deduplicated set of all known states: every state explicitly
    /// named in any transition endpoint, plus `initial_states`, `extra_states`,
    /// and `deprecated_states`.
    ///
    /// States that only appear via the wildcard (`Any`) endpoint are NOT
    /// automatically included; add them to `extra_states` for full wildcard
    /// expansion.
    pub fn all_states(&self) -> Vec<S> {
        let mut set = HashSet::new();

        // States mentioned in transition endpoints
        for t in &self.transitions {
            if let Some(states) = t.from.explicit_states() {
                set.extend(states.iter().cloned());
            }
            if let Some(states) = t.to.explicit_states() {
                set.extend(states.iter().cloned());
            }
        }

        // Always include initial states
        set.extend(self.initial_states.iter().cloned());
        // Explicitly provided extras and deprecated states
        set.extend(self.extra_states.iter().cloned());
        set.extend(self.deprecated_states.iter().cloned());

        set.into_iter().collect()
    }

    /// Returns only the non-deprecated states (the "active" set).
    pub fn active_states(&self) -> Vec<S> {
        let deprecated: HashSet<_> = self.deprecated_states.iter().collect();
        self.all_states()
            .into_iter()
            .filter(|s| !deprecated.contains(s))
            .collect()
    }

    /// Returns `true` if `state` is a valid initial state.
    pub fn is_valid_initial_state(&self, state: &S) -> bool {
        self.initial_states.contains(state)
    }

    /// Returns every state that can be reached from `current` via any action.
    pub fn possible_next_states(&self, current: &S) -> Vec<S> {
        let all = self.all_states();
        all.into_iter()
            .filter(|target| {
                self.transitions
                    .iter()
                    .any(|t| t.from.matches(current) && t.to.matches(target))
            })
            .collect()
    }

    /// Returns every state that can be reached from `current` using `action`.
    pub fn possible_next_states_for_action(&self, current: &S, action: &str) -> Vec<S> {
        let all = self.all_states();
        all.into_iter()
            .filter(|target| {
                self.transitions
                    .iter()
                    .any(|t| t.matches_action_from(action, current) && t.to.matches(target))
            })
            .collect()
    }

    /// Returns `true` if the given (`action`, `from` → `to`) triple is covered
    /// by at least one transition rule.
    pub fn is_transition_allowed(&self, action: &str, from: &S, to: &S) -> bool {
        self.transitions.iter().any(|t| t.matches(action, from, to))
    }

    /// Returns every transition whose `from` endpoint covers `current`.
    pub fn transitions_from(&self, current: &S) -> Vec<&Transition<S>> {
        self.transitions
            .iter()
            .filter(|t| t.from.matches(current))
            .collect()
    }
}
