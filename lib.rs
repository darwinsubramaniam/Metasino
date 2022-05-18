#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract]
mod metasino {

    use ink_prelude::vec::Vec;

    /// The maximum players alowed in the game participaction.
    const MAX_PLAYERS: u8 = 10;
    /// The minimum player required to start the game.
    const MIN_PLAYERS: u8 = 3;

    #[derive(
        Debug,
        Copy,
        Clone,
        PartialEq,
        Eq,
        scale::Encode,
        scale::Decode,
        ink_storage::traits::SpreadLayout,
        ink_storage::traits::PackedLayout,
    )]
    #[cfg_attr(
        feature = "std",
        derive(::scale_info::TypeInfo, ::ink_storage::traits::StorageLayout)
    )]
    pub enum STATE {
        STAGING,
        PLAYING,
        ENDED
    }

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
        state: STATE,
    }

    impl Metasino {
        /// Constructor that initializes the `bool` value to the given `init_value`.
        #[ink(constructor)]
        pub fn new(required_start_bet: Balance) -> Self {
            if required_start_bet <= 0 {
                panic!("Required start bet must be greater than 0");
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
                state: STATE::STAGING,
            }
        }

        #[ink(message)]
        pub fn terminate(&mut self) {
            self.table_status_guard();
            if self.get_players().contains(&Self::env().caller()) {
                panic!("Only the initializer can terminate the game");
            }
            self.players.clear();
        }

        /// Register new player into the table.
        /// error if the player is already registered.
        /// error if the table is full.
        /// error if new player places bet less or more than the required start bet.
        #[ink(message)]
        pub fn register_player(&mut self, start_bet: Balance) {
            self.table_status_guard();
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

        /// Start the game by extending the table to the game contract.
        #[ink(message)]
        pub fn start_game(&mut self) {
            self.table_status_guard();

            if self.get_players_count() < MIN_PLAYERS {
                panic!("Minimum {} players required to start the game", MIN_PLAYERS);
            }
            self.state = STATE::PLAYING;
        }

        /// Guarding the contract from being executed in a wrong state.
        fn table_status_guard(&self){
            if self.state == STATE::PLAYING {
                panic!("Game is ongoing!!");
            }

            if self.state == STATE::ENDED {
                panic!("Game has already has ended");
            }
        }

        #[ink(message)]
        pub fn get_table_state(&self) -> STATE {
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

        /// Test constructor works as per expected.
        /// - Test the required start bet value is as per the initialized.
        /// - Test the initializer is the caller who initialized the contract.
        /// - Test the total number of player is 1 when initialized.
        /// - Test the total accumulated pot value is as start bet value put by the initializer caller.
        /// - Test the state is 0 when initialized.
        #[ink::test]
        fn initialize_with_player_count_equal_one() {
            let accounts = ink_env::test::default_accounts::<ink_env::DefaultEnvironment>();
            ink_env::test::set_caller::<ink_env::DefaultEnvironment>(accounts.alice);
            let metasino = Metasino::new(100);
            assert_eq!(true, metasino.get_required_start_bet().eq(&100));
            assert_eq!(accounts.alice, metasino.initializer);
            assert_eq!(metasino.get_players_count(), 1);
            assert_eq!(metasino.get_accumulated_pot(), 100);
            assert_eq!(metasino.get_table_state(), STATE::STAGING);
        }

        #[ink::test]
        #[should_panic = "Player already registered"]
        fn register_same_player_will_fail() {
            let accounts = ink_env::test::default_accounts::<ink_env::DefaultEnvironment>();
            ink_env::test::set_caller::<ink_env::DefaultEnvironment>(accounts.alice);
            let mut metasino = Metasino::new(100);
            metasino.register_player(100);
        }

        #[ink::test]
        fn adding_new_player() {
            let accounts = ink_env::test::default_accounts::<ink_env::DefaultEnvironment>();
            ink_env::test::set_caller::<ink_env::DefaultEnvironment>(accounts.alice);
            let mut metasino = Metasino::new(100);
            ink_env::test::set_caller::<ink_env::DefaultEnvironment>(accounts.bob);
            metasino.register_player(100);
            assert_eq!(metasino.get_players_count(), 2);
            assert_eq!(metasino.get_accumulated_pot(), 200);
            assert_eq!(metasino.get_players()[0], accounts.alice);
            assert_eq!(metasino.get_players()[1], accounts.bob);
            assert_eq!(metasino.get_table_state(), STATE::STAGING);
            assert_eq!(metasino.get_required_start_bet(), 100);
        }

        #[ink::test]
        #[should_panic = "Required start bet must be greater than 0"]
        fn initialize_with_zero_start_bet() {
            Metasino::new(0);
        }

        #[ink::test]
        #[should_panic = "Minimum 3 players required to start the game"]
        fn less_than_minimum_player_unable_to_start_game(){
            let accounts = ink_env::test::default_accounts::<ink_env::DefaultEnvironment>();
            ink_env::test::set_caller::<ink_env::DefaultEnvironment>(accounts.alice);
            let mut metasino = Metasino::new(100);
            ink_env::test::set_caller::<ink_env::DefaultEnvironment>(accounts.bob);
            metasino.register_player(100);
            metasino.start_game();
        }

        #[ink::test]
        fn able_to_start_game(){
            let accounts = ink_env::test::default_accounts::<ink_env::DefaultEnvironment>();
            ink_env::test::set_caller::<ink_env::DefaultEnvironment>(accounts.alice);
            let mut metasino = Metasino::new(100);
            ink_env::test::set_caller::<ink_env::DefaultEnvironment>(accounts.bob);
            metasino.register_player(100);
            ink_env::test::set_caller::<ink_env::DefaultEnvironment>(accounts.charlie);
            metasino.register_player(100);
            metasino.start_game();
            assert_eq!(metasino.get_table_state(), STATE::PLAYING);
        }

        #[ink::test]
        #[should_panic = "Game is ongoing!!"]
        fn fail_to_add_player_when_game_status_started(){
            let accounts = ink_env::test::default_accounts::<ink_env::DefaultEnvironment>();
            ink_env::test::set_caller::<ink_env::DefaultEnvironment>(accounts.alice);
            let mut metasino = Metasino::new(100);
            ink_env::test::set_caller::<ink_env::DefaultEnvironment>(accounts.bob);
            metasino.register_player(100);
            ink_env::test::set_caller::<ink_env::DefaultEnvironment>(accounts.charlie);
            metasino.register_player(100);
            metasino.start_game();

            ink_env::test::set_caller::<ink_env::DefaultEnvironment>(accounts.django);
            metasino.register_player(100);
        }

        #[ink::test]
        #[should_panic = "Game is ongoing!!"]
        fn should_not_allow_termination_if_table_game_in_started_state(){
            let accounts = ink_env::test::default_accounts::<ink_env::DefaultEnvironment>();
            ink_env::test::set_caller::<ink_env::DefaultEnvironment>(accounts.alice);
            let mut metasino = Metasino::new(100);
            ink_env::test::set_caller::<ink_env::DefaultEnvironment>(accounts.bob);
            metasino.register_player(100);
            ink_env::test::set_caller::<ink_env::DefaultEnvironment>(accounts.charlie);
            metasino.register_player(100);
            metasino.start_game();
            metasino.terminate();
        }
    }
}
