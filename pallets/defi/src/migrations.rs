use crate::{Accounts, AddressInfo, BalanceOf, BlockNumber, Config};
use core::cmp;
use frame_support::{
	log,
	pallet_prelude::Weight,
	sp_runtime::{runtime_logger::RuntimeLogger, traits::Zero},
	traits::Get,
};

pub fn migrate<T: Config>() -> Weight {
	RuntimeLogger::init();
	migrate_accounts_data::<T>()
}

pub fn migrate_accounts_data<T: Config>() -> Weight {
	let mut weight: u64 = 0;

	let current_block = frame_system::Pallet::<T>::block_number();

	Accounts::<T>::translate_values::<
		(BalanceOf<T>, BlockNumber<T>, BalanceOf<T>, BlockNumber<T>),
		_,
	>(|(deposit_principal, deposit_date, borrow_principal, borrow_date)| {
		weight += 1;

		let mut created_at: BlockNumber<T> = BlockNumber::<T>::zero();

		if deposit_date == created_at && borrow_date == created_at {
			created_at = current_block;
		} else if deposit_date == created_at || borrow_date == created_at {
			created_at = cmp::max(deposit_date, borrow_date);
		} else {
			created_at = cmp::min(deposit_date, borrow_date);
		}

		Some(AddressInfo {
			deposit_principal,
			deposit_date,
			borrow_principal,
			borrow_date,
			created_at,
		})
	});

	log::info!(
		target: "runtime",
		"⚠️ Accounts migrated to new version with the created_at field"
	);

	T::DbWeight::get().reads_writes(weight, weight)
}
