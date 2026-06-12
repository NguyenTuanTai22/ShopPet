# SplitPay Smart Contract

## Overview

SplitPay is a smart contract built on Soroban (Stellar Smart Contracts) that automatically distributes incoming token payments among multiple recipients according to predefined percentage shares.

The contract is designed for teams, business partnerships, content creators, freelancers, and organizations that need transparent and automatic revenue sharing without relying on a trusted intermediary.

---

## Problem Statement

In traditional payment systems, when revenue needs to be shared among multiple participants, one person or organization must manually calculate and distribute funds.

This process introduces several challenges:

* Human errors in calculations.
* Delays in payment distribution.
* Lack of transparency.
* Potential disputes between participants.
* Additional administrative costs.

For example:

* A startup with multiple founders sharing profits.
* A music group splitting streaming revenue.
* A freelance team dividing project payments.
* An online platform sharing commissions among contributors.

A trustless and automated revenue-sharing mechanism is needed.

---

## Proposed Solution

SplitPay solves this problem by automatically distributing deposited tokens according to predefined ownership percentages.

When a payment is deposited:

1. The contract receives the tokens.
2. The contract calculates each member's share.
3. Tokens are immediately transferred to recipients.
4. Any rounding remainder is recorded.
5. Statistics are updated on-chain.

All operations are transparent, deterministic, and auditable through the blockchain.

---

## Key Features

### Automatic Revenue Distribution

Automatically distributes deposited tokens among shareholders.

### Configurable Shareholders

The administrator can update shareholder allocations when business requirements change.

### Revenue Statistics

Tracks:

* Total funds received.
* Total funds distributed.
* Remaining undistributed balance.

### Remainder Management

Handles integer division rounding errors by storing remainders and allowing administrators to claim them later.

### Emergency Pause

Administrators can pause contract operations during emergencies or maintenance.

### Event Logging

Important actions generate blockchain events:

* Contract initialization.
* Revenue distribution.
* Shareholder updates.

---

## Architecture

### Core Components

#### Shareholder

Represents a revenue recipient.

```rust
pub struct Shareholder {
    pub account: Address,
    pub share: u32,
}
```

Where:

* `account` = recipient wallet address.
* `share` = ownership percentage (10000 = 100%).

---

#### Contract Storage

The contract stores the following data:

| Key              | Description            |
| ---------------- | ---------------------- |
| Admin            | Contract administrator |
| Shareholders     | List of recipients     |
| TokenAddress     | Accepted token         |
| TotalReceived    | Total deposits         |
| TotalDistributed | Total payouts          |
| Remainder        | Unallocated balance    |
| IsPaused         | Contract status        |

---

## Workflow

### Initialization

The administrator initializes the contract with:

* Admin address
* Token contract address
* Shareholder list

Requirements:

* At least one shareholder.
* Maximum 20 shareholders.
* Total shares must equal 10,000.

---

### Deposit and Split

User deposits tokens.

The contract:

1. Transfers tokens into the contract.
2. Calculates allocations.
3. Sends tokens to each shareholder.
4. Records any remainder.
5. Updates statistics.

Example:

| Member | Share |
| ------ | ----- |
| Alice  | 60%   |
| Bob    | 40%   |

Deposit:

```text
1000 Tokens
```

Result:

```text
Alice → 600
Bob → 400
```

---

### Share Update

The administrator can modify the shareholder structure.

Validation rules:

* Total shares must equal 10,000.
* Maximum 20 members.

---

### Claim Remainder

Due to integer division, tiny amounts may remain undistributed.

The administrator can withdraw accumulated remainder funds.

---

### Pause Contract

The administrator may temporarily disable deposits and distributions.

Useful for:

* Security incidents.
* Contract upgrades.
* Operational maintenance.

---

## Functions

### initialize()

Creates and configures the contract.

### deposit_and_split()

Accepts tokens and distributes them according to ownership shares.

### update_shares()

Updates shareholder allocations.

### claim_remainder()

Withdraws accumulated remainder funds.

### set_paused()

Enables or disables contract activity.

### get_shareholders()

Returns the shareholder list.

### get_stats()

Returns:

```text
(total_received,
 total_distributed,
 remainder)
```

---

## Security Considerations

The contract implements several protections:

* Authentication checks using `require_auth()`.
* Share validation (must equal 100%).
* Overflow protection using checked arithmetic.
* Emergency pause functionality.
* Controlled administrative operations.

---

## Use Cases

* Startup founder profit sharing.
* DAO treasury distributions.
* Affiliate commission payments.
* Creator revenue sharing.
* Freelance team payouts.
* Partnership profit allocation.

---

## Future Improvements

Potential enhancements include:

* Multi-token support.
* Scheduled distributions.
* Time-locked payouts.
* Governance-based shareholder updates.
* Automated remainder redistribution.
* Role-based access control.

---

## Technology Stack

* Rust
* Soroban SDK
* Stellar Network
* Freighter Wallet

---

## Conclusion

SplitPay demonstrates how smart contracts can automate revenue sharing in a transparent and trustless manner. By eliminating manual calculations and reducing operational overhead, the contract provides a secure and efficient solution for collaborative financial management on the Stellar ecosystem.
