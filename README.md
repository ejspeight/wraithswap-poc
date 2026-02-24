# WraithSwap PoC

Proof of concept for WraithSwap. Validating that eigenwallet/core's ASB can be used to facilitate trustless BTC to XMR atomic swaps.

## Why This Exists

Atomic swaps between BTC and XMR are hard. Bitcoin and Monero use fundamentally different cryptography, so there's no simple hash-lock approach like you'd use with two EVM chains. The swap protocol requires multiple on-chain transactions, timelocks, and cryptographic proofs on both sides.

eigenwallet/core (fork of comit-network/xmr-btc-swap) already solves this with their ASB (Automated Swap Backend). Rather than rebuilding the swap protocol from scratch, this PoC validates that we can wrap their ASB and read swap state from its SQLite database. If this works, the ASB becomes the swap engine inside WraithSwap.

## What Success Looks Like

- Swap testnet BTC for stagenet XMR using the ASB
- Track swap state changes through each confirmation step
- Verify the XMR lands in a stagenet wallet

## What This Proves

- [x] ASB can be run on testnet
- [x] We can read swap state from ASB's SQLite database
- [x] Real-time monitoring of swap state changes works
- [x] Integration approach is viable
