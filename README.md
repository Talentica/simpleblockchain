# simpleblockchain
Simplified version of blockchain

## Basic Workflow


              
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
