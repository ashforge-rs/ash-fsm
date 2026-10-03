//! # ash-fsm
//!
//! A configurable, trait-based finite state machine library for Rust, inspired by
//! [AshStateMachine](https://hexdocs.pm/ash_state_machine/api-reference.html).
//!
//! ## Design
//!
//! - **Traits first** — define your machine by implementing [`StateMachine`] on
//!   any struct. The trait provides all transition logic as default methods.
//! - **Declarative transitions** — describe allowed `(action, from → to)` triples
//!   in [`StateMachineConfig`]. Wildcards (`*`) cover any action or any state.
//! - **History** — every successful transition is recorded as a
//!   [`TransitionRecord`] in a bounded ring buffer (default 100 entries,
//!   configurable per machine).
//! - **Serialization** — the `serde` feature (on by default) lets you round-trip
//!   history and config through any serde-compatible format.
//!
//! ## Quick start
//!
//! ```rust
//! use ash_fsm::{State, StateMachine};
//! use ash_fsm::config::StateMachineConfig;
//! use ash_fsm::transition::{Transition, TransitionRecord};
//!
//! // 1. Define your state enum.
//! #[derive(Debug, Clone, PartialEq, Eq, Hash)]
//! enum OrderState { Pending, Active, Shipped, Closed }
//! impl State for OrderState {}
//!
//! // 2. Define your domain struct.
//! struct Order {
//!     state: OrderState,
//!     history: Vec<TransitionRecord<OrderState>>,
//! }
//!
//! // 3. Implement the trait.
//! impl StateMachine for Order {
//!     type State = OrderState;
//!
//!     fn config() -> StateMachineConfig<OrderState> {
//!         StateMachineConfig::new(OrderState::Pending)
//!             .transitions(vec![
//!                 Transition::new("activate")
//!                     .from(vec![OrderState::Pending])
//!                     .to(vec![OrderState::Active]),
//!                 Transition::new("ship")
//!                     .from(vec![OrderState::Active])
//!                     .to(vec![OrderState::Shipped]),
//!                 // Close is allowed from any state.
//!                 Transition::new("close")
//!                     .from_any()
//!                     .to(vec![OrderState::Closed]),
//!             ])
//!     }
//!
//!     fn current_state(&self) -> &OrderState { &self.state }
//!     fn set_state(&mut self, s: OrderState) { self.state = s; }
//!     fn history(&self) -> &[TransitionRecord<OrderState>] { &self.history }
//!     fn history_mut(&mut self) -> &mut Vec<TransitionRecord<OrderState>> { &mut self.history }
//! }
//!
//! // 4. Use it.
//! let mut order = Order { state: OrderState::Pending, history: Vec::new() };
//! order.validate_initial_state().unwrap();
//! order.transition("activate", OrderState::Active).unwrap();
//! order.transition("ship", OrderState::Shipped).unwrap();
//! assert_eq!(order.history().len(), 2);
//! assert_eq!(order.current_state(), &OrderState::Shipped);
//!
//! // Invalid transition returns an error and leaves state unchanged.
//! let err = order.transition("activate", OrderState::Active);
//! assert!(err.is_err());
//! ```

pub mod config;
pub mod error;
pub mod machine;
pub mod shared;
pub mod state;
pub mod transition;

// ── top-level re-exports ──────────────────────────────────────────────────────

pub use config::StateMachineConfig;
pub use error::FsmError;
pub use machine::StateMachine;
pub use shared::SharedMachine;
pub use state::State;
pub use transition::{Transition, TransitionEndpoint, TransitionRecord};

#[cfg(feature = "async")]
pub use shared::AsyncMachine;

/// Convenient glob import: `use ash_fsm::prelude::*;`
pub mod prelude {
    #[cfg(feature = "async")]
    pub use crate::AsyncMachine;
    pub use crate::{
        FsmError, SharedMachine, State, StateMachine, StateMachineConfig, Transition,
        TransitionEndpoint, TransitionRecord,
    };
}

// ── tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::DEFAULT_HISTORY_LIMIT;

    // ── helpers ───────────────────────────────────────────────────────────────

    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    enum TicketState {
        Received,
        NeedsMoreInfo,
        InProgress,
        Resolved,
        Closed,
        Archived,
    }

    impl State for TicketState {}

    struct Ticket {
        state: TicketState,
        history: Vec<TransitionRecord<TicketState>>,
    }

    impl Ticket {
        fn new() -> Self {
            Self {
                state: TicketState::Received,
                history: Vec::new(),
            }
        }
    }

    impl StateMachine for Ticket {
        type State = TicketState;

        fn config() -> StateMachineConfig<TicketState> {
            StateMachineConfig::new(TicketState::Received)
                .initial_states(vec![TicketState::Received])
                .deprecated_states(vec![TicketState::Archived])
                .transitions(vec![
                    Transition::new("request_info")
                        .from(vec![TicketState::Received, TicketState::InProgress])
                        .to(vec![TicketState::NeedsMoreInfo]),
                    Transition::new("start")
                        .from(vec![TicketState::Received, TicketState::NeedsMoreInfo])
                        .to(vec![TicketState::InProgress]),
                    Transition::new("resolve")
                        .from(vec![TicketState::InProgress])
                        .to(vec![TicketState::Resolved]),
                    // close is allowed from any state
                    Transition::new("close")
                        .from_any()
                        .to(vec![TicketState::Closed]),
                ])
        }

        fn current_state(&self) -> &TicketState {
            &self.state
        }
        fn set_state(&mut self, s: TicketState) {
            self.state = s;
        }
        fn history(&self) -> &[TransitionRecord<TicketState>] {
            &self.history
        }
        fn history_mut(&mut self) -> &mut Vec<TransitionRecord<TicketState>> {
            &mut self.history
        }
    }

    // ── transition tests ──────────────────────────────────────────────────────

    #[test]
    fn valid_transition_succeeds() {
        let mut t = Ticket::new();
        t.transition("start", TicketState::InProgress).unwrap();
        assert_eq!(t.current_state(), &TicketState::InProgress);
    }

    #[test]
    fn invalid_transition_returns_error() {
        let mut t = Ticket::new();
        let err = t.transition("resolve", TicketState::Resolved);
        assert!(matches!(err, Err(FsmError::NoMatchingTransition { .. })));
        // State is unchanged
        assert_eq!(t.current_state(), &TicketState::Received);
    }

    #[test]
    fn wildcard_from_allows_close_from_any_state() {
        let mut t = Ticket::new();
        t.transition("close", TicketState::Closed).unwrap();
        assert_eq!(t.current_state(), &TicketState::Closed);
    }

    // ── history tests ─────────────────────────────────────────────────────────

    #[test]
    fn history_records_transitions() {
        let mut t = Ticket::new();
        t.transition("start", TicketState::InProgress).unwrap();
        t.transition("resolve", TicketState::Resolved).unwrap();

        assert_eq!(t.history().len(), 2);
        assert_eq!(t.history()[0].from, TicketState::Received);
        assert_eq!(t.history()[0].to, TicketState::InProgress);
        assert_eq!(t.history()[0].action, "start");
        assert_eq!(t.history()[1].from, TicketState::InProgress);
        assert_eq!(t.history()[1].to, TicketState::Resolved);
    }

    #[test]
    fn history_evicts_oldest_at_limit() {
        struct TinyMachine {
            state: TicketState,
            history: Vec<TransitionRecord<TicketState>>,
        }
        impl StateMachine for TinyMachine {
            type State = TicketState;
            fn config() -> StateMachineConfig<TicketState> {
                StateMachineConfig::new(TicketState::Received)
                    .history_limit(2)
                    .transitions(vec![Transition::new("*").from_any().to_any()])
            }
            fn current_state(&self) -> &TicketState {
                &self.state
            }
            fn set_state(&mut self, s: TicketState) {
                self.state = s;
            }
            fn history(&self) -> &[TransitionRecord<TicketState>] {
                &self.history
            }
            fn history_mut(&mut self) -> &mut Vec<TransitionRecord<TicketState>> {
                &mut self.history
            }
        }

        let mut m = TinyMachine {
            state: TicketState::Received,
            history: Vec::new(),
        };
        m.transition("*", TicketState::InProgress).unwrap();
        m.transition("*", TicketState::Resolved).unwrap();
        m.transition("*", TicketState::Closed).unwrap(); // should evict first

        assert_eq!(m.history().len(), 2);
        assert_eq!(m.history()[0].from, TicketState::InProgress);
    }

    // ── validation tests ──────────────────────────────────────────────────────

    #[test]
    fn validate_initial_state_ok() {
        let t = Ticket::new();
        t.validate_initial_state().unwrap();
    }

    #[test]
    fn validate_initial_state_err_for_invalid() {
        let t = Ticket {
            state: TicketState::Resolved,
            history: Vec::new(),
        };
        let err = t.validate_initial_state();
        assert!(matches!(err, Err(FsmError::InvalidInitialState { .. })));
    }

    // ── introspection tests ───────────────────────────────────────────────────

    #[test]
    fn possible_next_states_from_received() {
        let t = Ticket::new();
        let mut next = t.possible_next_states();
        next.sort_by(|a, b| format!("{a:?}").cmp(&format!("{b:?}")));
        // From Received: can go to NeedsMoreInfo (via request_info), InProgress (via start), Closed (via close)
        assert!(next.contains(&TicketState::NeedsMoreInfo));
        assert!(next.contains(&TicketState::InProgress));
        assert!(next.contains(&TicketState::Closed));
    }

    #[test]
    fn possible_next_states_for_action() {
        let t = Ticket::new();
        let next = t.possible_next_states_for_action("close");
        assert_eq!(next, vec![TicketState::Closed]);
    }

    #[test]
    fn can_transition_predicate() {
        let t = Ticket::new();
        assert!(t.can_transition("start", &TicketState::InProgress));
        assert!(!t.can_transition("resolve", &TicketState::Resolved));
    }

    #[test]
    fn all_states_includes_deprecated() {
        let all = Ticket::all_states();
        assert!(all.contains(&TicketState::Archived));
    }

    #[test]
    fn active_states_excludes_deprecated() {
        let active = Ticket::active_states();
        assert!(!active.contains(&TicketState::Archived));
    }

    #[test]
    fn default_history_limit_is_100() {
        assert_eq!(DEFAULT_HISTORY_LIMIT, 100);
        let config = Ticket::config();
        assert_eq!(config.history_limit, 100);
    }

    // ── shared / async tests ──────────────────────────────────────────────────

    mod shared_tests {
        use super::*;
        use crate::shared::SharedMachine;
        use std::thread;

        fn make_shared() -> SharedMachine<Ticket> {
            SharedMachine::new(Ticket::new())
        }

        #[test]
        fn shared_machine_transitions() {
            let m = make_shared();
            m.transition("start", TicketState::InProgress).unwrap();
            assert_eq!(m.current_state(), TicketState::InProgress);
        }

        #[test]
        fn shared_machine_invalid_transition_returns_error() {
            let m = make_shared();
            let err = m.transition("resolve", TicketState::Resolved);
            assert!(matches!(err, Err(FsmError::NoMatchingTransition { .. })));
            assert_eq!(m.current_state(), TicketState::Received);
        }

        #[test]
        fn shared_machine_history_snapshot() {
            let m = make_shared();
            m.transition("start", TicketState::InProgress).unwrap();
            let history = m.history();
            assert_eq!(history.len(), 1);
            assert_eq!(history[0].action, "start");
        }

        #[test]
        fn shared_machine_send_across_threads() {
            let m = make_shared();
            let handle = m.clone();
            let t = thread::spawn(move || {
                handle.transition("start", TicketState::InProgress).unwrap();
            });
            t.join().unwrap();
            assert_eq!(m.current_state(), TicketState::InProgress);
        }

        #[test]
        fn shared_machine_concurrent_read() {
            let m = make_shared();
            m.transition("start", TicketState::InProgress).unwrap();

            let handles: Vec<_> = (0..4)
                .map(|_| {
                    let clone = m.clone();
                    thread::spawn(move || clone.current_state())
                })
                .collect();

            for h in handles {
                assert_eq!(h.join().unwrap(), TicketState::InProgress);
            }
        }

        #[test]
        fn shared_machine_with_closure() {
            let m = make_shared();
            let result = m.with(|inner| {
                inner.transition("start", TicketState::InProgress).unwrap();
                inner.history().len()
            });
            assert_eq!(result, 1);
        }

        #[cfg(feature = "async")]
        mod async_tests {
            use super::*;
            use crate::shared::AsyncMachine;

            fn make_async() -> AsyncMachine<Ticket> {
                AsyncMachine::new(Ticket::new())
            }

            #[tokio::test]
            async fn async_machine_transitions() {
                let m = make_async();
                m.transition("start", TicketState::InProgress)
                    .await
                    .unwrap();
                assert_eq!(m.current_state().await, TicketState::InProgress);
            }

            #[tokio::test]
            async fn async_machine_invalid_transition() {
                let m = make_async();
                let err = m.transition("resolve", TicketState::Resolved).await;
                assert!(matches!(err, Err(FsmError::NoMatchingTransition { .. })));
            }

            #[tokio::test]
            async fn async_machine_history() {
                let m = make_async();
                m.transition("start", TicketState::InProgress)
                    .await
                    .unwrap();
                let hist = m.history().await;
                assert_eq!(hist.len(), 1);
                assert_eq!(hist[0].action, "start");
            }

            #[tokio::test]
            async fn async_machine_send_across_tasks() {
                let m = make_async();
                let handle = m.clone();
                tokio::spawn(async move {
                    handle
                        .transition("start", TicketState::InProgress)
                        .await
                        .unwrap();
                })
                .await
                .unwrap();
                assert_eq!(m.current_state().await, TicketState::InProgress);
            }

            #[tokio::test]
            async fn async_machine_with_closure() {
                let m = make_async();
                let result = m
                    .with(|inner| {
                        inner.transition("start", TicketState::InProgress).unwrap();
                        inner.history().len()
                    })
                    .await;
                assert_eq!(result, 1);
            }
        }
    }

    #[cfg(feature = "serde")]
    mod serde_tests {
        use super::*;

        #[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
        enum SimpleState {
            A,
            B,
            C,
        }
        impl State for SimpleState {}

        struct SimpleMachine {
            state: SimpleState,
            history: Vec<TransitionRecord<SimpleState>>,
        }
        impl StateMachine for SimpleMachine {
            type State = SimpleState;
            fn config() -> StateMachineConfig<SimpleState> {
                StateMachineConfig::new(SimpleState::A).transitions(vec![
                    Transition::new("go")
                        .from(vec![SimpleState::A])
                        .to(vec![SimpleState::B]),
                    Transition::new("go")
                        .from(vec![SimpleState::B])
                        .to(vec![SimpleState::C]),
                ])
            }
            fn current_state(&self) -> &SimpleState {
                &self.state
            }
            fn set_state(&mut self, s: SimpleState) {
                self.state = s;
            }
            fn history(&self) -> &[TransitionRecord<SimpleState>] {
                &self.history
            }
            fn history_mut(&mut self) -> &mut Vec<TransitionRecord<SimpleState>> {
                &mut self.history
            }
        }

        #[test]
        fn history_serializes_and_deserializes() {
            let mut m = SimpleMachine {
                state: SimpleState::A,
                history: Vec::new(),
            };
            m.transition("go", SimpleState::B).unwrap();
            m.transition("go", SimpleState::C).unwrap();

            let json = m.serialize_history().unwrap();
            assert!(json.contains("\"from\":\"A\""));

            let mut m2 = SimpleMachine {
                state: SimpleState::C,
                history: Vec::new(),
            };
            m2.deserialize_history(&json).unwrap();
            assert_eq!(m2.history().len(), 2);
            assert_eq!(m2.history()[0].from, SimpleState::A);
            assert_eq!(m2.history()[1].from, SimpleState::B);
        }

        #[test]
        fn config_serializes() {
            let config = SimpleMachine::config();
            let json = serde_json::to_string(&config).unwrap();
            assert!(json.contains("initial_states"));
            assert!(json.contains("transitions"));
        }
    }
}
