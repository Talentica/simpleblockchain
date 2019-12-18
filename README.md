# simpleblockchain
Simplified version of blockchain

## Basic Workflow

### Transaction Flow

![Alt text](./docs/TransactionFlow.png?raw=true "Transaction Flow in a blockchain")

              
              Client        Blockchain Node        Blockchain Peer Node
                   +            +                        +
                   | Submit     |                        |
                   | Transaction|                        |
                   |+---------> |                        |
                   |            |                        |
                   |            |                        |
                   |            |Validate Txn            |
                   |            |                        |
                   v            |                        |
                                |                        |
                                |                        |
                                |Send Valid Txn          |
                                |to peers                |Validate Txn <----+
                                | +------------------>   |                  |
                                |                        |                  |
                                |                        |                  |
                                |                        |                  |
                                |Add Txn to Txn          |Send Txn to Peers +
                                |Pool                    |
                                |                        |
                                |                        |
                                |                        |
                                |                        |Add Txn to Txn Pool
                                |                        |
                                |                        |
                                |                        |
                                |                        |
                                v                        v

### Block creation - Consensus
