#![cfg_attr(not(feature = "std"), no_std)]

pub use codec::{Decode, Encode};
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

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
			traits::{AccountIdConversion, One, Zero},
			FixedU128, SaturatedConversion, Saturating,
		},
		traits::{Currency, ExistenceRequirement, ReservableCurrency},
		transactional, PalletId,
	};
	use frame_system::pallet_prelude::*;
	use hex_literal::hex;

	const PALLET_ID: PalletId = PalletId(*b"defipllt");

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

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
		let bytes = hex!("fed77d0df3f5068d8a875e5ae7c3248ba7c602439623cab507206af8e50edd4b");
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

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Funds deposited [who, amount, block]
		Deposited(AccountIdOf<T>, BalanceOf<T>, BlockNumber<T>),
		/// Funds withdrawn [who, amount, block]
		Withdrawn(AccountIdOf<T>, BalanceOf<T>, BlockNumber<T>),
		/// Loan repaid [who, amount, block]
		LoanRepaid(AccountIdOf<T>, BalanceOf<T>, BlockNumber<T>),
		/// Funds borrowed [who, amount, block]
		Borrowed(AccountIdOf<T>, BalanceOf<T>, BlockNumber<T>),
		/// Address liquidated [who]
		AddressLiquidated(AccountIdOf<T>),
		/// Deposit rate updated [rate]
		DepositRateUpdated(FixedU128),
		/// Borrowing rate updated [rate]
		BorrowingRateUpdated(FixedU128),
		/// Collateral factor updated [factor]
		CollateralFactorUpdated(FixedU128),
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Invalid deposit amount
		InvalidDepositAmount,
		/// Insufficient balance
		InsufficientBalance,
		/// User has no deposited balance
		NoFundsDeposited,
		/// Pallet has not enough funds to pay the user
		PalletHasNotEnoughFunds,
		/// User has not as much funds as he asked for
		UserHasNotEnoughFunds,
		/// Unallowed borrow amount
		UnallowedBorrowAmount,
		/// Nothing to repay
		NothingToRepay,
		/// Unauthorized user action
		UnauthorizedUserAction,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Deposit funds
		#[transactional]
		#[pallet::call_index(0)]
		#[pallet::weight(0)]
		pub fn deposit(origin: OriginFor<T>, amount: BalanceOf<T>) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			// This function will return an error if the extrinsic is not signed.
			let user = ensure_signed(origin)?;

			// Check if the deposited amount is greater than 0
			ensure!(amount > <BalanceOf<T>>::zero(), Error::<T>::InvalidDepositAmount);

			// Check if user has enough funds
			ensure!(T::Currency::free_balance(&user) >= amount, Error::<T>::InsufficientBalance);

			// Get address info of extrinsic caller
			let mut address_info = Accounts::<T>::get(&user);

			// Get current block
			let current_block = frame_system::Pallet::<T>::block_number();

			// Deposit funds to pallet
			T::Currency::transfer(
				&user,
				&Self::account_id(),
				amount,
				ExistenceRequirement::KeepAlive,
			)?;

			// Set address info
			let deposit_principal_fixed =
				FixedU128::from_inner(address_info.deposit_principal.saturated_into::<u128>());
			let amount_fixed = FixedU128::from_inner(amount.saturated_into::<u128>());
			address_info.deposit_principal = deposit_principal_fixed
				.saturating_add(amount_fixed)
				.into_inner()
				.saturated_into();
			address_info.deposit_date = current_block;

			// Put updated address info into storage
			Accounts::<T>::insert(&user, address_info);

			// Emit an event
			Self::deposit_event(Event::Deposited(user, amount, current_block));

			// Return a successful DispatchResult
			Ok(())
		}

		/// Withdraw funds
		#[transactional]
		#[pallet::call_index(1)]
		#[pallet::weight(0)]
		pub fn withdraw(origin: OriginFor<T>, amount: BalanceOf<T>) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			// This function will return an error if the extrinsic is not signed.
			let user = ensure_signed(origin)?;

			// Get address info of extrinsic caller and check if it has deposited funds
			let mut address_info = Accounts::<T>::get(&user);

			ensure!(
				address_info.deposit_principal != <BalanceOf<T>>::zero(),
				Error::<T>::NoFundsDeposited
			);

			// Check if user and pallet have enough funds
			let balance_info = Self::get_lending_amount(user.clone());

			ensure!(amount <= balance_info, Error::<T>::UserHasNotEnoughFunds);

			ensure!(
				amount <= T::Currency::free_balance(&Self::account_id()),
				Error::<T>::PalletHasNotEnoughFunds
			);

			// Get current block
			let current_block = frame_system::Pallet::<T>::block_number();

			// Withdraw funds from pallet
			T::Currency::transfer(
				&Self::account_id(),
				&user,
				amount,
				ExistenceRequirement::KeepAlive,
			)?;

			// Update address info
			if amount == address_info.deposit_principal {
				address_info.deposit_date = BlockNumber::<T>::zero();
			} else {
				address_info.deposit_date = current_block;
			}

			let deposit_principal_fixed =
				FixedU128::from_inner(address_info.deposit_principal.saturated_into::<u128>());
			let amount_fixed = FixedU128::from_inner(amount.saturated_into::<u128>());
			address_info.deposit_principal = deposit_principal_fixed
				.saturating_sub(amount_fixed)
				.into_inner()
				.saturated_into();

			// Put updated address info into storage
			Accounts::<T>::insert(&user, address_info);

			// Emit an event
			Self::deposit_event(Event::Withdrawn(user, amount, current_block));

			// Return a successful DispatchResult
			Ok(())
		}

		/// Borrow funds
		#[transactional]
		#[pallet::call_index(2)]
		#[pallet::weight(0)]
		pub fn borrow(origin: OriginFor<T>, amount: BalanceOf<T>) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			// This function will return an error if the extrinsic is not signed.
			let user = ensure_signed(origin)?;

			// Check if pallet has enough funds
			ensure!(
				amount <= T::Currency::free_balance(&Self::account_id()),
				Error::<T>::PalletHasNotEnoughFunds
			);

			// Get address info of extrinsic caller
			let mut address_info = Accounts::<T>::get(&user);

			// Get allowed borrowing amount
			let borrowing_balance = Self::get_debt_amount(user.clone());
			let borrowing_info =
				Self::get_allowed_borrowing_amount(user.clone(), borrowing_balance, false);
			ensure!(amount <= borrowing_info, Error::<T>::UnallowedBorrowAmount);

			// Get current block
			let current_block = frame_system::Pallet::<T>::block_number();

			// Borrow funds from pallet
			T::Currency::transfer(
				&Self::account_id(),
				&user,
				amount,
				ExistenceRequirement::KeepAlive,
			)?;

			// Update address info
			let borrowing_balance_fixed =
				FixedU128::from_inner(borrowing_balance.saturated_into::<u128>());
			let amount_fixed = FixedU128::from_inner(amount.saturated_into::<u128>());
			address_info.borrow_principal = borrowing_balance_fixed
				.saturating_add(amount_fixed)
				.into_inner()
				.saturated_into();
			address_info.borrow_date = current_block;

			// Put updated address info into storage
			Accounts::<T>::insert(&user, address_info);

			// Emit an event
			Self::deposit_event(Event::Borrowed(user, amount, current_block));

			// Return a successful DispatchResult
			Ok(())
		}

		/// Repay loan
		#[transactional]
		#[pallet::call_index(3)]
		#[pallet::weight(0)]
		pub fn repay(origin: OriginFor<T>, mut amount: BalanceOf<T>) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			// This function will return an error if the extrinsic is not signed.
			let user = ensure_signed(origin)?;

			// Check if the user has enough on his balance
			ensure!(T::Currency::free_balance(&user) >= amount, Error::<T>::InsufficientBalance);

			// Check if the user has anything to repay
			let mut address_info = Accounts::<T>::get(&user);

			ensure!(
				address_info.borrow_principal > <BalanceOf<T>>::zero(),
				Error::<T>::NothingToRepay
			);

			// Check if there is repay overflow
			let balance_info = Self::get_debt_amount(user.clone());
			if amount > balance_info {
				amount = balance_info;
			}

			// Transfer funds from user to pallet
			T::Currency::transfer(
				&user,
				&Self::account_id(),
				amount,
				ExistenceRequirement::KeepAlive,
			)?;

			// Get principal with accrued interest
			let current_block = frame_system::Pallet::<T>::block_number();

			// Update address info
			if amount == balance_info {
				address_info.borrow_date = BlockNumber::<T>::zero();
			} else {
				address_info.borrow_date = current_block;
			}

			let balance_fixed = FixedU128::from_inner(balance_info.saturated_into::<u128>());
			let amount_fixed = FixedU128::from_inner(amount.saturated_into::<u128>());
			address_info.borrow_principal =
				balance_fixed.saturating_sub(amount_fixed).into_inner().saturated_into();

			// Put updated address info into storage
			Accounts::<T>::insert(&user, address_info);

			// Emit an event
			Self::deposit_event(Event::LoanRepaid(user, amount, current_block));

			// Return a successful DispatchResult
			Ok(())
		}

		/// Update deposit rate
		#[pallet::call_index(4)]
		#[pallet::weight(0)]
		pub fn update_deposit_rate(origin: OriginFor<T>, new_rate: FixedU128) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			// This function will return an error if the extrinsic is not signed.
			let user = ensure_signed(origin)?;

			// Check if the caller is the authority account
			ensure!(user == AuthorityAccount::<T>::get(), Error::<T>::UnauthorizedUserAction);

			// Update deposit rate
			DepositRate::<T>::put(new_rate);

			// Emit an event
			Self::deposit_event(Event::DepositRateUpdated(new_rate));

			Ok(())
		}

		/// Update borrowing rate
		#[pallet::call_index(5)]
		#[pallet::weight(0)]
		pub fn update_borrowing_rate(origin: OriginFor<T>, new_rate: FixedU128) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			// This function will return an error if the extrinsic is not signed.
			let user = ensure_signed(origin)?;

			// Check if the caller is the authority account
			ensure!(user == AuthorityAccount::<T>::get(), Error::<T>::UnauthorizedUserAction);

			// Update deposit rate
			BorrowingRate::<T>::put(new_rate);

			// Emit an event
			Self::deposit_event(Event::BorrowingRateUpdated(new_rate));

			Ok(())
		}

		/// Update collateral factor
		#[pallet::call_index(6)]
		#[pallet::weight(0)]
		pub fn update_collateral_factor(
			origin: OriginFor<T>,
			new_factor: FixedU128,
		) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			// This function will return an error if the extrinsic is not signed.
			let user = ensure_signed(origin)?;

			// Check if the caller is the authority account
			ensure!(user == AuthorityAccount::<T>::get(), Error::<T>::UnauthorizedUserAction);

			// Update deposit rate
			CollateralFactor::<T>::put(new_factor);

			// Emit an event
			Self::deposit_event(Event::CollateralFactorUpdated(new_factor));

			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		/// Get user's balance
		pub fn get_lending_amount(user: T::AccountId) -> BalanceOf<T> {
			// Get address info and check if deposit principal is zero
			let address_info = Accounts::<T>::get(user);

			if address_info.deposit_principal == <BalanceOf<T>>::zero() {
				return <BalanceOf<T>>::zero()
			}

			// Calculate principal with accrued interest
			let current_block = frame_system::Pallet::<T>::block_number();
			let balance = Self::get_principal_with_accrued_interest(
				current_block,
				address_info.deposit_date,
				address_info.deposit_principal,
				DepositRate::<T>::get(),
			);

			balance
		}

		/// Get user's debt
		pub fn get_debt_amount(user: T::AccountId) -> BalanceOf<T> {
			// Get address info and check if borrow principal is zero
			let address_info = Accounts::<T>::get(user);

			if address_info.borrow_principal == <BalanceOf<T>>::zero() {
				return <BalanceOf<T>>::zero()
			}

			// Calculate elapsed blocks
			let current_block = frame_system::Pallet::<T>::block_number();
			let balance = Self::get_principal_with_accrued_interest(
				current_block,
				address_info.borrow_date,
				address_info.borrow_principal,
				BorrowingRate::<T>::get(),
			);

			balance
		}

		/// Get user's allowed borrowing amount
		pub fn get_allowed_borrowing_amount(
			user: T::AccountId,
			mut borrowing_balance: BalanceOf<T>,
			is_rpc: bool,
		) -> BalanceOf<T> {
			// Get borrowing balance and deposit principal
			let deposit_balance = Self::get_lending_amount(user.clone());
			if is_rpc {
				borrowing_balance = Self::get_debt_amount(user.clone());
			}

			let deposit_balance_fixed =
				FixedU128::from_inner(deposit_balance.saturated_into::<u128>());
			let borrowing_balance_fixed =
				FixedU128::from_inner(borrowing_balance.saturated_into::<u128>());

			// Calculate allowed borrowing amount
			let allowed_borrowing_amount = (deposit_balance_fixed
				.saturating_mul(CollateralFactor::<T>::get())
				.saturating_sub(borrowing_balance_fixed))
			.into_inner()
			.saturated_into();

			allowed_borrowing_amount
		}

		/// Get deposit APY
		pub fn get_deposit_apy() -> BalanceOf<T> {
			let deposit_apy = (FixedU128::one().saturating_add(DepositRate::<T>::get()))
				.saturating_pow(T::NumberOfBlocksYearly::get() as usize)
				.saturating_sub(FixedU128::one());

			deposit_apy.into_inner().saturated_into()
		}

		/// Get borrowing APY
		pub fn get_borrowing_apy() -> BalanceOf<T> {
			let borrowing_apy = (FixedU128::one().saturating_add(BorrowingRate::<T>::get()))
				.saturating_pow(T::NumberOfBlocksYearly::get() as usize)
				.saturating_sub(FixedU128::one());

			borrowing_apy.into_inner().saturated_into()
		}

		/// Get principal with accrued interest
		fn get_principal_with_accrued_interest(
			current_block: BlockNumber<T>,
			date: BlockNumber<T>,
			principal: BalanceOf<T>,
			rate: FixedU128,
		) -> BalanceOf<T> {
			// Calculate elapsed blocks
			let elapsed_time_block_number = current_block - date;
			let elapsed_time: u32 = TryInto::try_into(elapsed_time_block_number)
				.ok()
				.expect("blockchain will not exceed 2^32 blocks; qed");

			// Calculate principal with accrued interest
			let multiplier = (FixedU128::one().saturating_add(rate))
				.saturating_pow(elapsed_time as usize)
				.saturating_sub(FixedU128::one());
			let principal_fixed = FixedU128::from_inner(principal.saturated_into::<u128>());

			return principal_fixed
				.saturating_add(principal_fixed.saturating_mul(multiplier))
				.into_inner()
				.saturated_into()
		}

		/// The account ID of pallet
		fn account_id() -> T::AccountId {
			PALLET_ID.into_account_truncating()
		}

		/// Hook functions that is called on each initialized block
		fn check_liquidity(_current_block: BlockNumber<T>) -> Weight {
			let mut counter: u64 = 0;

			for address in Accounts::<T>::iter() {
				let collateral = FixedU128::from_inner(
					Self::get_lending_amount(address.0.clone()).saturated_into::<u128>(),
				);
				let borrowed = FixedU128::from_inner(
					Self::get_debt_amount(address.0.clone()).saturated_into::<u128>(),
				);
				let collateralized_balance =
					collateral.saturating_mul(CollateralFactor::<T>::get());

				if borrowed >= collateralized_balance {
					Accounts::<T>::remove(&address.0);

					Self::deposit_event(Event::AddressLiquidated(address.0));

					counter += 1;
				}
			}

			T::DbWeight::get()
				.reads(counter)
				.saturating_add(T::DbWeight::get().writes(counter))
		}
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_initialize(now: BlockNumber<T>) -> Weight {
			let consumed_weight = Self::check_liquidity(now);

			consumed_weight
		}
	}
}