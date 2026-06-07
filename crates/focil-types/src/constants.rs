// EIP-7805 / Heze fork constants.
// Source: https://github.com/ethereum/consensus-specs/blob/dev/specs/heze/beacon-chain.md

/// Number of validators per slot on the inclusion-list committee.
pub const INCLUSION_LIST_COMMITTEE_SIZE: usize = 16;

/// BLS domain type for inclusion list committee signatures (0x0E000000).
pub const DOMAIN_INCLUSION_LIST_COMMITTEE: [u8; 4] = [0x0E, 0x00, 0x00, 0x00];
