# Simpleblockchain
A framework to build blockchain applications.
(Please refer to Wiki for details)

## Build
To build the docker image, use these below steps
  * Clone the repository
  * Run from shell ./build.sh
  * The build will generate 3 docker images simplebc_buildbase, simplebc_build and simplebc
  * The build will also generate a wallet_app_client binary which can be used to test the wallet functionality of the blockchain.

## Setting up the test network
We will deploy a three node network where one node is in validator configuration with POA consensus and other 2 in observation mode. 
After the docker images are generated using build script,
  * Go to tests/docker folder and verify the configuration settings. 
    - Make sure the flag `genesis_block` is true for the validator node.
    - Make sure the user developed apps are copied to ./config*/app folder properly and path for the same is correctly mentioned using `client_apps`.
    - Ports exposed by all the docker nodes are correct.
  * Set RUST_BACKTRACE=1 in the docker environment for debugging.
  * Run `docker-compose up` to bring up all the nodes.
  * use the `wallet_app_client` to mint the coins, query the blockchain state and submit a transaction.
  

## Basic Transaction Flow

![Alt text](./misc/BlockchainTxnFlowDiagram.jpg?raw=true "Transaction Flow in a blockchain")

