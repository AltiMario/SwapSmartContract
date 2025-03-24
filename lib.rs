#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod swap_contract {
    use ink::storage::Mapping;

    /// Represents a swap between two parties
    #[derive(scale::Decode, scale::Encode)]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
    )]
    pub struct Swap {
        initiator: AccountId,
        counterparty: AccountId,
        initiator_asset: Balance,
        counterparty_asset: Balance,
    }

    /// Defines the storage of the contract
    #[ink(storage)]
    pub struct SwapContract {
        /// Mapping from swap ID to Swap struct
        swaps: Mapping<u32, Swap>,
        /// Counter for generating unique swap IDs
        next_swap_id: u32,
    }

    /// Custom errors for the swap contract
    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        SwapNotFound,
        NotAuthorized,
        InsufficientBalance,
    }

    /// Event emitted when a new swap is initiated
    #[ink(event)]
    pub struct SwapInitiated {
        #[ink(topic)]
        swap_id: u32,
        initiator: AccountId,
        counterparty: AccountId,
        initiator_asset: Balance,
        counterparty_asset: Balance,
    }

    /// Event emitted when a swap is accepted
    #[ink(event)]
    pub struct SwapAccepted {
        #[ink(topic)]
        swap_id: u32,
    }

    /// Event emitted when a swap is cancelled
    #[ink(event)]
    pub struct SwapCancelled {
        #[ink(topic)]
        swap_id: u32,
    }

    impl SwapContract {
        /// Constructor to initialize the contract
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {
                swaps: Mapping::default(),
                next_swap_id: 0,
            }
        }

        /// Initiates a new swap
        #[ink(message, payable)]
        pub fn initiate_swap(&mut self, counterparty: AccountId, counterparty_asset: Balance) -> Result<u32, Error> {
            let initiator = self.env().caller();
            let initiator_asset = self.env().transferred_value();

            // Ensure the initiator has sufficient balance
            if self.env().balance() < initiator_asset {
                return Err(Error::InsufficientBalance);
            }

            let swap_id = self.next_swap_id;
            self.next_swap_id += 1;

            let swap = Swap {
                initiator,
                counterparty,
                initiator_asset,
                counterparty_asset,
            };

            self.swaps.insert(swap_id, &swap);

            // Emit the SwapInitiated event
            self.env().emit_event(SwapInitiated {
                swap_id,
                initiator,
                counterparty,
                initiator_asset,
                counterparty_asset,
            });

            Ok(swap_id)
        }

        /// Accepts an existing swap
        #[ink(message, payable)]
        pub fn accept_swap(&mut self, swap_id: u32) -> Result<(), Error> {
            let swap = self.swaps.get(swap_id).ok_or(Error::SwapNotFound)?;
            let caller = self.env().caller();

            // Ensure the caller is the counterparty
            if caller != swap.counterparty {
                return Err(Error::NotAuthorized);
            }

            // Ensure the counterparty has transferred the correct amount
            if self.env().transferred_value() != swap.counterparty_asset {
                return Err(Error::InsufficientBalance);
            }

            // Transfer assets to both parties
            self.env().transfer(swap.initiator, swap.counterparty_asset).map_err(|_| Error::InsufficientBalance)?;
            self.env().transfer(swap.counterparty, swap.initiator_asset).map_err(|_| Error::InsufficientBalance)?;

            // Remove the swap from storage
            self.swaps.remove(swap_id);

            // Emit the SwapAccepted event
            self.env().emit_event(SwapAccepted { swap_id });

            Ok(())
        }

        /// Cancels an existing swap
        #[ink(message)]
        pub fn cancel_swap(&mut self, swap_id: u32) -> Result<(), Error> {
            let swap = self.swaps.get(swap_id).ok_or(Error::SwapNotFound)?;
            let caller = self.env().caller();

            // Ensure the caller is the initiator
            if caller != swap.initiator {
                return Err(Error::NotAuthorized);
            }

            // Return the assets to the initiator
            self.env().transfer(swap.initiator, swap.initiator_asset).map_err(|_| Error::InsufficientBalance)?;

            // Remove the swap from storage
            self.swaps.remove(swap_id);

            // Emit the SwapCancelled event
            self.env().emit_event(SwapCancelled { swap_id });

            Ok(())
        }
    }
    #[cfg(test)]
    mod tests {
        use super::*;
        use ink::env::test;

        #[ink::test]
        fn test_initiate_swap() {
            let mut contract = SwapContract::new();
            let accounts = test::default_accounts::<ink::env::DefaultEnvironment>();

            // Set the caller to the initiator
            test::set_caller::<ink::env::DefaultEnvironment>(accounts.alice);
            // Set the transferred value
            test::set_value_transferred::<ink::env::DefaultEnvironment>(100);

            let result = contract.initiate_swap(accounts.bob, 200);
            assert!(result.is_ok());
            let swap_id = result.unwrap();
            assert_eq!(swap_id, 0);

            let swap = contract.swaps.get(swap_id).unwrap();
            assert_eq!(swap.initiator, accounts.alice);
            assert_eq!(swap.counterparty, accounts.bob);
            assert_eq!(swap.initiator_asset, 100);
            assert_eq!(swap.counterparty_asset, 200);
        }

        #[ink::test]
        fn test_accept_swap() {
            let mut contract = SwapContract::new();
            let accounts = test::default_accounts::<ink::env::DefaultEnvironment>();

            // Set the caller to the initiator
            test::set_caller::<ink::env::DefaultEnvironment>(accounts.alice);
            // Set the transferred value
            test::set_value_transferred::<ink::env::DefaultEnvironment>(100);

            let swap_id = contract.initiate_swap(accounts.bob, 200).unwrap();

            // Set the caller to the counterparty
            test::set_caller::<ink::env::DefaultEnvironment>(accounts.bob);
            // Set the transferred value
            test::set_value_transferred::<ink::env::DefaultEnvironment>(200);

            let result = contract.accept_swap(swap_id);
            assert!(result.is_ok());
        }

        #[ink::test]
        fn test_cancel_swap() {
            let mut contract = SwapContract::new();
            let accounts = test::default_accounts::<ink::env::DefaultEnvironment>();

            // Set the caller to the initiator
            test::set_caller::<ink::env::DefaultEnvironment>(accounts.alice);
            // Set the transferred value
            test::set_value_transferred::<ink::env::DefaultEnvironment>(100);

            let swap_id = contract.initiate_swap(accounts.bob, 200).unwrap();

            // Set the caller to the initiator
            test::set_caller::<ink::env::DefaultEnvironment>(accounts.alice);

            let result = contract.cancel_swap(swap_id);
            assert!(result.is_ok());
        }

        #[ink::test]
        fn test_accept_swap_not_authorized() {
            let mut contract = SwapContract::new();
            let accounts = test::default_accounts::<ink::env::DefaultEnvironment>();

            // Set the caller to the initiator
            test::set_caller::<ink::env::DefaultEnvironment>(accounts.alice);
            // Set the transferred value
            test::set_value_transferred::<ink::env::DefaultEnvironment>(100);

            let swap_id = contract.initiate_swap(accounts.bob, 200).unwrap();

            // Set the caller to a different account
            test::set_caller::<ink::env::DefaultEnvironment>(accounts.charlie);
            // Set the transferred value
            test::set_value_transferred::<ink::env::DefaultEnvironment>(200);

            let result = contract.accept_swap(swap_id);
            assert_eq!(result, Err(Error::NotAuthorized));
        }

        #[ink::test]
        fn test_cancel_swap_not_authorized() {
            let mut contract = SwapContract::new();
            let accounts = test::default_accounts::<ink::env::DefaultEnvironment>();

            // Set the caller to the initiator
            test::set_caller::<ink::env::DefaultEnvironment>(accounts.alice);
            // Set the transferred value
            test::set_value_transferred::<ink::env::DefaultEnvironment>(100);

            let swap_id = contract.initiate_swap(accounts.bob, 200).unwrap();

            // Set the caller to a different account
            test::set_caller::<ink::env::DefaultEnvironment>(accounts.charlie);

            let result = contract.cancel_swap(swap_id);
            assert_eq!(result, Err(Error::NotAuthorized));
        }
    }
    
}
