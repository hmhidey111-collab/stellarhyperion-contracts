# Resource budget benchmarks

The router's budget baseline is emitted by the pinned Soroban test host, not by a wall-clock
microbenchmark. This keeps results reproducible across machines and makes a regression actionable:
the report names the operation, input size, stored-record count, CPU instructions, and memory
bytes.

## Reproduce

From the repository root, with the committed `soroban/Cargo.lock` and the repository's pinned
Rust toolchain:

```sh
cargo test --manifest-path soroban/Cargo.toml -p hyperion-router \
  print_reproducible_quote_dispatch_inbound_and_settlement_budgets -- --nocapture
```

The output contains lines in this format:

```text
HYPERION_BUDGET case=settlement input_bytes=64 records=32 cpu=... memory=...
```

Compare the `records=1` and `records=32` settlement rows. A change is actionable when the delta
grows unexpectedly with the stored-record count, or when the same case's input size changes
without an intentional wire-format change. The fixture uses a large-but-valid set of 32 parked
claims, exact integer amounts, a real CCTP route, and a recipient that becomes eligible before
settlement. It does not use malformed or synthetic records.

The four measured paths are quote, outbound dispatch, inbound delivery, and claim settlement.
They are independent fresh worlds, so setup costs cannot leak from one row into the next. The
test intentionally prints measurements instead of asserting machine-specific ceilings; protocol
limits and host cost models belong to the pinned toolchain and should be reviewed when either
changes.
