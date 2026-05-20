# ash-fsm

[![Crates.io](https://img.shields.io/crates/v/ash-fsm)](https://crates.io/crates/ash-fsm)
[![docs.rs](https://img.shields.io/docsrs/ash-fsm)](https://docs.rs/ash-fsm)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue)](LICENSE)
[![CI](https://github.com/ashforge-rs/ash-fsm/actions/workflows/rust.yml/badge.svg)](https://github.com/ashforge-rs/ash-fsm/actions/workflows/rust.yml)

A configurable, trait-based finite state machine library for Rust, inspired by
[AshStateMachine](https://hexdocs.pm/ash_state_machine/api-reference.html).

## Overview

`ash-fsm` lets you model domain objects as finite state machines with clear,
declarative transition rules. It is designed around a single `StateMachine`
trait that you implement on any struct — the library provides all of the
transition logic, validation, introspection, and history tracking as default
methods.

## Features

- **Trait-first design** — implement `StateMachine` on any struct; no macros required.
- **Declarative transitions** — describe `(action, from → to)` rules with wildcard (`*`) support.
- **Bounded history** — every successful transition is logged in a ring buffer (default 100 entries, configurable per machine).
- **Introspection** — query possible next states, available transitions, and whether a transition is allowed before attempting it.
- **Deprecated states** — mark states as deprecated without removing them from the type; they remain valid but are excluded from wildcard expansion.
- **Serde support** — the optional `serde` feature (on by default) lets you serialize/deserialize history and configuration.

## Installation

```toml
[dependencies]
ash-fsm = "0.1"

# Without serde
ash-fsm = { version = "0.1", default-features = false }
```

## Quick Start

```rust
use ash_fsm::{State, StateMachine};
use ash_fsm::config::StateMachineConfig;
use ash_fsm::transition::{Transition, TransitionRecord};

// 1. Define your state enum.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum OrderState {
    Pending,
    Active,
    Shipped,
    Closed,
}

impl State for OrderState {}

// 2. Define your domain struct.
struct Order {
    state: OrderState,
    history: Vec<TransitionRecord<OrderState>>,
}

// 3. Implement the trait — five required methods, everything else is free.
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
                // "close" is allowed from any state
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

// 4. Use it.
let mut order = Order { state: OrderState::Pending, history: Vec::new() };
order.validate_initial_state().unwrap();

order.transition("activate", OrderState::Active).unwrap();
order.transition("ship", OrderState::Shipped).unwrap();

assert_eq!(order.current_state(), &OrderState::Shipped);
assert_eq!(order.history().len(), 2);

// Query possible next states
let next = order.possible_next_states();
assert!(next.contains(&OrderState::Closed));

// Check before committing
assert!(!order.can_transition("activate", &OrderState::Active));

// Invalid transitions return errors and leave state unchanged
let err = order.transition("activate", OrderState::Active);
assert!(err.is_err());
```

## Configuration Reference

| Option | Default | Description |
|---|---|---|
| `initial_states` | `[default_initial_state]` | States valid as starting states |
| `default_initial_state` | *(required)* | Assigned when no state is provided |
| `deprecated_states` | `[]` | Valid but excluded from wildcard `*` expansion |
| `extra_states` | `[]` | States not mentioned in transitions but reachable via wildcards |
| `transitions` | `[]` | Ordered list of `(action, from → to)` rules |
| `history_limit` | `100` | Maximum number of history entries retained |

## Transition Wildcards

Use `"*"` as the action name to match any action, and `.from_any()` / `.to_any()` to
match any state:

```rust
// Allow any action to transition from any state to Closed
Transition::new("*").from_any().to(vec![State::Closed])
```

## History

Every successful `transition()` call appends a `TransitionRecord` to the machine's
history. The timestamp is an [`HlcTimestamp`](https://docs.rs/ash-time) from a
process-global Hybrid Logical Clock — strictly monotone within the process and
anchored to wall time.

```rust
for record in order.history() {
    // happened_before() checks causal order
    println!("{:?} → {:?} via '{}' physical={}ns logical={}",
        record.from, record.to, record.action,
        record.timestamp.physical, record.timestamp.logical);
}
```

## Serialization (serde feature)

With `#[derive(Serialize, Deserialize)]` on your state enum you can round-trip history:

```rust
let json = order.serialize_history().unwrap();
let json_pretty = order.serialize_history_pretty().unwrap();
order.deserialize_history(&json).unwrap();
```

`StateMachineConfig` and `TransitionRecord` also derive `Serialize`/`Deserialize`
when the feature is enabled.

## Documentation

- [API Documentation](https://docs.rs/ash-fsm)
- [Changelog](CHANGELOG.md)

## Development

### Prerequisites

- Rust 1.70 or higher

### Building

```bash
cargo build
cargo test
cargo test --no-default-features   # without serde
cargo clippy -- -D warnings
cargo fmt --check
```

### Makefile

A `Makefile` is included with common development targets:

```bash
make test
make lint
make fmt
```

## Security

Please see [SECURITY.md](SECURITY.md) for vulnerability reporting instructions.

## License

Licensed under the [Apache License, Version 2.0](LICENSE).

### Building

```bash
# Build the project
cargo build

# Run tests
cargo test

# Run with all features
cargo test --all-features
```

### Running Examples

```bash
# TODO: Add example commands
cargo run --example basic
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the Apache-2.0 License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

TODO: Add acknowledgments, credits, or inspirations

## Contact

TODO: Add contact information or links to discussions/issues

---

**Template Information**: This project was created from the ashforge-rs template starter. Remove this section when ready for production.
