use ssz_derive::{Decode, Encode};
use ssz_types::{
    typenum::{U1048576, U1073741824},
    VariableList,
};
use tree_hash_derive::TreeHash;

/// Max bytes per single transaction (matches MAX_BYTES_PER_TRANSACTION in consensus-specs).
pub type MaxBytesPerTransaction = U1073741824;

/// Max transactions per payload (matches MAX_TRANSACTIONS_PER_PAYLOAD in consensus-specs).
pub type MaxTransactionsPerPayload = U1048576;

/// A single raw EIP-2718 encoded transaction (opaque bytes).
pub type Transaction = VariableList<u8, MaxBytesPerTransaction>;

/// Bounded list of transactions — same cap as MAX_TRANSACTIONS_PER_PAYLOAD.
pub type Transactions = VariableList<Transaction, MaxTransactionsPerPayload>;

/// A single committee member's inclusion list for a given slot.
///
/// Spec: `specs/heze/beacon-chain.md` — `InclusionList` container.
#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, TreeHash)]
pub struct InclusionList {
    /// Slot this IL covers.
    pub slot: u64,
    /// Beacon-chain validator index of the submitting committee member.
    pub validator_index: u64,
    /// `hash_tree_root` of the IL committee vector for this slot.
    /// Used to detect equivocation across committee rotations.
    pub inclusion_list_committee_root: [u8; 32],
    /// Ordered list of raw transactions the builder must include.
    pub transactions: Transactions,
}

#[cfg(test)]
mod tests {
    use super::*;
    use ssz::{Decode as _, Encode as _};

    fn sample_il() -> InclusionList {
        let tx: Transaction = VariableList::new(vec![0xde_u8, 0xad, 0xbe, 0xef]).unwrap();
        let transactions: Transactions = VariableList::new(vec![tx]).unwrap();
        InclusionList {
            slot: 42,
            validator_index: 7,
            inclusion_list_committee_root: [1u8; 32],
            transactions,
        }
    }

    #[test]
    fn ssz_round_trip() {
        let il = sample_il();
        let encoded = il.as_ssz_bytes();
        let decoded = InclusionList::from_ssz_bytes(&encoded).expect("decode failed");
        assert_eq!(il, decoded);
    }

    #[test]
    fn slot_and_validator_index_preserved() {
        let il = sample_il();
        assert_eq!(il.slot, 42);
        assert_eq!(il.validator_index, 7);
    }
}
