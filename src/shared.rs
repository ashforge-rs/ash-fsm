//! Thread-safe handles to a state machine.
//!
//! # Sync — [`SharedMachine`]
//!
//! [`SharedMachine<M>`] wraps any `M: StateMachine + Send` in an
//! `Arc<Mutex<M>>` so it can be cloned and shared across threads without any
//! additional locking ceremony:
//!
//! ```rust
//! use std::thread;
//! use ash_fsm::{State, StateMachine};
//! use ash_fsm::config::StateMachineConfig;
//! use ash_fsm::transition::{Transition, TransitionRecord};
//! use ash_fsm::shared::SharedMachine;
//!
//! #[derive(Debug, Clone, PartialEq, Eq, Hash)]
//! enum S { Idle, Running, Done }
//! impl State for S {}
//!
//! struct Worker { state: S, history: Vec<TransitionRecord<S>> }
//! impl StateMachine for Worker {
//!     type State = S;
//!     fn config() -> StateMachineConfig<S> {
//!         StateMachineConfig::new(S::Idle)
//!             .transitions(vec![
//!                 Transition::new("run").from(vec![S::Idle]).to(vec![S::Running]),
//!                 Transition::new("finish").from(vec![S::Running]).to(vec![S::Done]),
//!             ])
//!     }
//!     fn current_state(&self) -> &S { &self.state }
//!     fn set_state(&mut self, s: S) { self.state = s; }
//!     fn history(&self) -> &[TransitionRecord<S>] { &self.history }
//!     fn history_mut(&mut self) -> &mut Vec<TransitionRecord<S>> { &mut self.history }
//! }
//!
//! let machine = SharedMachine::new(Worker { state: S::Idle, history: vec![] });
//! let handle = machine.clone();
//!
//! let t = thread::spawn(move || {
//!     handle.transition("run", S::Running).unwrap();
//! });
//! t.join().unwrap();
//!
//! assert_eq!(machine.current_state(), S::Running);
//! ```
//!
//! # Async — [`AsyncMachine`]
//!
//! [`AsyncMachine<M>`] is the tokio-flavoured counterpart: it wraps the
//! machine in `Arc<tokio::sync::Mutex<M>>` and exposes `async fn` methods
//! so it can be awaited across `.await` points without blocking the executor.
//!
//! Enable the `async` feature to use it:
//!
//! ```toml
//! ash-fsm = { version = "0.1", features = ["async"] }
//! ```

use std::sync::{Arc, Mutex, PoisonError};

use crate::{
    error::FsmError,
    machine::StateMachine,
    transition::TransitionRecord,
};

// ── SharedMachine ─────────────────────────────────────────────────────────────

/// A cheaply-cloneable, thread-safe handle to a [`StateMachine`].
///
/// Internally `Arc<Mutex<M>>`. All methods lock the mutex, delegate to the
/// inner machine, and release the lock before returning. Panics on mutex
/// poisoning are handled by recovering the guard (the machine state is still
/// consistent because transitions are atomic).
///
/// Clone to distribute to threads; each clone shares the same machine.
pub struct SharedMachine<M: StateMachine>(Arc<Mutex<M>>);

impl<M: StateMachine + Send> SharedMachine<M> {
    /// Wraps `machine` in a new `SharedMachine`.
    pub fn new(machine: M) -> Self {
        Self(Arc::new(Mutex::new(machine)))
    }

    /// Transitions the machine via `action` → `target`.
    ///
    /// See [`StateMachine::transition`] for semantics.
    pub fn transition(
        &self,
        action: &str,
        target: M::State,
    ) -> Result<(), FsmError<M::State>> {
        self.lock().transition(action, target)
    }

    /// Returns a clone of the current state.
    pub fn current_state(&self) -> M::State {
        self.lock().current_state().clone()
    }

    /// Returns a snapshot of the history buffer.
    pub fn history(&self) -> Vec<TransitionRecord<M::State>> {
        self.lock().history().to_vec()
    }

    /// Returns `true` if transitioning to `target` via `action` would succeed.
    pub fn can_transition(&self, action: &str, target: &M::State) -> bool {
        self.lock().can_transition(action, target)
    }

    /// Returns every state reachable from the current state via any action.
    pub fn possible_next_states(&self) -> Vec<M::State> {
        self.lock().possible_next_states()
    }

    /// Returns every state reachable from the current state via `action`.
    pub fn possible_next_states_for_action(&self, action: &str) -> Vec<M::State> {
        self.lock().possible_next_states_for_action(action)
    }

    /// Validates that the current state is a legal initial state.
    pub fn validate_initial_state(&self) -> Result<(), FsmError<M::State>> {
        self.lock().validate_initial_state()
    }

    /// Locks the machine and runs `f` with exclusive mutable access.
    ///
    /// Use this when you need to perform multiple operations atomically.
    ///
    /// ```rust
    /// # use ash_fsm::{State, StateMachine};
    /// # use ash_fsm::config::StateMachineConfig;
    /// # use ash_fsm::transition::{Transition, TransitionRecord};
    /// # use ash_fsm::shared::SharedMachine;
    /// # #[derive(Debug, Clone, PartialEq, Eq, Hash)] enum S { A, B }
    /// # impl State for S {}
    /// # struct M { state: S, history: Vec<TransitionRecord<S>> }
    /// # impl StateMachine for M {
    /// #     type State = S;
    /// #     fn config() -> StateMachineConfig<S> { StateMachineConfig::new(S::A).transitions(vec![Transition::new("go").from(vec![S::A]).to(vec![S::B])]) }
    /// #     fn current_state(&self) -> &S { &self.state }
    /// #     fn set_state(&mut self, s: S) { self.state = s; }
    /// #     fn history(&self) -> &[TransitionRecord<S>] { &self.history }
    /// #     fn history_mut(&mut self) -> &mut Vec<TransitionRecord<S>> { &mut self.history }
    /// # }
    /// let shared = SharedMachine::new(M { state: S::A, history: vec![] });
    /// let history_len = shared.with(|m| {
    ///     m.transition("go", S::B).unwrap();
    ///     m.history().len()
    /// });
    /// assert_eq!(history_len, 1);
    /// ```
    pub fn with<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut M) -> R,
    {
        f(&mut *self.lock())
    }

    // ── internal ──────────────────────────────────────────────────────────────

    fn lock(&self) -> std::sync::MutexGuard<'_, M> {
        self.0.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

impl<M: StateMachine + Send> Clone for SharedMachine<M> {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

// SAFETY: Arc<Mutex<M>> is Send + Sync when M: Send.
unsafe impl<M: StateMachine + Send> Send for SharedMachine<M> {}
unsafe impl<M: StateMachine + Send> Sync for SharedMachine<M> {}

// ── AsyncMachine ──────────────────────────────────────────────────────────────

/// A cheaply-cloneable, async-safe handle to a [`StateMachine`].
///
/// Wraps `Arc<tokio::sync::Mutex<M>>`. All methods are `async` and yield
/// across `.await` points without blocking the tokio thread pool.
///
/// Requires the **`async`** feature:
///
/// ```toml
/// ash-fsm = { version = "0.1", features = ["async"] }
/// ```
#[cfg(feature = "async")]
pub struct AsyncMachine<M: StateMachine>(Arc<tokio::sync::Mutex<M>>);

#[cfg(feature = "async")]
impl<M: StateMachine + Send + 'static> AsyncMachine<M> {
    /// Wraps `machine` in a new `AsyncMachine`.
    pub fn new(machine: M) -> Self {
        Self(Arc::new(tokio::sync::Mutex::new(machine)))
    }

    /// Transitions the machine via `action` → `target`.
    pub async fn transition(
        &self,
        action: &str,
        target: M::State,
    ) -> Result<(), FsmError<M::State>> {
        self.0.lock().await.transition(action, target)
    }

    /// Returns a clone of the current state.
    pub async fn current_state(&self) -> M::State {
        self.0.lock().await.current_state().clone()
    }

    /// Returns a snapshot of the history buffer.
    pub async fn history(&self) -> Vec<TransitionRecord<M::State>> {
        self.0.lock().await.history().to_vec()
    }

    /// Returns `true` if transitioning to `target` via `action` would succeed.
    pub async fn can_transition(&self, action: &str, target: &M::State) -> bool {
        self.0.lock().await.can_transition(action, target)
    }

    /// Returns every state reachable from the current state via any action.
    pub async fn possible_next_states(&self) -> Vec<M::State> {
        self.0.lock().await.possible_next_states()
    }

    /// Returns every state reachable from the current state via `action`.
    pub async fn possible_next_states_for_action(&self, action: &str) -> Vec<M::State> {
        self.0.lock().await.possible_next_states_for_action(action)
    }

    /// Validates that the current state is a legal initial state.
    pub async fn validate_initial_state(&self) -> Result<(), FsmError<M::State>> {
        self.0.lock().await.validate_initial_state()
    }

    /// Locks the machine and runs `f` with exclusive mutable access inside an
    /// async context. The lock is held for the duration of `f`.
    pub async fn with<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut M) -> R,
    {
        f(&mut *self.0.lock().await)
    }
}

#[cfg(feature = "async")]
impl<M: StateMachine + Send + 'static> Clone for AsyncMachine<M> {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}
