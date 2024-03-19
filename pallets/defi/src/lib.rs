
#![cfg_attr(not(feature = "std"), no_std)]

pub use codec::{Decode, Encode};
pub use pallet::*;

#[derive(Encode, Decode, Default, PartialEq, Eq, scale_info::TypeInfo)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct AddressInfo<Balance, BlockNumber> {
	/// The deposit balance of the account after last adjustment
	deposit_principal: Balance,
	/// The time (block height) at which the deposit balance was last adjusted
	deposit_date: BlockNumber,
	/// The borrowing balance of the account after last adjustment
	borrow_principal: Balance,
	/// The time (block height) at which the borrowing balance was last adjusted
	borrow_date: BlockNumber,
}

#[frame_support::pallet(dev_mode)]
pub mod pallet {
	use super::*;
	use frame_support::{
		pallet_prelude::*,
		sp_runtime::{
			FixedU128
		},
		traits::{Currency, ReservableCurrency},
	};
	use frame_system::pallet_prelude::*;
	use hex_literal::hex;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The currency in which deposit/borrowing work
		type Currency: ReservableCurrency<Self::AccountId>;

		/// Number of blocks on yearly basis
		type NumberOfBlocksYearly: Get<u32>;
	}

	type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
	type BalanceOf<T> = <<T as Config>::Currency as Currency<AccountIdOf<T>>>::Balance;
	type BlockNumber<T> = BlockNumberFor<T>;

	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	// Default authority account
	#[pallet::type_value]
	pub fn DefaultForAuthorityAccount<T: Config>() -> AccountIdOf<T> {
		let bytes = hex!("96ea3c9c0be7bbc7b0656a1983db5eed75210256891a9609012362e36815b132");
		AccountIdOf::<T>::decode(&mut &bytes[..]).unwrap()
	}

	// Deposit rate default value
	#[pallet::type_value]
	pub fn DefaultDepositRate<T: Config>() -> FixedU128 {
		FixedU128::from_inner(92828) / FixedU128::from_inner(10000000000000)
	}

	// Borrowing rate default value
	#[pallet::type_value]
	pub fn DefaultBorrowingRate<T: Config>() -> FixedU128 {
		FixedU128::from_inner(128727) / FixedU128::from_inner(10000000000000)
	}

	// Collateral factor default value
	#[pallet::type_value]
	pub fn DefaultCollateralFactor<T: Config>() -> FixedU128 {
		FixedU128::from_inner(75) / FixedU128::from_inner(100)
	}

	#[pallet::storage]
	#[pallet::getter(fn authority_account)]
	pub type AuthorityAccount<T: Config> =
		StorageValue<_, AccountIdOf<T>, ValueQuery, DefaultForAuthorityAccount<T>>;

	#[pallet::storage]
	#[pallet::getter(fn deposit_rate)]
	pub type DepositRate<T: Config> = StorageValue<_, FixedU128, ValueQuery, DefaultDepositRate<T>>;

	#[pallet::storage]
	#[pallet::getter(fn borrowing_rate)]
	pub type BorrowingRate<T: Config> =
		StorageValue<_, FixedU128, ValueQuery, DefaultBorrowingRate<T>>;

	#[pallet::storage]
	#[pallet::getter(fn collateral_factor)]
	pub type CollateralFactor<T: Config> =
		StorageValue<_, FixedU128, ValueQuery, DefaultCollateralFactor<T>>;

	#[pallet::storage]
	#[pallet::getter(fn accounts)]
	pub(super) type Accounts<T: Config> = StorageMap<
		_,
		Identity,
		AccountIdOf<T>,
		AddressInfo<BalanceOf<T>, BlockNumber<T>>,
		ValueQuery,
	>;
}
