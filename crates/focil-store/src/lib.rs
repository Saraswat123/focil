//! Thin re-export + validator-facing API wrapping the core store.
//!
//! The p2p handler receives a `SignedInclusionList`, validates it, then calls
//! `FocilStore::on_inclusion_list`. Fork-choice calls `get_satisfaction` to
//! decide whether a block payload satisfies the ILs.

pub use focil_fork_choice::{Satisfaction, is_inclusion_list_satisfied};
pub use focil_types::{InclusionList, InclusionListStore, SignedInclusionList};
