#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use std::sync::LazyLock;

use ash_time::{HlcClock, HlcTimestamp};

use crate::state::State;

// ── Process-global HLC clock ──────────────────────────────────────────────────

/// The process-wide [`HlcClock`] used to stamp every transition.
///
/// `HlcClock` is internally thread-safe (`Send + Sync`) and is designed to be
/// shared via `Arc` or as a global. Using a single instance per process
/// ensures that timestamps are strictly monotone across all machines running
/// in the same process.
static CLOCK: LazyLock<HlcClock> = LazyLock::new(HlcClock::new);

/// Returns a new [`HlcTimestamp`] for the current instant.
///
/// Falls back to the last recorded timestamp if the wall clock is unavailable.
pub(crate) fn current_timestamp() -> HlcTimestamp {
    CLOCK.now().unwrap_or_else(|_| CLOCK.last())
}

// ── Serde support for HlcTimestamp ───────────────────────────────────────────

/// Custom serde module for [`HlcTimestamp`].
///
/// `HlcTimestamp` has no built-in serde support; this module serializes it as
/// `{ "physical": <u64 nanos>, "logical": <u32> }` which preserves all
/// information and round-trips exactly.
///
/// Used via `#[serde(with = "hlc_serde")]`.
#[cfg(feature = "serde")]
pub mod hlc_serde {
    use ash_time::HlcTimestamp;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    #[derive(Serialize, Deserialize)]
    struct Helper {
        physical: u64,
        logical: u32,
    }

    pub fn serialize<S: Serializer>(ts: &HlcTimestamp, ser: S) -> Result<S::Ok, S::Error> {
        Helper {
            physical: ts.physical,
            logical: ts.logical,
        }
        .serialize(ser)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(de: D) -> Result<HlcTimestamp, D::Error> {
        let h = Helper::deserialize(de)?;
        Ok(HlcTimestamp {
            physical: h.physical,
            logical: h.logical,
        })
    }
}

// ── TransitionEndpoint ────────────────────────────────────────────────────────

/// One side (from or to) of a [`Transition`] rule.
///
/// `Any` is the wildcard (`*`) and matches every state. `States` restricts
/// matching to the listed states.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum TransitionEndpoint<S> {
    /// Wildcard — matches any state.
    Any,
    /// Matches only the listed states.
    States(Vec<S>),
}

impl<S: State> TransitionEndpoint<S> {
    /// Returns `true` if `state` is covered by this endpoint.
    pub fn matches(&self, state: &S) -> bool {
        match self {
            Self::Any => true,
            Self::States(states) => states.contains(state),
        }
    }

    /// Returns the explicitly listed states, or `None` for the wildcard.
    pub fn explicit_states(&self) -> Option<&[S]> {
        match self {
            Self::Any => None,
            Self::States(states) => Some(states),
        }
    }
}

// ── Transition ────────────────────────────────────────────────────────────────

/// A rule that describes when a state transition is allowed.
///
/// A transition is matched when all three fields agree with the attempted
/// operation:
/// - `action` is the name of the action (or `"*"` to match any action).
/// - `from` is the set of states the machine must currently be in.
/// - `to` is the set of states the machine is allowed to move to.
///
/// # Builder
///
/// Use [`Transition::new`] followed by [`from`](Transition::from),
/// [`from_any`](Transition::from_any), [`to`](Transition::to), and
/// [`to_any`](Transition::to_any) to construct transitions fluently:
///
/// ```rust
/// use ash_fsm::transition::{Transition, TransitionEndpoint};
///
/// # #[derive(Debug, Clone, PartialEq, Eq, Hash)] enum S { A, B, C }
/// # impl ash_fsm::State for S {}
/// let t = Transition::new("approve")
///     .from(vec![S::A])
///     .to(vec![S::B, S::C]);
/// ```
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Transition<S> {
    /// The action name this transition applies to. Use `"*"` for any action.
    pub action: String,
    /// States from which this transition may start.
    pub from: TransitionEndpoint<S>,
    /// States to which this transition may lead.
    pub to: TransitionEndpoint<S>,
}

impl<S: State> Transition<S> {
    /// Creates a new transition for `action`. The `from` and `to` endpoints
    /// default to [`TransitionEndpoint::Any`].
    pub fn new(action: impl Into<String>) -> Self {
        Self {
            action: action.into(),
            from: TransitionEndpoint::Any,
            to: TransitionEndpoint::Any,
        }
    }

    /// Restricts the transition to originate from the given states.
    pub fn from(mut self, states: Vec<S>) -> Self {
        self.from = TransitionEndpoint::States(states);
        self
    }

    /// Allows the transition from any state (wildcard).
    pub fn from_any(mut self) -> Self {
        self.from = TransitionEndpoint::Any;
        self
    }

    /// Restricts the transition to lead to the given states.
    pub fn to(mut self, states: Vec<S>) -> Self {
        self.to = TransitionEndpoint::States(states);
        self
    }

    /// Allows the transition to lead to any state (wildcard).
    pub fn to_any(mut self) -> Self {
        self.to = TransitionEndpoint::Any;
        self
    }

    // ── internal helpers ──────────────────────────────────────────────────────

    /// Returns `true` if this transition matches the given action name.
    ///
    /// A transition with action `"*"` matches every action.
    pub(crate) fn action_matches(&self, action: &str) -> bool {
        self.action == "*" || self.action == action
    }

    /// Returns `true` if this rule covers the full (`action`, `from`, `to`) triple.
    pub(crate) fn matches(&self, action: &str, from: &S, to: &S) -> bool {
        self.action_matches(action) && self.from.matches(from) && self.to.matches(to)
    }

    /// Returns `true` if this rule covers the (`action`, `from`) pair,
    /// regardless of the target state.
    pub(crate) fn matches_action_from(&self, action: &str, from: &S) -> bool {
        self.action_matches(action) && self.from.matches(from)
    }
}

// ── TransitionRecord ──────────────────────────────────────────────────────────

/// An entry in the machine's transition history.
///
/// Stored in [`StateMachine::history`](crate::machine::StateMachine::history).
/// The oldest entries are evicted once the history limit is reached (default 100).
///
/// The `timestamp` field is a [`HlcTimestamp`] produced by a process-global
/// [`HlcClock`]. Hybrid Logical Clocks stay close to wall time while remaining
/// strictly monotone within a process, making them suitable for ordering
/// transitions causally.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct TransitionRecord<S> {
    /// State before the transition.
    pub from: S,
    /// State after the transition.
    pub to: S,
    /// Name of the action that triggered the transition.
    pub action: String,
    /// HLC timestamp at the time of the transition.
    ///
    /// `physical` is nanoseconds since the Unix epoch; `logical` breaks ties
    /// within the same nanosecond. Use `ts.happened_before(other)` to compare.
    #[cfg_attr(feature = "serde", serde(with = "hlc_serde"))]
    pub timestamp: HlcTimestamp,
}
