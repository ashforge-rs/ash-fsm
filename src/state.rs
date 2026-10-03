/// Marker trait for types that can be used as states in a state machine.
///
/// Any type that satisfies all of the supertraits automatically satisfies `State`.
/// The typical implementation is a plain `enum`:
///
/// ```rust
/// #[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// enum TicketState {
///     Open,
///     InProgress,
///     Resolved,
///     Closed,
/// }
///
/// impl ash_fsm::State for TicketState {}
/// ```
///
/// When the `serde` feature is enabled (the default), you should also derive
/// `serde::Serialize` and `serde::Deserialize` so the full machine can be
/// serialized.
pub trait State:
    Clone + PartialEq + Eq + std::hash::Hash + std::fmt::Debug + Send + Sync + 'static
{
}
