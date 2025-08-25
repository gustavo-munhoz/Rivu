use crate::core::instance_header::InstanceHeader;
use crate::core::instances::instance::Instance;
use std::io::Error;

/// Pull-based interface for data streams that produce `Instance`s.
///
/// Implementations may represent finite datasets (e.g., files) or unbounded
/// generators. All returned instances must conform to the same, immutable
/// [`InstanceHeader`] for the lifetime of the stream.
pub trait Stream {
    /// Returns the stream header (relation name, attributes, class index).
    ///
    /// The header must remain valid and immutable for the entire lifetime of
    /// the stream. Every instance yielded by [`next_instance`] must match this
    /// schema (same number/order of attributes and class index).
    fn header(&self) -> &InstanceHeader;

    /// Indicates whether the stream *may* produce more instances.
    ///
    /// Finite streams should return `false` once exhausted. Unbounded streams
    /// (e.g., generators) typically return `true` always.
    ///
    /// This call should be cheap and side effect free. If it returns `false`,
    /// a subsequent call to [`next_instance`] must return `None`.
    fn has_more_instances(&self) -> bool;

    /// Produces the next instance, or `None` if the stream is exhausted.
    ///
    /// Implementations should not panic on normal end-of-stream conditions.
    /// For sources that can contain malformed records, implementations may
    /// choose to skip invalid rows and continue, or end the stream (returning
    /// `None`) and optionally flip [`has_more_instances`] to `false`.
    ///
    /// Returned instances must be compatible with [`header`].
    fn next_instance(&mut self) -> Option<Box<dyn Instance>>;

    /// Resets the stream to its initial state.
    ///
    /// For file-backed streams, this typically seeks back to the start of the
    /// data section; for generators, it usually re-seeds the RNG and clears
    /// internal counters. The header must remain unchanged.
    ///
    /// Returns an error if the underlying source cannot be reopened or sought.
    fn restart(&mut self) -> Result<(), Error>;
}
