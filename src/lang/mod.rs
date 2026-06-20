//! Language-specific petname generators.
//!
//! The default [`Petnames`][`crate::Petnames`] generator is English-shaped: it
//! relies on English having (almost) no inflection and a fixed
//! adverb–adjective–noun order, so any combination of words is well-formed.
//! Other languages need their own models to produce names that read naturally
//! to a native speaker.
//!
//! Each language is a distinct type implementing [`Generator`][`crate::Generator`].
//! There is deliberately no shared abstraction beyond that trait: where
//! commonality emerges (for example between Germanic languages) it can be
//! factored out later.

#[cfg(feature = "lang-turkish")]
pub mod turkish;
