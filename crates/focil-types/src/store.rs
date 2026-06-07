use std::collections::{HashMap, HashSet};

use crate::inclusion_list::{InclusionList, Transaction};

/// Key: (slot, inclusion_list_committee_root).
type StoreKey = (u64, [u8; 32]);

/// In-memory store for inclusion lists received on the p2p network.
///
/// Spec: `specs/heze/inclusion-list.md` — `InclusionListStore`.
///
/// Rules:
/// - One IL per validator per (slot, committee_root).
/// - If a validator submits two *different* ILs for the same key → equivocator;
///   both ILs are discarded and the validator is blacklisted for that key.
/// - ILs received after the view-freeze cutoff are ignored.
#[derive(Debug, Default)]
pub struct InclusionListStore {
    inclusion_lists: HashMap<StoreKey, Vec<InclusionList>>,
    equivocators: HashMap<StoreKey, HashSet<u64>>,
}

impl InclusionListStore {
    pub fn new() -> Self {
        Self::default()
    }

    /// Process an incoming `InclusionList`.
    ///
    /// `is_before_cutoff` — caller sets this based on slot timing.
    /// ILs arriving after the view-freeze cutoff are not stored.
    pub fn process(&mut self, il: InclusionList, is_before_cutoff: bool) {
        let key: StoreKey = (il.slot, il.inclusion_list_committee_root);

        if self
            .equivocators
            .get(&key)
            .map_or(false, |s| s.contains(&il.validator_index))
        {
            return;
        }

        let stored = self.inclusion_lists.entry(key).or_default();

        if let Some(pos) = stored
            .iter()
            .position(|s| s.validator_index == il.validator_index)
        {
            if stored[pos] != il {
                // Equivocation: same validator, different IL for same key.
                self.equivocators
                    .entry(key)
                    .or_default()
                    .insert(il.validator_index);
                stored.remove(pos);
            }
            return;
        }

        if is_before_cutoff {
            stored.push(il);
        }
    }

    /// Return deduplicated union of all valid (non-equivocating) transactions
    /// for the given slot and committee root.
    ///
    /// Spec: `get_inclusion_list_transactions`.
    pub fn transactions(&self, slot: u64, committee_root: [u8; 32]) -> Vec<Transaction> {
        let key: StoreKey = (slot, committee_root);
        let equivocators = self.equivocators.get(&key);

        // Use the SSZ bytes as a deduplication key.
        let mut seen: HashSet<Vec<u8>> = HashSet::new();
        let mut out: Vec<Transaction> = Vec::new();

        if let Some(ils) = self.inclusion_lists.get(&key) {
            for il in ils {
                if equivocators.map_or(false, |eq| eq.contains(&il.validator_index)) {
                    continue;
                }
                for tx in il.transactions.iter() {
                    let raw: Vec<u8> = tx.iter().copied().collect();
                    if seen.insert(raw) {
                        out.push(tx.clone());
                    }
                }
            }
        }

        out
    }

    /// Return a bitmask (index → submitted?) over the committee.
    ///
    /// `committee` is the ordered validator-index vector for the slot
    /// (length == INCLUSION_LIST_COMMITTEE_SIZE).
    ///
    /// Spec: `get_inclusion_list_bits`.
    pub fn bits(&self, slot: u64, committee_root: [u8; 32], committee: &[u64]) -> Vec<bool> {
        let key: StoreKey = (slot, committee_root);
        let equivocators = self.equivocators.get(&key);

        let submitted: HashSet<u64> = self
            .inclusion_lists
            .get(&key)
            .map(|ils| {
                ils.iter()
                    .filter(|il| {
                        !equivocators.map_or(false, |eq| eq.contains(&il.validator_index))
                    })
                    .map(|il| il.validator_index)
                    .collect()
            })
            .unwrap_or_default();

        committee.iter().map(|vi| submitted.contains(vi)).collect()
    }

    /// Return `true` if `candidate_bits` is a superset of the locally observed bits.
    ///
    /// Spec: `is_inclusion_list_bits_inclusive`.
    pub fn is_bits_inclusive(
        &self,
        slot: u64,
        committee_root: [u8; 32],
        committee: &[u64],
        candidate_bits: &[bool],
    ) -> bool {
        let local = self.bits(slot, committee_root, committee);
        local
            .iter()
            .zip(candidate_bits.iter())
            .all(|(local_bit, candidate_bit)| *candidate_bit || !local_bit)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ssz_types::VariableList;

    fn make_il(slot: u64, validator_index: u64, txs: Vec<Vec<u8>>) -> InclusionList {
        let txs: Vec<Transaction> = txs
            .into_iter()
            .map(|v| VariableList::new(v).unwrap())
            .collect();
        let transactions = VariableList::new(txs).unwrap();
        InclusionList {
            slot,
            validator_index,
            inclusion_list_committee_root: [0u8; 32],
            transactions,
        }
    }

    #[test]
    fn stores_valid_il() {
        let mut store = InclusionListStore::new();
        store.process(make_il(1, 0, vec![vec![0x01]]), true);
        let txs = store.transactions(1, [0u8; 32]);
        assert_eq!(txs.len(), 1);
        assert_eq!(txs[0].to_vec(), vec![0x01u8]);
    }

    #[test]
    fn deduplicates_transactions() {
        let mut store = InclusionListStore::new();
        store.process(make_il(1, 0, vec![vec![0xAA]]), true);
        store.process(make_il(1, 1, vec![vec![0xAA], vec![0xBB]]), true);
        let txs = store.transactions(1, [0u8; 32]);
        assert_eq!(txs.len(), 2);
    }

    #[test]
    fn equivocator_discarded() {
        let mut store = InclusionListStore::new();
        store.process(make_il(1, 0, vec![vec![0x01]]), true);
        store.process(make_il(1, 0, vec![vec![0x02]]), true);
        let txs = store.transactions(1, [0u8; 32]);
        assert!(txs.is_empty(), "equivocator txs must be dropped");
    }

    #[test]
    fn after_cutoff_not_stored() {
        let mut store = InclusionListStore::new();
        store.process(make_il(1, 0, vec![vec![0x01]]), false);
        let txs = store.transactions(1, [0u8; 32]);
        assert!(txs.is_empty());
    }

    #[test]
    fn bits_and_is_bits_inclusive() {
        let mut store = InclusionListStore::new();
        let committee = vec![10u64, 11, 12];
        store.process(make_il(1, 10, vec![]), true);
        store.process(make_il(1, 12, vec![]), true);

        let bits = store.bits(1, [0u8; 32], &committee);
        assert_eq!(bits, vec![true, false, true]);

        assert!(store.is_bits_inclusive(1, [0u8; 32], &committee, &[true, true, true]));
        assert!(!store.is_bits_inclusive(1, [0u8; 32], &committee, &[false, true, true]));
    }
}
