# ash-fsm

[![Crates.io](https://img.shields.io/crates/v/ash-fsm)](https://crates.io/crates/ash-fsm)
[![docs.rs](https://img.shields.io/docsrs/ash-fsm)](https://docs.rs/ash-fsm)
[![CI](https://github.com/ashforge-rs/ash-fsm/actions/workflows/rust.yml/badge.svg)](https://github.com/ashforge-rs/ash-fsm/actions/workflows/rust.yml)

A configurable, trait-based finite state machine library for Rust, inspired by
[AshStateMachine](https://hexdocs.pm/ash_state_machine/api-reference.html).

> **Disclaimer:** this repository contains AI-generated code.

You implement one trait, `StateMachine`, on your own struct and describe the
allowed transitions declaratively. The trait's default methods provide
transition checking, introspection and history tracking.

## Features

- **Trait-first:** implement `StateMachine` on any struct. No macros required.
- **Declarative transitions:** `(action, from → to)` rules, with wildcards for
  any action or any state.
- **Bounded history:** every successful transition is recorded in a ring
  buffer (100 entries by default, configurable per machine), timestamped with
  a Hybrid Logical Clock from [`ash-time`](https://docs.rs/ash-time).
- **Introspection:** query the possible next states and check whether a
  transition is allowed before attempting it.
- **Deprecated states:** still valid, but excluded from wildcard expansion.
- **Shared access:** `SharedMachine` (and `AsyncMachine` with the `async`
  feature) wraps a machine for use across threads or tasks.

## Installation

```sh
cargo add ash-fsm
```

## Quick start

```rust
use ash_fsm::{State, StateMachine, StateMachineConfig, Transition, TransitionRecord};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum OrderState {
    Pending,
    Active,
    Shipped,
    Closed,
}

impl State for OrderState {}

struct Order {
    state: OrderState,
    history: Vec<TransitionRecord<OrderState>>,
}

impl StateMachine for Order {
    type State = OrderState;

    fn config() -> StateMachineConfig<OrderState> {
        StateMachineConfig::new(OrderState::Pending)
            .initial_states(vec![OrderState::Pending])
            .transitions(vec![
                Transition::new("activate")
                    .from(vec![OrderState::Pending])
                    .to(vec![OrderState::Active]),
                Transition::new("ship")
                    .from(vec![OrderState::Active])
                    .to(vec![OrderState::Shipped]),
                // "close" is allowed from any state.
                Transition::new("close")
                    .from_any()
                    .to(vec![OrderState::Closed]),
            ])
    }

    fn current_state(&self) -> &OrderState { &self.state }
    fn set_state(&mut self, s: OrderState) { self.state = s; }
    fn history(&self) -> &[TransitionRecord<OrderState>] { &self.history }
    fn history_mut(&mut self) -> &mut Vec<TransitionRecord<OrderState>> { &mut self.history }
}

let mut order = Order { state: OrderState::Pending, history: Vec::new() };
order.validate_initial_state().unwrap();

order.transition("activate", OrderState::Active).unwrap();
order.transition("ship", OrderState::Shipped).unwrap();
assert_eq!(order.current_state(), &OrderState::Shipped);
assert_eq!(order.history().len(), 2);

// Introspection.
assert!(order.possible_next_states().contains(&OrderState::Closed));
assert!(!order.can_transition("activate", &OrderState::Active));

// An invalid transition returns an error and leaves the state unchanged.
assert!(order.transition("activate", OrderState::Active).is_err());
```

## Configuration

| Option | Default | Description |
|---|---|---|
| `default_initial_state` | *(required)* | Assigned when no state is provided |
| `initial_states` | `[default_initial_state]` | States valid as starting states |
| `transitions` | `[]` | Ordered list of `(action, from → to)` rules |
| `deprecated_states` | `[]` | Valid, but excluded from wildcard expansion |
| `extra_states` | `[]` | States not named in any transition but reachable through wildcards |
| `history_limit` | `100` | Maximum number of history entries kept |

Use `"*"` as the action name to match any action, and `.from_any()` /
`.to_any()` to match any state.

## Cargo features

| Feature | Default | Enables |
|---|---|---|
| `serde` | yes | `Serialize`/`Deserialize` for history and configuration, plus `serialize_history` / `deserialize_history` |
| `async` | no | `AsyncMachine`, backed by a `tokio` mutex |

## Minimum supported Rust version

Rust 1.88 or later.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md). To report a security issue, see
[SECURITY.md](SECURITY.md).

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <https://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <https://opensource.org/licenses/MIT>)

at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
