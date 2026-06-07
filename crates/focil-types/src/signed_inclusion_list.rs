use ssz_derive::{Decode, Encode};
use ssz_types::{typenum::U96, FixedVector};
use tree_hash_derive::TreeHash;

use crate::inclusion_list::InclusionList;

/// BLS signature as a fixed 96-byte vector (SSZ/TreeHash compatible).
pub type BlsSignature = FixedVector<u8, U96>;

/// `InclusionList` with a BLS signature from the submitting validator.
///
/// Spec: `specs/heze/beacon-chain.md` — `SignedInclusionList` container.
#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, TreeHash)]
pub struct SignedInclusionList {
    pub message: InclusionList,
    pub signature: BlsSignature,
}

#[cfg(test)]
mod tests {
    use super::*;
    use ssz::{Decode as _, Encode as _};

    fn sample() -> SignedInclusionList {
        SignedInclusionList {
            message: InclusionList {
                slot: 1,
                validator_index: 0,
                inclusion_list_committee_root: [0u8; 32],
                transactions: Default::default(),
            },
            signature: BlsSignature::default(),
        }
    }

    #[test]
    fn ssz_round_trip() {
        let sil = sample();
        let encoded = sil.as_ssz_bytes();
        let decoded = SignedInclusionList::from_ssz_bytes(&encoded).expect("decode failed");
        assert_eq!(sil, decoded);
    }
}
