## A simple Asset Swap smart contract

This smart contract is designed to be used in a peer-to-peer exchange scenario where two parties agree to exchange assets. The contract ensures that the assets are locked up until it is accepted by the other party. At any point before it is accepted, one can cancel the swap to retrieve the assets.

This contract is just for academic purposes because of its limitations. It assumes that both parties are dealing with the same asset type. It lacks the ability to handle multiple asset types or interact with token standards (e.g., ERC-20 or PSP-22 for Polkadot).

### Key Features

| Feature | Description |
|---------|-------------|
| Atomic Swaps | Either both parties get their assets or the swap is canceled |
| Authorization Checks | Strict permission controls for swap operations |
| Reentrancy Protection | Prevents recursive call attacks |
| Event Logging | Track all swap lifecycle changes |
| Detailed Error Codes | Clear failure reasons for debugging |
| Storage Efficiency | Optimized mapping structure for swap storage |

### Data Structures

| Structure | Fields | Description |
|-----------|--------|-------------|
| `Swap` | initiator: AccountId<br>counterparty: AccountId<br>initiator_asset: Balance<br>counterparty_asset: Balance | Represents a single swap agreement |
| `SwapContract` | swaps: Mapping<u32, Swap><br>next_swap_id: u32<br>reentrancy_guard: bool | Main contract storage with security features |

### Functions Overview

#### `initiate_swap()` - Creates new swap agreement

- **Key Points:**
  - Validate initiator's deposit > 0
  - Generate incremental swap ID
  - Store terms in mapping
  - Emit SwapInitiated event

#### `accept_swap()` - Execute asset exchange

- **Key Points:**
  - Activate reentrancy guard
  - Verify counterparty identity
  - Validate transferred amount
  - Transfer assets atomically
  - Clean storage & emit event

#### `cancel_swap()` - Abort swap and refund

- **Key Points:**
  - Verify initiator identity
  - Return locked funds
  - Remove swap record
  - Emit cancellation event

### State Diagram

```mermaid
stateDiagram
    [*] --> Idle
    Idle --> Initiated: initiate_swap()
    Initiated --> Accepted: accept_swap()
    Initiated --> Canceled: cancel_swap()
    Accepted --> [*]
    Canceled --> [*]
```

### Sequence Diagram

```mermaid
sequenceDiagram
    participant I as Initiator
    participant C as Contract
    participant R as Counterparty

    I->>C: initiate_swap(counterparty, amount)
    C-->>I: swap_id
    
    R->>C: accept_swap(swap_id) with funds
    C->>I: Transfer counterparty_asset
    C->>R: Transfer initiator_asset
    C-->>R: SwapAccepted event
    
    alt Cancel
    I->>C: cancel_swap(swap_id)
    C->>I: Return initiator_asset
    C-->>I: SwapCancelled event
    end
```