# Vesting Partial Claim Tracking

## Overview
This document describes the mechanism for tracking partial claims in the vesting contract to ensure consistency and prevent math breakage.

## Partial Claim Ledger
Each beneficiary has a ledger of partial claims stored as a vector of `PartialClaim` structs.

### PartialClaim Struct
- amount: i128 - The amount claimed in this partial claim
- timestamp: u64 - The timestamp of the claim

### Cursor/Tracking
- The total claimed amount is tracked in the `VestingSchedule.claimed` field.
- Each partial claim is appended to the claims vector for auditability.
- Claims are validated against the vested amount at the time of claim to prevent over-claiming.

## Invariants
- Total claimed <= vested amount at any time
- Sum of partial claims == total claimed
- No negative claims allowed
- Claims only increase total claimed

## Security Notes
- Partial claims prevent large single transactions that could be frontrun or cause liquidity issues.
- Tracking ensures no "dust loss" and maintains investor trust.
- Events are emitted for each claim with schema version 1.0.