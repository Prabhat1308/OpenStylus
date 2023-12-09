use alloc::{string::String, vec::Vec};
use core::marker::PhantomData;
use stylus_sdk::{
    alloy_primitives::{Address, U256},
    alloy_sol_types::{sol, SolError},
    evm, msg,
    prelude::*,
};


sol_storage! {
    /// Erc20 implements all ERC-20 methods.
    pub struct Erc6909 {
        /// Maps users to balances
        mapping(address => uint256) balances;
        /// Maps users to a mapping of each spender's allowance
        mapping(address => mapping(address => uint256)) allowances;
        /// The total supply of the token
        uint256 total_supply;

        mapping(address => mapping(address => bool)) public isOperator;

        mapping(address => mapping(uint256 => uint256)) public balanceOf;

        mapping(address => mapping(address => mapping(uint256 => uint256))) public allowance;
    }
}

// Declare events and Solidity error types
sol! {
    event Transfer(address indexed from, address indexed to, uint256 value);
    event OperatorSet(address indexed owner, address indexed operator, bool approved);
    event Approval(address indexed owner, address indexed spender, uint256 value);
    event Approval(address indexed owner, address indexed spender, uint256 indexed id, uint256 amount);
    error InsufficientBalance(address from, uint256 have, uint256 want);
    error InsufficientAllowance(address owner, address spender, uint256 have, uint256 want);
    event Transfer(address caller, address indexed from, address indexed to, uint256 indexed id, uint256 amount);
}


impl Erc6909 {

  pub fn mint(&mut self, receiver: Address, id: U256, amount: U256) -> Result<bool, Vec<u8>> {
        initial_amount = self.balanceOf[receiver][id];
        final_amount = initial_amount + amount;
        self.balanceOf[receiver][id] = final_amount;
        evm::log(Transfer(msg::sender, msg::sender, receiver, id, amount));
        return true;
    }

    pub fn burn(&mut self, owner: Address, id: U256, amount: U256) -> Result<bool, Vec<u8>> {
        initial_amount = self.balanceOf[owner][id];
        final_amount = initial_amount - amount;
        self.balanceOf[owner][id] = final_amount;
        evm::log(Transfer(msg::sender, owner, msg::sender, id, amount));
        return true;
    }
}

// These methods are external to other contracts
// Note: modifying storage will become much prettier soon
#[external]
impl Erc6909 {

    pub fn transfer(
        &mut self,
        receiver: Address,
        id: U256,
        amount: U256,
    ) -> Result<bool, Vec<u8>> {
        initial_amount = self.balanceOf[msg::sender][id];
        final_amount = initial_amount - amount;
        self.balanceOf[msg::sender][id] = final_amount;
        evm::log(Transfer(msg::sender, msg::sender, receiver, id, amount));
        return true;
    }

    pub fn transfer_from(
        &mut self,
        from: Address,
        to: Address,
        id: U256,
        value: U256,
    ) -> Result<bool, Erc20Error> {
       sender_balance_initial = self.balanceOf[from][id];
       receiver_balance_initial = self.balanceOf[to][id];
       sender_balance_final = sender_balance_initial - value;
       receiver_balance_final = receiver_balance_initial + value;

        evm::log(Transfer(msg::sender, from, to, id, value));
    }

    pub fn approve(
        &mut self,
        spender: Address,
        id: U256,
        amount: U256,
    ) -> Result<bool, Vec<u8>> {
        self.allowance[msg::sender][spender][id] = amount;
        evm::log(Approval(msg::sender, spender, id, amount));
        return true;
    }

    pub fn setOperator(&mut self, operator: Address, approved: bool) -> Result<bool, Erc20Error> {
        self.isOperator[msg::sender][operator] = approved;
        evm::log(OperatorSet(msg::sender, operator, approved));
        return true;
    }

    fn supports_interface(interface_id: u32) -> bool {
        interface_id == 0x01ffc9a7 || // ERC165 Interface ID for ERC165
        interface_id == 0x0f632fb3   // ERC165 Interface ID for ERC6909
    }
    
}