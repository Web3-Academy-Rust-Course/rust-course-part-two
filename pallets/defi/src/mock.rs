use crate as pallet_defi;
use codec::Decode;
use frame_support::{
	construct_runtime, parameter_types,
	traits::{Everything, Hooks},
	PalletId,
};
use frame_system;
use hex_literal::hex;
use sp_core::H256;
use sp_runtime::{
	traits::{AccountIdConversion, BlakeTwo256, IdentityLookup},
	BuildStorage, FixedU128,
};

pub type Block = frame_system::mocking::MockBlock<Runtime>;
pub type Balance = u128;
pub type AccountId = u128;

#[macro_export]
macro_rules! balance {
	($num:expr) => {{
		let num: u64 = $num.try_into().unwrap();
		num as u128 * 1_000_000_000_000_000_00
	}};
}

// Define test accounts
pub const ALICE: AccountId = 1;
pub const BOB: AccountId = 2;
pub const CHARLIE: AccountId = 3;

// Define helper function
pub fn get_authority_account() -> AccountId {
	let bytes = hex!("fed77d0df3f5068d8a875e5ae7c3248ba7c602439623cab507206af8e50edd4b");
	AccountId::decode(&mut &bytes[..]).unwrap()
}

pub fn pallet_id() -> AccountId {
	PalletId(*b"defipllt").into_account_truncating()
}

pub fn get_default_deposit_rate() -> FixedU128 {
	FixedU128::from_inner(92828) / FixedU128::from_inner(10000000000000)
}

pub fn get_default_borrowing_rate() -> FixedU128 {
	FixedU128::from_inner(128727) / FixedU128::from_inner(10000000000000)
}

pub fn get_default_collateral_factor() -> FixedU128 {
	FixedU128::from_inner(75) / FixedU128::from_inner(100)
}

// Configure a mock runtime to test the pallet.
construct_runtime!(
	pub struct Runtime {
		System: frame_system,
		Balances: pallet_balances,
		Defi: pallet_defi,
	}
);

parameter_types! {
	pub const BlockHashCount: u64 = 250;
}

impl frame_system::Config for Runtime {
	type BaseCallFilter = Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeCall = RuntimeCall;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type RuntimeEvent = RuntimeEvent;
	type BlockHashCount = BlockHashCount;
	type DbWeight = ();
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<Balance>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ();
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<65536>;
	type Nonce = u32;
	type Block = Block;
}

parameter_types! {
	pub const ExistentialDeposit: u128 = 1;
}

impl pallet_balances::Config for Runtime {
	type Balance = Balance;
	type DustRemoval = ();
	type RuntimeEvent = RuntimeEvent;
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = ();
	type MaxLocks = ();
	type MaxReserves = ();
	type ReserveIdentifier = ();
	type MaxHolds = ();
	type MaxFreezes = ();
	type RuntimeHoldReason = ();
	type FreezeIdentifier = ();
}

parameter_types! {
	pub const NumberOfBlocksYearly: u32 = 5256000;
}

impl pallet_defi::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type NumberOfBlocksYearly = NumberOfBlocksYearly;
}

pub struct ExtBuilder {
	endowed_accounts: Vec<(AccountId, Balance)>,
}

impl Default for ExtBuilder {
	fn default() -> Self {
		Self {
			endowed_accounts: vec![
				(ALICE, balance!(100)),
				(BOB, balance!(100)),
				(CHARLIE, balance!(100)),
			],
		}
	}
}

impl ExtBuilder {
	pub fn build(self) -> sp_io::TestExternalities {
		let mut storage = RuntimeGenesisConfig::default().build_storage().unwrap();

		pallet_balances::GenesisConfig::<Runtime> {
			balances: self.endowed_accounts.iter().map(|(acc, balance)| (*acc, *balance)).collect(),
		}
		.assimilate_storage(&mut storage)
		.unwrap();

		storage.into()
	}
}

pub fn run_to_block(n: u64) {
	while System::block_number() < n {
		System::on_finalize(System::block_number());
		System::set_block_number(System::block_number() + 1);
		System::on_initialize(System::block_number());
		Defi::on_initialize(System::block_number());
	}
}
