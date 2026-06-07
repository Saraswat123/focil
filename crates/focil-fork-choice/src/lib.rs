//! EIP-7805 FOCIL fork-choice satisfaction check.
//!
//! Spec: `specs/heze/fork-choice.md` — `is_inclusion_list_satisfied`.

use focil_types::inclusion_list::Transaction;

/// Outcome of checking whether a block satisfies the IL constraints.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Satisfaction {
    /// Every IL transaction is either included or invalid at block start.
    Satisfied,
    /// One or more valid IL transactions are missing from the block.
    Unsatisfied { missing: Vec<Transaction> },
}

/// Check if `block_transactions` satisfies the IL constraints.
///
/// A block satisfies the ILs when, for every transaction T in `il_transactions`,
/// *either*:
///   (a) T appears in `block_transactions`, **or**
///   (b) T is invalid at block start (in `invalid_at_block_start`).
///
/// Spec: `is_inclusion_list_satisfied` in `specs/heze/fork-choice.md`.
pub fn is_inclusion_list_satisfied(
    block_transactions: &[Transaction],
    il_transactions: &[Transaction],
    invalid_at_block_start: &[Transaction],
) -> Satisfaction {
    let mut missing = Vec::new();

    for il_tx in il_transactions {
        let included = block_transactions.iter().any(|bt| bt == il_tx);
        let invalid = invalid_at_block_start.iter().any(|inv| inv == il_tx);

        if !included && !invalid {
            missing.push(il_tx.clone());
        }
    }

    if missing.is_empty() {
        Satisfaction::Satisfied
    } else {
        Satisfaction::Unsatisfied { missing }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tx(bytes: &[u8]) -> Transaction {
        Transaction::new(bytes.to_vec()).unwrap()
    }

    #[test]
    fn all_included_is_satisfied() {
        let result = is_inclusion_list_satisfied(
            &[tx(&[0xAA]), tx(&[0xBB])],
            &[tx(&[0xAA]), tx(&[0xBB])],
            &[],
        );
        assert_eq!(result, Satisfaction::Satisfied);
    }

    #[test]
    fn invalid_tx_excused() {
        let t = tx(&[0xCC]);
        let result = is_inclusion_list_satisfied(&[], &[t.clone()], &[t]);
        assert_eq!(result, Satisfaction::Satisfied);
    }

    #[test]
    fn missing_valid_tx_unsatisfied() {
        let tx_a = tx(&[0x01]);
        let tx_b = tx(&[0x02]);
        let result =
            is_inclusion_list_satisfied(&[tx_a.clone()], &[tx_a, tx_b.clone()], &[]);
        assert_eq!(result, Satisfaction::Unsatisfied { missing: vec![tx_b] });
    }

    #[test]
    fn empty_il_always_satisfied() {
        let result = is_inclusion_list_satisfied(&[], &[], &[]);
        assert_eq!(result, Satisfaction::Satisfied);
    }
}
