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


## 
