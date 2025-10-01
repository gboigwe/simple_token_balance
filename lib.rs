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
    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        /// Trying to spend more tokens than available
        InsufficientBalance,
        /// Only owner can perform this operation
        Unauthorized,
        /// Cannot transfer zero tokens
        InvalidAmount,
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

            // Update the balance (add new tokens)
            self.balances.insert(to, &(current_balance + amount));

            // Update total supply
            self.total_supply += amount;

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

            // Update balances (subtract from caller, add to recipient)
            self.balances.insert(caller, &(caller_balance - amount));
            self.balances.insert(to, &(to_balance + amount));

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