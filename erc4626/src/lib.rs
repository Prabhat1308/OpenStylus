#![cfg_attr(not(feature = "export-abi"), no_main, no_std)]
extern crate alloc;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

use crate::erc20::{Erc20, Erc20Params};
use alloc::vec::Vec;
use stylus_sdk::{alloy_primitives::{U256, Address}, call, call::RawCall, msg, function_selector, prelude::*};
mod erc20;
use alloc::vec;

use rustlearn::prelude::*;
use rustlearn::trees::decision_tree;

struct VaultParams;

impl Erc20Params for VaultParams {
    const NAME: &'static str = "Vault Example";
    const SYMBOL: &'static str = "VAULT";
    const DECIMALS: u8 = 18;
}

sol_storage! {
    #[entrypoint]
    struct Vault {
        address asset;
        uint totalSupply;

        #[borrow] 
        Erc20<VaultParams> erc20;
    }
}

#[external]
#[inherit(Erc20<VaultParams>)]
impl Vault {

    pub fn setAsset(&mut self, _asset: Address) -> Result<Address, Vec<u8>> {
        self.asset.set(_asset);
        Ok(_asset)
    }

    #[payable]
    pub fn deposit(&mut self, amount: U256) -> Result<(), Vec<u8>> {
        let selector = function_selector!("transferFrom(address,address,uint256)");
        let data = [
            &selector[..],
            &msg::sender().into_array(),
            &self.asset.get().into_array(),
            &amount.to_be_bytes::<32>(),
        ].concat();
        RawCall::new().call(self.asset.get(), &data);

        let supply = self.totalSupply.get();
        let shares = if supply == U256::ZERO { 
            amount 
        } else { 
            amount.checked_mul(supply).ok_or("Overflow")?.checked_div(self.totalAssets()?).ok_or("Divide by zero")?
        };
        self.erc20.mint(msg::sender(), amount);

        self.afterDeposit(amount);

        Ok(())
    }

    pub fn withdraw(&mut self, amount: U256) -> Result<(), Vec<u8>> {
        self.beforeWithdraw(amount);

        let supply = self.totalSupply.get();
        let shares = if supply == U256::ZERO { 
            amount 
        } else { 
            amount.checked_mul(supply).ok_or("Overflow")?.checked_div(self.totalAssets()?).ok_or("Divide by zero")?
        };
    
        self.erc20.burn(msg::sender(), amount)?;
        call::transfer_eth(msg::sender(), amount)
    }

    pub fn asset(&self) -> Result<Address, Vec<u8>> {
        Ok(self.asset.get())
    }

    fn afterDeposit(&mut self, assets: U256) -> Result<(), Vec<u8>> {
        // Add your logic here
        Ok(())
    }

    fn beforeWithdraw(&mut self, amount: U256)-> Result<(), Vec<u8>> {
        // Add your logic here
        Ok(())
    }

    pub fn totalAssets(&self) -> Result<U256, Vec<u8>> {
        Ok(self.totalSupply.get())
    }


    // This function will take an array of datafeeds and an array of strategies, 
    // and use machine learning to determine the best strategy.
    pub fn rebalance(datafeeds: Vec<Vec<u32>>, strategies: Vec<i32>) -> Result<i32, Vec<u8>> {
        // Convert the datafeeds to f32 and flatten into a single Vec<f32>
        let flattened_datafeeds: Vec<f32> = datafeeds.clone().into_iter().flatten().map(|x| x as f32).collect();
        let flattened_strategies: Vec<f32> = strategies.into_iter().map(|x| x as f32).collect();
    
        // Convert the flattened datafeeds and strategies to a rustlearn dataset
        let x = Array::from(flattened_datafeeds);
        let y = Array::from(flattened_strategies);
    
        // Create a decision tree model
        let mut model = decision_tree::Hyperparameters::new(x.cols())
            .one_vs_rest();
    
        // Train the model
        match model.fit(&x, &y) {
            Ok(_) => {
                // Use the model to predict the best strategy for the next datafeed
                let next_datafeed: Vec<f32> = datafeeds.last().unwrap().clone().into_iter().map(|x| x as f32).collect();
                let prediction = model.predict(&Array::from(next_datafeed)).unwrap();
    
                // Return the first prediction as an i32
                Ok(prediction.data()[0] as i32)
            },
            Err(_) => Err("Failed to train model".into())
        }
    }
}