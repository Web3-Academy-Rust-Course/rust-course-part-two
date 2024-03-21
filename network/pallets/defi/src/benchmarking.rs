#![cfg(feature = "runtime-benchmarks")]

use super::*;

use crate::Pallet as Defi;
use codec::Decode;
use frame_benchmarking::benchmarks;
use frame_support::{sp_runtime::FixedU128, traits::Hooks};
use frame_system::{EventRecord, RawOrigin};
use hex_literal::hex;

// Define helper function
fn authority<T: Config>() -> T::AccountId {
	let bytes = hex!("fed77d0df3f5068d8a875e5ae7c3248ba7c602439623cab507206af8e50edd4b");

	T::AccountId::decode(&mut &bytes[..]).unwrap()
}

fn alice<T: Config>() -> T::AccountId {
	let bytes = hex!("d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d");

	T::AccountId::decode(&mut &bytes[..]).unwrap()
}

fn bob<T: Config>() -> T::AccountId {
	let bytes = hex!("8eaf04151687736326c9fea17e25fc5287613693c912909cb226aa4794f26a48");

	T::AccountId::decode(&mut &bytes[..]).unwrap()
}

fn assert_last_event<T: Config>(generic_event: <T as Config>::RuntimeEvent) {
	let events = frame_system::Pallet::<T>::events();
	let system_event: <T as frame_system::Config>::RuntimeEvent = generic_event.into();

	// Compare to the last event record
	let EventRecord { event, .. } = &events[events.len() - 1];

	assert_eq!(event, &system_event);
}

fn run_to_block<T: Config>(n: u32) {
	while frame_system::Pallet::<T>::block_number() < n.into() {
		frame_system::Pallet::<T>::on_finalize(frame_system::Pallet::<T>::block_number().into());
		frame_system::Pallet::<T>::set_block_number(
			frame_system::Pallet::<T>::block_number() + 1u32.into(),
		);
		frame_system::Pallet::<T>::on_initialize(frame_system::Pallet::<T>::block_number().into());
		Defi::<T>::on_initialize(frame_system::Pallet::<T>::block_number().into());
	}
}

benchmarks! {
	deposit {
		let user = alice::<T>();
		let amount = 1u32;
	} : {
		Defi::<T>::deposit(
			RawOrigin::Signed(user.clone()).into(),
			amount.into(),
		).unwrap();
	} verify {
		let current_block = frame_system::Pallet::<T>::block_number();

		assert_last_event::<T>(Event::<T>::Deposited(user.clone(), amount.into(), current_block).into());
	}

	withdraw {
		let user = alice::<T>();
		let deposit_amount = 10u32;
		let withdraw_amount = 5u32;
	} : {
		Defi::<T>::deposit(
			RawOrigin::Signed(user.clone()).into(),
			deposit_amount.into(),
		).unwrap();

		let _ =Defi::<T>::withdraw(
			RawOrigin::Signed(user.clone()).into(),
			withdraw_amount.into(),
		);
	} verify {
		let current_block = frame_system::Pallet::<T>::block_number();

		assert_last_event::<T>(Event::<T>::Withdrawn(user.clone(), withdraw_amount.into(), current_block).into());
	}

	borrow {
		let borrowing_user = alice::<T>();
		let depositing_user = bob::<T>();
		let borrowing_amount: u32 = 5;
		let depositing_amount_1: u32 = 10;
		let depositing_amount_2: u32 = 100;
	} : {
		Defi::<T>::deposit(
			RawOrigin::Signed(depositing_user.clone()).into(),
			depositing_amount_2.into(),
		).unwrap();

		Defi::<T>::deposit(
			RawOrigin::Signed(borrowing_user.clone()).into(),
			depositing_amount_1.into(),
		).unwrap();

		let _ = Defi::<T>::borrow(
			RawOrigin::Signed(borrowing_user.clone()).into(),
			borrowing_amount.into(),
		);
	} verify {
		let current_block = frame_system::Pallet::<T>::block_number();

		assert_last_event::<T>(Event::<T>::Borrowed(borrowing_user.clone(), borrowing_amount.into(), current_block).into());
	}

	repay {
		let user = alice::<T>();
		let borrowing_amount: u32 = 5;
		let depositing_amount: u32 = 10;
	} : {
		run_to_block::<T>(1);

		Defi::<T>::deposit(
			RawOrigin::Signed(user.clone()).into(),
			depositing_amount.into(),
		).unwrap();

		Defi::<T>::borrow(
			RawOrigin::Signed(user.clone()).into(),
			borrowing_amount.into(),
		).unwrap();

		run_to_block::<T>(11);

		let _ = Defi::<T>::repay(
			RawOrigin::Signed(user.clone()).into(),
			borrowing_amount.into(),
		);
	} verify {
		let current_block = frame_system::Pallet::<T>::block_number();

		assert_last_event::<T>(Event::<T>::LoanRepaid(user.clone(), borrowing_amount.into(), current_block).into());
	}

	update_deposit_rate {
		let authority = authority::<T>();
		let new_rate = FixedU128::from_inner(1);
	} : {
		let _ = Defi::<T>::update_deposit_rate(
			RawOrigin::Signed(authority).into(),
			new_rate,
		);
	} verify {
		assert_last_event::<T>(Event::<T>::DepositRateUpdated(new_rate).into());
	}

	update_borrowing_rate {
		let authority = authority::<T>();
		let new_rate = FixedU128::from_inner(1);
	} : {
		let _ = Defi::<T>::update_borrowing_rate(
			RawOrigin::Signed(authority).into(),
			new_rate,
		);
	} verify {
		assert_last_event::<T>(Event::<T>::BorrowingRateUpdated(new_rate).into());
	}

	update_collateral_factor {
		let authority = authority::<T>();
		let new_factor = FixedU128::from_inner(1);
	} : {
		let _ = Defi::<T>::update_collateral_factor(
			RawOrigin::Signed(authority).into(),
			new_factor,
		);
	} verify {
		assert_last_event::<T>(Event::<T>::CollateralFactorUpdated(new_factor).into());
	}

	impl_benchmark_test_suite!(
		Pallet,
		crate::mock::ExtBuilder::default().build(),
		crate::mock::Runtime,
	);
}