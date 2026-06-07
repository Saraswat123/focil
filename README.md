# focil

EIP-7805 **Fork-Choice Enforced Inclusion Lists (FOCIL)** — Rust implementation targeting the Ethereum **Heze** fork.

Spec reference: [`ethereum/consensus-specs/specs/heze/`](https://github.com/ethereum/consensus-specs/tree/dev/specs/heze)

---

## What is FOCIL?

FOCIL (EIP-7805) solves **censorship** at the base layer:

- Today (PBS): block builders can drop any transaction — including privacy-protocol txs
- With FOCIL: a committee of 16 validators each independently submit an **Inclusion List (IL)** from their local mempool. The fork-choice rule enforces that the block **must** include every transaction from any IL — or that transaction must be invalid at block start.
- If even **one** committee member sees your tx and includes it in their IL → builder has no choice but to include it.

**Privacy connection:** AA + FOCIL = privacy protocol txs (shielded pools, etc.) become uncensorable at the base layer.

---

## Crate Structure

```
focil/
├── crates/
│   ├── focil-types/        # Core SSZ types + in-memory store
│   ├── focil-fork-choice/  # Fork-choice satisfaction check
│   └── focil-store/        # Re-export facade for consumers
```

### `focil-types`

| File | What it implements | Spec |
|------|--------------------|------|
| `constants.rs` | `INCLUSION_LIST_COMMITTEE_SIZE = 16`, `DOMAIN_INCLUSION_LIST_COMMITTEE` | `beacon-chain.md` |
| `inclusion_list.rs` | `InclusionList` container (SSZ + TreeHash) | `beacon-chain.md` |
| `signed_inclusion_list.rs` | `SignedInclusionList` + `BlsSignature` type | `beacon-chain.md` |
| `store.rs` | `InclusionListStore` — full spec logic | `inclusion-list.md` |

**`InclusionList` fields (from spec):**
```rust
pub struct InclusionList {
    pub slot: u64,
    pub validator_index: u64,
    pub inclusion_list_committee_root: [u8; 32],
    pub transactions: VariableList<Transaction, MaxTransactionsPerPayload>,
}
```

**`InclusionListStore` implements all spec helpers:**
- `process_inclusion_list` — stores ILs, detects equivocators, respects view-freeze cutoff
- `get_inclusion_list_transactions` — deduplicated union of valid ILs for a slot
- `get_inclusion_list_bits` — bitvector of which committee members submitted
- `is_inclusion_list_bits_inclusive` — superset check for fork-choice

### `focil-fork-choice`

Implements `is_inclusion_list_satisfied` from `specs/heze/fork-choice.md`:

```rust
pub fn is_inclusion_list_satisfied(
    block_transactions: &[Transaction],
    il_transactions: &[Transaction],
    invalid_at_block_start: &[Transaction],
) -> Satisfaction { ... }
```

A block satisfies the ILs when every IL transaction is either:
- included in the block, **or**
- invalid at block start (wrong nonce, insufficient balance, etc.)

Returns `Satisfaction::Satisfied` or `Satisfaction::Unsatisfied { missing }`.

---

## Run Tests

```bash
cargo test
```

```
test inclusion_list::tests::ssz_round_trip          ... ok
test inclusion_list::tests::slot_and_validator_index_preserved ... ok
test signed_inclusion_list::tests::ssz_round_trip   ... ok
test store::tests::stores_valid_il                  ... ok
test store::tests::deduplicates_transactions        ... ok
test store::tests::equivocator_discarded            ... ok
test store::tests::after_cutoff_not_stored          ... ok
test store::tests::bits_and_is_bits_inclusive       ... ok
test tests::all_included_is_satisfied               ... ok
test tests::invalid_tx_excused                      ... ok
test tests::missing_valid_tx_unsatisfied            ... ok
test tests::empty_il_always_satisfied               ... ok

12 passed
```

---

## How It Works — Example Flow

```
Slot N starts
│
├── t=0..cutoff: IL committee (16 validators) each observe local mempool
│   └── Each validator → InclusionList { slot, validator_index, transactions }
│       └── Signs → SignedInclusionList
│       └── Gossips on `inclusion_list` p2p topic
│
├── InclusionListStore.process(il, is_before_cutoff=true)
│   ├── Ignore equivocators (same validator, different IL = blacklisted)
│   └── Store valid ILs per (slot, committee_root)
│
├── t=cutoff: Block builder collects ILs
│   └── store.transactions(slot, committee_root) → union of all valid txs
│   └── Builder MUST include all of them in the block
│
└── Block arrives → fork-choice calls is_inclusion_list_satisfied(block_txs, il_txs, invalid)
    ├── Satisfied   → block gets full fork-choice weight
    └── Unsatisfied → block penalized in LMD-GHOST
```

---

## What's Next

- [ ] P2P gossip validation (`inclusion_list` topic, BLS signature check)
- [ ] `get_inclusion_list_committee` (requires beacon state accessor)
- [ ] `payload_inclusion_list_satisfaction` map in fork-choice store
- [ ] `InclusionListByCommitteeIndices` req/resp protocol
- [ ] Integration with Lighthouse beacon chain

---

## References

- [EIP-7805](https://eips.ethereum.org/EIPS/eip-7805) — FOCIL spec
- [consensus-specs/heze](https://github.com/ethereum/consensus-specs/tree/dev/specs/heze) — beacon-chain, inclusion-list, fork-choice, p2p
- [Lighthouse](https://github.com/sigp/lighthouse) — target client for future integration
