#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract]
mod metasino {

    use ink_prelude::vec::Vec;

    /// The maximum players alowed in the game participaction.
    const MAX_PLAYERS: u8 = 10;
    /// The minimum player required to start the game.
    const MIN_PLAYERS: u8 = 3;

    #[ink(event)]
    pub struct NewTableOpened {
        #[ink(topic)]
        pub initiator: AccountId,
        #[ink(topic)]
        pub required_start_bet: Balance,
    }

    #[ink(event)]
    pub struct MinimumPlayerReached {
        #[ink(topic)]
        pub account_id: AccountId,
    }

    /// Defines the storage of your contract.
    /// Add new fields to the below struct in order
    /// to add new static storage fields to your contract.
    /// StorageLayout is used to define the layout of the storage.
    /// SpreadLayout is used to define the layout of the spread.
    #[ink(storage)]
    pub struct Metasino {
        /// Account which Initialized the contract.
        initializer: AccountId,
        /// Store the current number of players ready for the game.
        players: Vec<AccountId>,
        /// Start betting value,
        required_start_bet: Balance,
        /// Accumulated value in the pot.
        pot: Balance,
        /// The current state of the game.
        /// Haven not figure out how to use ENUM in contract.
        /// Temporary solutoin is to use u8 
        /// 0: Not started
        /// 1: Started
        /// 3: Ended
        state: u8,
    }

    impl Metasino {
        /// Constructor that initializes the `bool` value to the given `init_value`.
        #[ink(constructor)]
        pub fn new(required_start_bet: Balance) -> Self {
            if required_start_bet <= 0 {
                panic!("Start bet must be greater than 0");
            }
            let mut players: Vec<AccountId> = Vec::new();
            players.push(Self::env().caller());
            Self::env().emit_event(NewTableOpened {
                initiator: Self::env().caller(),
                required_start_bet,
            });
            Self {
                initializer: Self::env().caller(),
                required_start_bet,
                players,
                pot: required_start_bet,
                state: 0,
            }
        }

        #[ink(message)]
        pub fn terminate(&mut self) {
            self.players.clear();
        }

        /// Register new player into the table.
        /// error if the player is already registered.
        /// error if the table is full.
        /// error if new player places bet less or more than the required start bet.
        #[ink(message)]
        pub fn register_player(&mut self, start_bet: Balance) {
            let caller = Self::env().caller();
            if self.get_players_count() >= MAX_PLAYERS {
                panic!("Max players reached");
            }

            if start_bet != self.required_start_bet {
                panic!(
                    "start Bet value requires at exact {}",
                    self.required_start_bet
                );
            }

            self.pot += start_bet;
            if !self.players.contains(&caller) {
                self.players.push(caller);
            } else {
                panic!("Player already registered");
            }
        }

        #[ink(message)]
        pub fn start_game(&mut self) {
            if self.get_players_count() < MIN_PLAYERS {
                panic!("Minimum {} players required to start the game", MIN_PLAYERS);
            }
            self.state = 1;
        }

        #[ink(message)]
        pub fn get_table_state(&self) -> u8 {
            self.state
        }

        /// Get the current number of players in the table.
        #[ink(message)]
        pub fn get_players_count(&self) -> u8 {
            self.players.len() as u8
        }

        /// Get the current accumulated pot value.
        #[ink(message)]
        pub fn get_accumulated_pot(&self) -> Balance {
            self.pot
        }

        /// Get address of the player in the table.
        #[ink(message)]
        pub fn get_players(&self) -> Vec<AccountId> {
            self.players.clone()
        }

        /// Get the required start bet value.
        #[ink(message)]
        pub fn get_required_start_bet(&self) -> Balance {
            self.required_start_bet
        }
    }

    /// Unit tests in Rust are normally defined within such a `#[cfg(test)]`
    /// module and test functions are marked with a `#[test]` attribute.
    /// The below code is technically just normal Rust code.
    #[cfg(test)]
    mod tests {
        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;

        /// Imports `ink_lang` so we can use `#[ink::test]`.
        use ink_lang as ink;

        /// We test if the default constructor does its job.
        #[ink::test]
        fn initialize_with_player_count_equal_one() {
            let metasino = Metasino::new(100);
            assert_eq!(metasino.get_players_count(), 1);
        }

        #[ink::test]
        #[should_panic = "Player already registered"]
        fn register_same_player_will_fail() {
            let mut metasino = Metasino::new(100);
            metasino.register_player(100);
        }
    }
}
