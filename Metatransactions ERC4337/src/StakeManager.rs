#![no_main]
#![no_std]

/**
 * manage deposits and stakes.
 * deposit is just a balance used to pay for UserOperations (either by a paymaster or an account)
 * stake is value locked for at least "unstakeDelay" by a paymaster.
 */
extern crate alloc;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

use alloc::{ string::String, vec::Vec };
use core::marker::PhantomData;
use stylus_sdk::{
    alloy_primitives::{ Address, U256, U64 },
    alloy_sol_types::{ sol, SolError },
    evm,
    msg,
    prelude::*,
    stylus_proc::entrypoint,
};

sol_interface! {
     /**
  * manage deposits and stakes.
  * deposit is just a balance used to pay for UserOperations (either by a paymaster or an account)
  * stake is value locked for at least "unstakeDelay" by the staked entity.
  */
 interface IStakeManager {
 
     event Deposited(
         address indexed account,
         uint256 totalDeposit
     );
 
     event Withdrawn(
         address indexed account,
         address withdrawAddress,
         uint256 amount
     );
 
     /// Emitted when stake or unstake delay are modified
     event StakeLocked(
         address indexed account,
         uint256 totalStaked,
         uint256 unstakeDelaySec
     );
 
     /// Emitted once a stake is scheduled for withdrawal
     event StakeUnlocked(
         address indexed account,
         uint256 withdrawTime
     );
 
     event StakeWithdrawn(
         address indexed account,
         address withdrawAddress,
         uint256 amount
     );
 
     /**
      * @param deposit the entity's deposit
      * @param staked true if this entity is staked.
      * @param stake actual amount of ether staked for this entity.
      * @param unstakeDelaySec minimum delay to withdraw the stake.
      * @param withdrawTime - first block timestamp where 'withdrawStake' will be callable, or zero if already locked
      * @dev sizes were chosen so that (deposit,staked, stake) fit into one cell (used during handleOps)
      *    and the rest fit into a 2nd cell.
      *    112 bit allows for 10^15 eth
      *    48 bit for full timestamp
      *    32 bit allows 150 years for unstake delay
      */
     struct DepositInfo {
         uint112 deposit;
         bool staked;
         uint112 stake;
         uint32 unstakeDelaySec;
         uint48 withdrawTime;
     }
 
     //API struct used by getStakeInfo and simulateValidation
     struct StakeInfo {
         uint256 stake;
         uint256 unstakeDelaySec;
     }
 
     /// @return info - full deposit information of given account
     function getDepositInfo(address account) external view returns (DepositInfo memory info);
 
     /// @return the deposit (for gas payment) of the account
     function balanceOf(address account) external view returns (uint256);
 
     /**
      * add to the deposit of the given account
      */
     function depositTo(address account) external payable;
 
     /**
      * add to the account's stake - amount and delay
      * any pending unstake is first cancelled.
      * @param _unstakeDelaySec the new lock duration before the deposit can be withdrawn.
      */
     function addStake(uint32 _unstakeDelaySec) external payable;
 
     /**
      * attempt to unlock the stake.
      * the value can be withdrawn (using withdrawStake) after the unstake delay.
      */
     function unlockStake() external;
 
     /**
      * withdraw from the (unlocked) stake.
      * must first call unlockStake and wait for the unstakeDelay to pass
      * @param withdrawAddress the address to send withdrawn value.
      */
     function withdrawStake(address payable withdrawAddress) external;
 
     /**
      * withdraw from the deposit.
      * @param withdrawAddress the address to send withdrawn value.
      * @param withdrawAmount the amount to withdraw.
      */
     function withdrawTo(address payable withdrawAddress, uint256 withdrawAmount) external;
 }
 
 }

sol_storage! {
    #[entrypoint]
     pub struct StakeManager {
     /// maps paymaster to their deposits and stakes
     mapping(address => DepositInfo) public deposits;
     }
 }

impl StakeManager {
    pub fn get_stake_info(&mut self, account: Address) -> Result<StakeInfo, Vec<u8>> {
        let stake_info = self.stakes[account];
        Ok(stake_info)
    }

    pub fn increment_deposit(&mut self, account: Address, amount: U256) -> Result<(), Vec<u8>> {
        self.deposits[account].deposit += amount;
        Ok(())
    }
}

#[external]
impl StakeManager {
    pub fn get_deposit_info(&mut self, account: Address) -> Result<DepositInfo, Vec<u8>> {
        let deposit_info = self.deposits[account];
        Ok(deposit_info)
    }

    pub fn get_balance_of(&mut self, account: Address) -> Result<U256, Vec<u8>> {
        let balance = self.deposits[account].deposit;
        Ok(balance)
    }

    // receive() external payable {
    //     depositTo(msg.sender);
    // }

    /**
     * add to the deposit of the given account
     */
    pub fn deposit_to(&mut self, account: Address) -> Result<(), Vec<u8>> {
        self.deposits[account].deposit += msg::value();
        evm::log(Deposited {
            account,
            total_deposit: self.deposits[account].deposit,
        });
        Ok(())
    }

    /**
     * add to the account's stake - amount and delay
     * any pending unstake is first cancelled.
     * @param unstakeDelaySec the new lock duration before the deposit can be withdrawn.
     */
    pub fn add_stake(&mut self, account: Address, unstake_delay_sec: U64) -> Result<(), Vec<u8>> {
        let deposit_info = self.deposits[account];
        let stake_info = self.stakes[account];
        let stake = stake_info.stake;
        let unstake_delay_sec = stake_info.unstake_delay_sec;
        let withdraw_time = stake_info.withdraw_time;

        if stake_info.stake > 0 {
            self.withdraw_stake(account);
        }

        self.stakes[account] = StakeInfo {
            stake: stake + msg::value(),
            unstake_delay_sec,
            withdraw_time,
        };

        evm::log(StakeLocked {
            account,
            total_staked: stake + msg::value(),
            unstake_delay_sec,
        });
        Ok(())
    }

    /**
     * attempt to unlock the stake.
     * the value can be withdrawn (using withdrawStake) after the unstake delay.
     */

     pub fn unlock_stake(&mut self, account: Address) -> Result<(), Vec<u8>> {
        let deposit_info = self.deposits[account];
        let stake_info = self.stakes[account];
        let stake = stake_info.stake;
        let unstake_delay_sec = stake_info.unstake_delay_sec;
        let withdraw_time = stake_info.withdraw_time;

        if stake_info.stake == 0 {
            return Err(StakeManagerError::NoStake.into());
        }

        if stake_info.withdraw_time > 0 {
            return Err(StakeManagerError::StakeLocked.into());
        }

        self.stakes[account] = StakeInfo {
            stake,
            unstake_delay_sec,
            withdraw_time: block::timestamp() + unstake_delay_sec,
        };

        evm::log(StakeUnlocked {
            account,
            withdraw_time: block::timestamp() + unstake_delay_sec,
        });
        Ok(())
    }

    /**
     * withdraw from the (unlocked) stake.
     * must first call unlockStake and wait for the unstakeDelay to pass
     * @param withdrawAddress the address to send withdrawn value.
     */

     pub fn withdraw_stake(&mut self, account: Address, withdraw_address: Address) -> Result<(), Vec<u8>> {
        let deposit_info = self.deposits[account];
        let stake_info = self.stakes[account];
        let stake = stake_info.stake;
        let unstake_delay_sec = stake_info.unstake_delay_sec;
        let withdraw_time = stake_info.withdraw_time;

        if stake_info.stake == 0 {
            return Err(StakeManagerError::NoStake.into());
        }

        if stake_info.withdraw_time == 0 {
            return Err(StakeManagerError::StakeLocked.into());
        }

        if stake_info.withdraw_time > block::timestamp() {
            return Err(StakeManagerError::StakeLocked.into());
        }

        self.stakes[account] = StakeInfo {
            stake: 0,
            unstake_delay_sec,
            withdraw_time: 0,
        };

        evm::log(StakeWithdrawn {
            account,
            withdraw_address,
            amount: stake,
        });
        Ok(())
    }

    pub fn withdraw_to(&mut self, account: Address, withdraw_address: Address, withdraw_amount: U256) -> Result<(), Vec<u8>> {
        let deposit_info = self.deposits[account];
        let stake_info = self.stakes[account];
        let stake = stake_info.stake;
        let unstake_delay_sec = stake_info.unstake_delay_sec;
        let withdraw_time = stake_info.withdraw_time;

        if stake_info.stake == 0 {
            return Err(StakeManagerError::NoStake.into());
        }

        if stake_info.withdraw_time > 0 {
            return Err(StakeManagerError::StakeLocked.into());
        }

        if stake_info.stake < withdraw_amount {
            return Err(StakeManagerError::InsufficientStake.into());
        }

        self.stakes[account] = StakeInfo {
            stake: stake - withdraw_amount,
            unstake_delay_sec,
            withdraw_time,
        };

        evm::log(Withdrawn {
            account,
            withdraw_address,
            amount: withdraw_amount,
        });
        Ok(())
    }
}