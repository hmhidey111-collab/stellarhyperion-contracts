//! Reproducible resource baselines for the four user-visible router paths.
//!
//! These are intentionally tests rather than a second benchmarking framework. Soroban's test
//! host already exposes the same CPU and memory counters used by the contract test suite, and
//! keeping the fixtures here means a baseline can be rerun against a pinned lockfile without
//! introducing a new dependency or hiding setup assumptions in a benchmark harness.

use super::{
    doubles::{evm_destination, Delivery},
    setup::{World, EVM_DECIMALS, HUNDRED},
};
use crate::types::{Destination, OutboundRequest};
use hyperion_core::{AddressKind, RouteKind};
use soroban_sdk::{testutils::Address as _, Address};

const SMALL_RECORDS: u32 = 1;
const LARGE_RECORDS: u32 = 32;

#[derive(Clone, Copy)]
struct Budget {
    cpu: u64,
    memory: u64,
}

fn budget(w: &World) -> Budget {
    Budget {
        cpu: w.env.cost_estimate().budget().cpu_instruction_cost(),
        memory: w.env.cost_estimate().budget().memory_bytes_cost(),
    }
}

fn report(name: &str, input_bytes: u32, records: u32, before: Budget, after: Budget) {
    let cpu = after.cpu.saturating_sub(before.cpu);
    let memory = after.memory.saturating_sub(before.memory);
    std::println!(
        "HYPERION_BUDGET case={name} input_bytes={input_bytes} records={records} cpu={cpu} memory={memory}"
    );
}

fn request(w: &World) -> OutboundRequest {
    OutboundRequest {
        token: w.token_id.clone(),
        amount: HUNDRED,
        route: RouteKind::Cctp,
        destination: Destination {
            chain: w.ethereum(),
            address: evm_destination(&w.env, 0x11),
        },
        destination_decimals: EVM_DECIMALS,
        min_destination_amount: 0,
    }
}

fn deliver(w: &World, nonce: u64) {
    w.fund_rail(HUNDRED);
    w.rail().deliver(&Delivery {
        route: RouteKind::Cctp,
        token: w.token_id.clone(),
        amount: HUNDRED,
        recipient: w.recipient.clone(),
        recipient_kind: AddressKind::Account,
        source_chain: w.ethereum(),
        source_nonce: nonce,
    });
}

fn parked_claims(w: &World, count: u32) -> (Address, Address) {
    let token = w.with_failable_token();
    w.failable(&token)
        .mint(&w.rail_id, &(HUNDRED * count as i128));
    w.failable(&token).set_blocked(&w.recipient, &true);
    for nonce in 0..count {
        w.rail().deliver(&Delivery {
            route: RouteKind::Cctp,
            token: token.clone(),
            amount: HUNDRED,
            recipient: w.recipient.clone(),
            recipient_kind: AddressKind::Account,
            source_chain: w.ethereum(),
            source_nonce: nonce as u64,
        });
    }
    (token, Address::generate(&w.env))
}

#[test]
fn print_reproducible_quote_dispatch_inbound_and_settlement_budgets() {
    let w = World::new();
    let before = budget(&w);
    w.router()
        .quote_routes(&w.token_id, &HUNDRED, &EVM_DECIMALS);
    report("quote", 32, 0, before, budget(&w));

    let w = World::new();
    let before = budget(&w);
    w.router().bridge_out(&w.user, &request(&w));
    report("dispatch", 32, 1, before, budget(&w));

    let w = World::new();
    let before = budget(&w);
    deliver(&w, 1);
    report("inbound", 32, 1, before, budget(&w));

    let w = World::new();
    let (token, settler) = parked_claims(&w, SMALL_RECORDS);
    w.failable(&token).set_blocked(&w.recipient, &false);
    let before = budget(&w);
    w.router().settle_claim(&settler, &1);
    report("settlement", 64, SMALL_RECORDS, before, budget(&w));

    let w = World::new();
    let (token, settler) = parked_claims(&w, LARGE_RECORDS);
    w.failable(&token).set_blocked(&w.recipient, &false);
    let before = budget(&w);
    w.router().settle_claim(&settler, &1);
    report("settlement", 64, LARGE_RECORDS, before, budget(&w));
}
