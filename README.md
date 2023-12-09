# Open Stylus
<div align="center"">
    <img src="https://github.com/Prabhat1308/OpenStylus/blob/main/assets/logo.png">
  </div>

This library implements the following 
* ERC20
* ERC721
* ERC1155
* ERC4626
* ERC6909
* WETH
* ERC 4337 Meta Transactions
* FFT 
* ABDK (fixed point numbers)

## FFT 

Fast Fourier Transform (FFT) is an algorithm that efficiently computes the Discrete Fourier Transform (DFT) of a sequence or signal. It reduces the time complexity from O(n^2) to O(n log n), enabling rapid computation of frequency information from time-domain data, widely used in signal processing and various scientific fields.

Note: Gas calculation for Rust Code is done by calling the rust contract via a solidity contract and then testing on foundry . Real gas cost of Rust Code may vary.

| Gas description | Rust Code| Solidity Code |
| -------- | -------- | -------- |
| Deployment cost   | 308972     | 767482  |
 Min Cost |28317 | 26411
Max Cost |28317 |58572
Median Cost |28317 | 42491
Average Cost |28317 | 42491

### Solidity Code
![solidity_code](https://github.com/Prabhat1308/OpenStylus/blob/main/assets/solidity_code.png)

### Rust Code
![rust_code](https://github.com/Prabhat1308/OpenStylus/blob/main/assets/rust_code.png)

## ABDK Maths Library
ABDK Math is a Solidity library providing advanced mathematical functions. It offers precise fixed-point arithmetic for Ethereum contracts. This library helps manage decimals and perform complex math operations accurately in smart contracts. It's used to avoid rounding errors and ensure reliable calculations in decentralized applications (dApps) on the Ethereum blockchain.Generally used for 64x64 FIXED point number calculations.

Functions implemented in ABDK library 
* i256 -> i128
* i128 -> i64 
* u256 -> i128
* i128 -> u64
* signed 128.128 fixed point number into signed 64.64-bit fixed point
* signed 64.64 fixed point number into signed 128.128 fixed point
* addition
* subtraction
* multiplication
* division
* negation
* absolute
* inverse
* average
* geometric mean
* power function 
* square root

## Meta Transactions using ERC 4337
Implementation of contracts for ERC-4337 account abstraction via alternative mempool.
Meta transactions using ERC-4337 allow users to perform transactions on behalf of others, paying fees in native tokens. This standard enables relayers to execute transactions, abstracting gas costs for users. By utilizing off-chain signatures, it promotes gasless interactions, enhancing usability and accessibility of decentralized applications.(This contract is currently under development and completed features are base account , noncce manager , stake manager , user operation and sender creator . Entrypoint and Paymaster are yet to be implemented).

##  WETH , ERC20 and ERC721
These contracts have been directly imported from Arbitrum example repos.

## ERC 1155
ERC-1155 is a token standard on the Ethereum blockchain designed to efficiently manage different types of tokens within a single smart contract. Unlike previous standards, it allows for the creation of both fungible (identical and interchangeable) and non-fungible (unique and distinct) tokens within one contract, reducing gas fees and optimizing storage. This versatility makes it particularly suitable for gaming, where various in-game items, currencies, or collectibles can exist within a single framework, streamlining transactions and interactions on the blockchain. (built from scratch)

## ERC 4626
ERC-4626 is a standard to optimize and unify the technical parameters of yield-bearing vaults. It provides a standard API for tokenized yield-bearing vaults that represent shares of a single underlying ERC-20 token. ERC-4626 also outlines an optional extension for tokenized vaults utilizing ERC-20, offering basic functionality for depositing, withdrawing tokens and reading balances. (built from scratch)

## ERC 6909 
ERC-6909 is a specification for managing multiple tokens by their id in a single contract. 
