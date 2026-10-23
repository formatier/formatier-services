//! Domain-level entities and shared iterator extensions.
//!
//! This module exposes common domain types and utilities used by application
//! crates.
pub mod entities;

/// Adds fallible collection to iterators of values convertible into `Result`.
pub trait TryCollectExt<I, T, E>: Iterator<Item = I> + Sized
where
    I: Into<Result<T, E>>,
{
    /// Collects iterator items and stops at the first conversion error.
    ///
    /// # Returns
    /// The collected output, or the first conversion error encountered.
    fn try_collect<B: FromIterator<T>>(self) -> Result<B, E> {
        let mut error = None;

        let collected: B = self
            .map_while(|item| match item.into() {
                Ok(v) => Some(v),
                Err(err) => {
                    error = Some(err);
                    None
                }
            })
            .collect();

        match error {
            Some(err) => Err(err),
            None => Ok(collected),
        }
    }
}

impl<S, I, T, E> TryCollectExt<I, T, E> for S
where
    S: Iterator<Item = I>,
    I: Into<Result<T, E>>,
{
}
