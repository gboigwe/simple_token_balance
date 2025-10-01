#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod simple_token {
    use ink::storage::Mapping;

    /// Storage structure for our simple token contract
    #[ink(storage)]
    pub struct SimpleToken {
        /// Owner of the contract (can mint new tokens)
        owner: AccountId,
        /// Mapping from account to token balance (like a phone book: person -> amount)
        balances: Mapping<AccountId, u128>,
        /// Total supply of tokens
        total_supply: u128,
    }

    /// Custom error types for better error handling
    #[derive(Debug, PartialEq, Eq)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    #[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
    pub enum Error {
        /// Trying to spend more tokens than available
        InsufficientBalance,
        /// Only owner can perform this operation
        Unauthorized,
        /// Cannot transfer zero tokens
        InvalidAmount,
        /// Arithmetic overflow occurred
        Overflow,
    }

    /// Result type alias for cleaner error handling
    pub type Result<T> = core::result::Result<T, Error>;

    /// Event emitted when tokens are minted (created)
    #[ink(event)]
    pub struct Minted {
        /// Account that received the new tokens
        #[ink(topic)]
        pub to: AccountId,
        /// Amount of tokens created
        pub amount: u128,
        /// When the minting happened
        pub timestamp: u64,
    }

    /// Event emitted when tokens are transferred
    #[ink(event)]
    pub struct Transfer {
        /// Account that sent the tokens
        #[ink(topic)]
        pub from: AccountId,
        /// Account that received the tokens
        #[ink(topic)]
        pub to: AccountId,
        /// Amount of tokens transferred
        pub amount: u128,
        /// When the transfer happened
        pub timestamp: u64,
    }

    impl SimpleToken {
        /// Constructor - called once when contract is deployed
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {
                owner: Self::env().caller(),
                balances: Mapping::default(),
                total_supply: 0,
            }
        }

        /// Mint (create) new tokens - only owner can do this
        #[ink(message)]
        pub fn mint(&mut self, to: AccountId, amount: u128) -> Result<()> {
            // Validate: Only owner can mint tokens
            if self.env().caller() != self.owner {
                return Err(Error::Unauthorized);
            }

            // Validate: Cannot mint zero tokens
            if amount == 0 {
                return Err(Error::InvalidAmount);
            }

            // Get current balance of the recipient
            let current_balance = self.balances.get(to).unwrap_or(0);

            // Update the balance (add new tokens) with overflow protection
            let new_balance = current_balance
                .checked_add(amount)
                .ok_or(Error::Overflow)?;
            self.balances.insert(to, &new_balance);

            // Update total supply with overflow protection
            self.total_supply = self.total_supply
                .checked_add(amount)
                .ok_or(Error::Overflow)?;

            // Emit event for transparency
            self.env().emit_event(Minted {
                to,
                amount,
                timestamp: self.env().block_timestamp(),
            });

            Ok(())
        }

        /// Check the balance of an account
        #[ink(message)]
        pub fn balance_of(&self, account: AccountId) -> u128 {
            // Get balance, return 0 if account doesn't exist
            self.balances.get(account).unwrap_or(0)
        }

        /// Transfer tokens from caller to another account
        #[ink(message)]
        pub fn transfer(&mut self, to: AccountId, amount: u128) -> Result<()> {
            let caller = self.env().caller();

            // Validate: Cannot transfer zero tokens
            if amount == 0 {
                return Err(Error::InvalidAmount);
            }

            // Get caller's balance
            let caller_balance = self.balances.get(caller).unwrap_or(0);

            // Validate: Caller must have enough tokens
            if caller_balance < amount {
                return Err(Error::InsufficientBalance);
            }

            // Get recipient's balance
            let to_balance = self.balances.get(to).unwrap_or(0);

            // Update balances with overflow/underflow protection
            let new_caller_balance = caller_balance
                .checked_sub(amount)
                .ok_or(Error::Overflow)?;
            let new_to_balance = to_balance
                .checked_add(amount)
                .ok_or(Error::Overflow)?;

            self.balances.insert(caller, &new_caller_balance);
            self.balances.insert(to, &new_to_balance);

            // Emit event for transparency
            self.env().emit_event(Transfer {
                from: caller,
                to,
                amount,
                timestamp: self.env().block_timestamp(),
            });

            Ok(())
        }

        /// Get the total supply of tokens
        #[ink(message)]
        pub fn total_supply(&self) -> u128 {
            self.total_supply
        }

        /// Get the owner of the contract
        #[ink(message)]
        pub fn get_owner(&self) -> AccountId {
            self.owner
        }
    }
}
