mod tests {
	use crate::{balance, mock::*, pallet, Error};
	use frame_support::{
		assert_err, assert_ok,
		sp_runtime::{traits::One, FixedU128, SaturatedConversion, Saturating},
	};
	use pallet_balances;

	// Global pallet values
	pub fn get_pallet_borrowing_rate() -> FixedU128 {
		FixedU128::from_inner(128727) / FixedU128::from_inner(10000000000000)
	}

	// Utility functions
	/// Get user's debt
	pub fn get_debt_amount(user: AccountId) -> Balance {
		// Get address info and check if borrow principal is zero
		let address_info = pallet::Accounts::<Runtime>::get(user);

		if address_info.borrow_principal == balance!(0) {
			return balance!(0)
		}

		// Calculate elapsed blocks
		let current_block = frame_system::Pallet::<Runtime>::block_number();
		let balance = get_principal_with_accrued_interest(
			current_block,
			address_info.borrow_date,
			address_info.borrow_principal,
			get_pallet_borrowing_rate(),
		);

		balance
	}

	/// Get principal with accrued interest
	fn get_principal_with_accrued_interest(
		current_block: u64,
		date: u64,
		principal: Balance,
		rate: FixedU128,
	) -> Balance {
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

	#[test]
	fn deposit_invalid_deposit_amount() {
		let mut ext = ExtBuilder::default().build();
		ext.execute_with(|| {
			// Start test from block 1
			run_to_block(1);

			assert_err!(
				Defi::deposit(RuntimeOrigin::signed(ALICE), balance!(0)),
				Error::<Runtime>::InvalidDepositAmount
			);
		});
	}

	#[test]
	fn deposit_insufficient_balance() {
		let mut ext = ExtBuilder::default().build();
		ext.execute_with(|| {
			// Start test from block 1
			run_to_block(1);

			assert_err!(
				Defi::deposit(RuntimeOrigin::signed(ALICE), balance!(101)),
				Error::<Runtime>::InsufficientBalance
			);
		});
	}

	#[test]
	fn deposit_new_user_ok() {
		let mut ext = ExtBuilder::default().build();
		ext.execute_with(|| {
			// Start test from block 1
			run_to_block(1);

			// Check balances before deposit
			assert_eq!(pallet_balances::Pallet::<Runtime>::free_balance(ALICE), balance!(100));
			assert_eq!(pallet_balances::Pallet::<Runtime>::free_balance(&pallet_id()), balance!(0));

			// Check ALICE lending/borrowing position before deposit
			let alice_info = pallet::Accounts::<Runtime>::get(ALICE);
			assert_eq!(alice_info.deposit_principal, balance!(0));
			assert_eq!(alice_info.deposit_date, 0);
			assert_eq!(alice_info.borrow_principal, balance!(0));
			assert_eq!(alice_info.borrow_date, 0);

			assert_ok!(Defi::deposit(RuntimeOrigin::signed(ALICE), balance!(10)));

			// Check balances after deposit
			assert_eq!(pallet_balances::Pallet::<Runtime>::free_balance(ALICE), balance!(90));
			assert_eq!(
				pallet_balances::Pallet::<Runtime>::free_balance(&pallet_id()),
				balance!(10)
			);

			// Check ALICE lending/borrowing position after deposit
			let alice_info = pallet::Accounts::<Runtime>::get(ALICE);
			assert_eq!(alice_info.deposit_principal, balance!(10));
			assert_eq!(alice_info.deposit_date, 1);
			assert_eq!(alice_info.borrow_principal, balance!(0));
			assert_eq!(alice_info.borrow_date, 0);
		});
	}

	#[test]
	fn deposit_old_user_ok() {
		let mut ext = ExtBuilder::default().build();
		ext.execute_with(|| {
			// Start test from block 1
			run_to_block(1);

			// Check balances before first deposit
			assert_eq!(pallet_balances::Pallet::<Runtime>::free_balance(ALICE), balance!(100));
			assert_eq!(pallet_balances::Pallet::<Runtime>::free_balance(&pallet_id()), balance!(0));

			// Check ALICE lending/borrowing position before first deposit
			let alice_info = pallet::Accounts::<Runtime>::get(ALICE);
			assert_eq!(alice_info.deposit_principal, balance!(0));
			assert_eq!(alice_info.deposit_date, 0);
			assert_eq!(alice_info.borrow_principal, balance!(0));
			assert_eq!(alice_info.borrow_date, 0);

			assert_ok!(Defi::deposit(RuntimeOrigin::signed(ALICE), balance!(10)));

			// Check ALICE lending/borrowing position after first deposit
			let alice_info = pallet::Accounts::<Runtime>::get(ALICE);
			assert_eq!(alice_info.deposit_principal, balance!(10));
			assert_eq!(alice_info.deposit_date, 1);
			assert_eq!(alice_info.borrow_principal, balance!(0));
			assert_eq!(alice_info.borrow_date, 0);

			// Check balances after first deposit
			assert_eq!(pallet_balances::Pallet::<Runtime>::free_balance(ALICE), balance!(90));
			assert_eq!(
				pallet_balances::Pallet::<Runtime>::free_balance(&pallet_id()),
				balance!(10)
			);

			// Run blockchain to block 11
			run_to_block(11);

			// Check balances before second deposit
			assert_eq!(pallet_balances::Pallet::<Runtime>::free_balance(ALICE), balance!(90));
			assert_eq!(
				pallet_balances::Pallet::<Runtime>::free_balance(&pallet_id()),
				balance!(10)
			);

			// Check ALICE lending/borrowing position before second deposit
			let alice_info = pallet::Accounts::<Runtime>::get(ALICE);
			assert_eq!(alice_info.deposit_principal, balance!(10));
			assert_eq!(alice_info.deposit_date, 1);
			assert_eq!(alice_info.borrow_principal, balance!(0));
			assert_eq!(alice_info.borrow_date, 0);

			assert_ok!(Defi::deposit(RuntimeOrigin::signed(ALICE), balance!(10)));

			// Check balances after second deposit
			assert_eq!(pallet_balances::Pallet::<Runtime>::free_balance(ALICE), balance!(80));
			assert_eq!(
				pallet_balances::Pallet::<Runtime>::free_balance(&pallet_id()),
				balance!(20)
			);

			// Check ALICE lending/borrowing position before second deposit
			let alice_info = pallet::Accounts::<Runtime>::get(ALICE);
			assert_eq!(alice_info.deposit_principal, balance!(20));
			assert_eq!(alice_info.deposit_date, 11);
			assert_eq!(alice_info.borrow_principal, balance!(0));
			assert_eq!(alice_info.borrow_date, 0);
		});
	}

	#[test]
	fn borrow_pallet_has_not_enough_funds() {
		let mut ext = ExtBuilder::default().build();
		ext.execute_with(|| {
			// Start test from block 1
			run_to_block(1);

			assert_err!(
				Defi::borrow(RuntimeOrigin::signed(ALICE), balance!(10)),
				Error::<Runtime>::PalletHasNotEnoughFunds
			);
		});
	}

	#[test]
	fn borrow_unallowed_borrow_amount() {
		let mut ext = ExtBuilder::default().build();
		ext.execute_with(|| {
			// Start test from block 1
			run_to_block(1);

			assert_ok!(Defi::deposit(RuntimeOrigin::signed(CHARLIE), balance!(99)));

			assert_err!(
				Defi::borrow(RuntimeOrigin::signed(ALICE), balance!(10)),
				Error::<Runtime>::UnallowedBorrowAmount
			);
		});
	}

	#[test]
	fn borrow_new_user_ok() {
		let mut ext = ExtBuilder::default().build();
		ext.execute_with(|| {
			// Start test from block 1
			run_to_block(1);

			// Execute prerequired deposits
			assert_ok!(Defi::deposit(RuntimeOrigin::signed(CHARLIE), balance!(99)));
			assert_ok!(Defi::deposit(RuntimeOrigin::signed(ALICE), balance!(10)));

			// Check balances before borrow
			assert_eq!(pallet_balances::Pallet::<Runtime>::free_balance(ALICE), balance!(90));
			assert_eq!(
				pallet_balances::Pallet::<Runtime>::free_balance(&pallet_id()),
				balance!(109)
			);

			// Check ALICE lending/borrowing position before borrow
			let alice_info = pallet::Accounts::<Runtime>::get(ALICE);
			assert_eq!(alice_info.deposit_principal, balance!(10));
			assert_eq!(alice_info.deposit_date, 1);
			assert_eq!(alice_info.borrow_principal, balance!(0));
			assert_eq!(alice_info.borrow_date, 0);

			assert_ok!(Defi::borrow(RuntimeOrigin::signed(ALICE), balance!(5)));

			// Check balances after borrow
			assert_eq!(pallet_balances::Pallet::<Runtime>::free_balance(ALICE), balance!(95));
			assert_eq!(
				pallet_balances::Pallet::<Runtime>::free_balance(&pallet_id()),
				balance!(104)
			);

			// Check ALICE lending/borrowing position after borrow
			let alice_info = pallet::Accounts::<Runtime>::get(ALICE);
			assert_eq!(alice_info.deposit_principal, balance!(10));
			assert_eq!(alice_info.deposit_date, 1);
			assert_eq!(alice_info.borrow_principal, balance!(5));
			assert_eq!(alice_info.borrow_date, 1);
		});
	}

	#[test]
	fn borrow_old_user_ok() {
		let mut ext = ExtBuilder::default().build();
		ext.execute_with(|| {
			// Start test from block 1
			run_to_block(1);

			// Execute prerequired deposits
			assert_ok!(Defi::deposit(RuntimeOrigin::signed(CHARLIE), balance!(99)));
			assert_ok!(Defi::deposit(RuntimeOrigin::signed(ALICE), balance!(10)));

			// Check balances before first borrow
			assert_eq!(pallet_balances::Pallet::<Runtime>::free_balance(ALICE), balance!(90));
			assert_eq!(
				pallet_balances::Pallet::<Runtime>::free_balance(&pallet_id()),
				balance!(109)
			);

			// Check ALICE lending/borrowing position before first borrow
			let alice_info = pallet::Accounts::<Runtime>::get(ALICE);
			assert_eq!(alice_info.deposit_principal, balance!(10));
			assert_eq!(alice_info.deposit_date, 1);
			assert_eq!(alice_info.borrow_principal, balance!(0));
			assert_eq!(alice_info.borrow_date, 0);

			assert_ok!(Defi::borrow(RuntimeOrigin::signed(ALICE), balance!(5)));

			// Check balances after borrow
			assert_eq!(pallet_balances::Pallet::<Runtime>::free_balance(ALICE), balance!(95));
			assert_eq!(
				pallet_balances::Pallet::<Runtime>::free_balance(&pallet_id()),
				balance!(104)
			);

			// Check ALICE lending/borrowing position after first borrow
			let alice_info = pallet::Accounts::<Runtime>::get(ALICE);
			assert_eq!(alice_info.deposit_principal, balance!(10));
			assert_eq!(alice_info.deposit_date, 1);
			assert_eq!(alice_info.borrow_principal, balance!(5));
			assert_eq!(alice_info.borrow_date, 1);

			// Run blockchain to block 11
			run_to_block(11);

			// Check balances before first borrow
			assert_eq!(pallet_balances::Pallet::<Runtime>::free_balance(ALICE), balance!(95));
			assert_eq!(
				pallet_balances::Pallet::<Runtime>::free_balance(&pallet_id()),
				balance!(104)
			);

			// Check ALICE lending/borrowing position before first borrow
			let alice_info = pallet::Accounts::<Runtime>::get(ALICE);
			assert_eq!(alice_info.deposit_principal, balance!(10));
			assert_eq!(alice_info.deposit_date, 1);
			assert_eq!(alice_info.borrow_principal, balance!(5));
			assert_eq!(alice_info.borrow_date, 1);

			assert_ok!(Defi::borrow(RuntimeOrigin::signed(ALICE), balance!(1)));

			// Check balances after borrow
			assert_eq!(pallet_balances::Pallet::<Runtime>::free_balance(ALICE), balance!(96));
			assert_eq!(
				pallet_balances::Pallet::<Runtime>::free_balance(&pallet_id()),
				balance!(103)
			);

			// Check ALICE lending/borrowing position after first borrow
			let calculated_interest = get_debt_amount(ALICE);

			let alice_info = pallet::Accounts::<Runtime>::get(ALICE);
			assert_eq!(alice_info.deposit_principal, balance!(10));
			assert_eq!(alice_info.deposit_date, 1);
			assert_eq!(alice_info.borrow_principal, calculated_interest);
			assert_eq!(alice_info.borrow_date, 11);
		});
	}

	#[test]
	fn withdraw_no_funds_deposited() {
		let mut ext = ExtBuilder::default().build();
		ext.execute_with(|| {
			// Start test from block 1
			run_to_block(1);

			assert_err!(
				Defi::withdraw(RuntimeOrigin::signed(ALICE), balance!(10)),
				Error::<Runtime>::NoFundsDeposited
			);
		});
	}

	#[test]
	fn withdraw_user_has_not_enough_funds() {
		let mut ext = ExtBuilder::default().build();
		ext.execute_with(|| {
			// Start test from block 1
			run_to_block(1);

			// Execute prerequired deposits
			assert_ok!(Defi::deposit(RuntimeOrigin::signed(ALICE), balance!(10)));

			assert_err!(
				Defi::withdraw(RuntimeOrigin::signed(ALICE), balance!(11)),
				Error::<Runtime>::UserHasNotEnoughFunds
			);
		});
	}

	#[test]
	fn withdraw_pallet_has_not_enough_funds() {
		let mut ext = ExtBuilder::default().build();
		ext.execute_with(|| {
			// Start test from block 1
			run_to_block(1);

			// Execute prerequired deposit and transaction
			assert_ok!(Defi::deposit(RuntimeOrigin::signed(ALICE), balance!(10)));
			assert_ok!(pallet_balances::Pallet::<Runtime>::transfer_keep_alive(
				RuntimeOrigin::signed(pallet_id()),
				BOB,
				balance!(5)
			));

			assert_err!(
				Defi::withdraw(RuntimeOrigin::signed(ALICE), balance!(6)),
				Error::<Runtime>::PalletHasNotEnoughFunds
			);
		});
	}

	#[test]
	fn withdraw_ok() {
		let mut ext = ExtBuilder::default().build();
		ext.execute_with(|| {
			// Start test from block 1
			run_to_block(1);

			// Execute prerequired deposit
			assert_ok!(Defi::deposit(RuntimeOrigin::signed(ALICE), balance!(10)));

			// Check balances before withdrawl
			assert_eq!(pallet_balances::Pallet::<Runtime>::free_balance(ALICE), balance!(90));
			assert_eq!(
				pallet_balances::Pallet::<Runtime>::free_balance(&pallet_id()),
				balance!(10)
			);

			// Check ALICE lending/borrowing position before withdrawl
			let alice_info = pallet::Accounts::<Runtime>::get(ALICE);
			assert_eq!(alice_info.deposit_principal, balance!(10));
			assert_eq!(alice_info.deposit_date, 1);
			assert_eq!(alice_info.borrow_principal, balance!(0));
			assert_eq!(alice_info.borrow_date, 0);

			// Run blockchain to block 11
			run_to_block(11);

			assert_ok!(Defi::withdraw(RuntimeOrigin::signed(ALICE), balance!(9)));

			// Check balances after withdrawl
			assert_eq!(pallet_balances::Pallet::<Runtime>::free_balance(ALICE), balance!(99));
			assert_eq!(pallet_balances::Pallet::<Runtime>::free_balance(&pallet_id()), balance!(1));

			// Check ALICE lending/borrowing position after withdrawl
			let alice_info = pallet::Accounts::<Runtime>::get(ALICE);
			assert_eq!(alice_info.deposit_principal, balance!(1));
			assert_eq!(alice_info.deposit_date, 11);
			assert_eq!(alice_info.borrow_principal, balance!(0));
			assert_eq!(alice_info.borrow_date, 0);
		});
	}

	#[test]
	fn repay_insufficient_balance() {
		let mut ext = ExtBuilder::default().build();
		ext.execute_with(|| {
			// Start test from block 1
			run_to_block(1);

			// Execute prerequired deposit
			assert_err!(
				Defi::repay(RuntimeOrigin::signed(ALICE), balance!(110)),
				Error::<Runtime>::InsufficientBalance
			);
		});
	}

	#[test]
	fn repay_nothing_to_repay() {
		let mut ext = ExtBuilder::default().build();
		ext.execute_with(|| {
			// Start test from block 1
			run_to_block(1);

			// Execute prerequired deposit
			assert_err!(
				Defi::repay(RuntimeOrigin::signed(ALICE), balance!(10)),
				Error::<Runtime>::NothingToRepay
			);
		});
	}

	#[test]
	fn repay_with_leftover_interest_ok() {
		let mut ext = ExtBuilder::default().build();
		ext.execute_with(|| {
			// Start test from block 1
			run_to_block(1);

			// Execute prerequired deposits and withdrawls
			assert_ok!(Defi::deposit(RuntimeOrigin::signed(ALICE), balance!(10)));
			assert_ok!(Defi::borrow(RuntimeOrigin::signed(ALICE), balance!(5)));

			// Run blockchain to block 11
			run_to_block(11);

			// Check balances before repay
			assert_eq!(pallet_balances::Pallet::<Runtime>::free_balance(ALICE), balance!(95));
			assert_eq!(pallet_balances::Pallet::<Runtime>::free_balance(&pallet_id()), balance!(5));

			// Check ALICE lending/borrowing position before repay
			let alice_info = pallet::Accounts::<Runtime>::get(ALICE);
			assert_eq!(alice_info.deposit_principal, balance!(10));
			assert_eq!(alice_info.deposit_date, 1);
			assert_eq!(alice_info.borrow_principal, balance!(5));
			assert_eq!(alice_info.borrow_date, 1);

			assert_ok!(Defi::repay(RuntimeOrigin::signed(ALICE), balance!(5)));

			// Check balances after repay
			assert_eq!(pallet_balances::Pallet::<Runtime>::free_balance(ALICE), balance!(90));
			assert_eq!(
				pallet_balances::Pallet::<Runtime>::free_balance(&pallet_id()),
				balance!(10)
			);

			// Check ALICE lending/borrowing position after repay
			let calculated_interest = get_debt_amount(ALICE) - 8285;

			let alice_info = pallet::Accounts::<Runtime>::get(ALICE);
			assert_eq!(alice_info.deposit_principal, balance!(10));
			assert_eq!(alice_info.deposit_date, 1);
			assert_eq!(alice_info.borrow_principal, calculated_interest + 8285);
			assert_eq!(alice_info.borrow_date, 11);
		});
	}

	#[test]
	fn repay_full_amount_ok() {
		let mut ext = ExtBuilder::default().build();
		ext.execute_with(|| {
			// Start test from block 1
			run_to_block(1);

			// Execute prerequired deposits and withdrawls
			assert_ok!(Defi::deposit(RuntimeOrigin::signed(ALICE), balance!(10)));
			assert_ok!(Defi::borrow(RuntimeOrigin::signed(ALICE), balance!(5)));

			// Run blockchain to block 11
			run_to_block(11);

			// Check balances before repay
			assert_eq!(pallet_balances::Pallet::<Runtime>::free_balance(ALICE), balance!(95));
			assert_eq!(pallet_balances::Pallet::<Runtime>::free_balance(&pallet_id()), balance!(5));

			// Check ALICE lending/borrowing position before repay
			let alice_info = pallet::Accounts::<Runtime>::get(ALICE);
			assert_eq!(alice_info.deposit_principal, balance!(10));
			assert_eq!(alice_info.deposit_date, 1);
			assert_eq!(alice_info.borrow_principal, balance!(5));
			assert_eq!(alice_info.borrow_date, 1);

			let calculated_interest = get_debt_amount(ALICE);
			assert_ok!(Defi::repay(RuntimeOrigin::signed(ALICE), calculated_interest));

			// Check balances after repay
			assert_eq!(
				pallet_balances::Pallet::<Runtime>::free_balance(ALICE),
				balance!(95) - calculated_interest
			);
			assert_eq!(
				pallet_balances::Pallet::<Runtime>::free_balance(&pallet_id()),
				balance!(5) + calculated_interest
			);

			// Check ALICE lending/borrowing position after repay
			let alice_info = pallet::Accounts::<Runtime>::get(ALICE);
			assert_eq!(alice_info.deposit_principal, balance!(10));
			assert_eq!(alice_info.deposit_date, 1);
			assert_eq!(alice_info.borrow_principal, balance!(0));
			assert_eq!(alice_info.borrow_date, 0);
		});
	}

	#[test]
	fn update_deposit_rate_unauthorized_user_action() {
		let mut ext = ExtBuilder::default().build();
		ext.execute_with(|| {
			// Start test from block 1
			run_to_block(1);

			assert_err!(
				Defi::update_deposit_rate(RuntimeOrigin::signed(ALICE), FixedU128::from_inner(1)),
				Error::<Runtime>::UnauthorizedUserAction
			);
		});
	}

	#[test]
	fn update_deposit_rate_ok() {
		let mut ext = ExtBuilder::default().build();
		ext.execute_with(|| {
			// Start test from block 1
			run_to_block(1);

			// Check deposit rate before update
			assert_eq!(pallet::DepositRate::<Runtime>::get(), get_default_deposit_rate(),);

			assert_ok!(Defi::update_deposit_rate(
				RuntimeOrigin::signed(get_authority_account()),
				FixedU128::from_inner(1)
			),);

			// Check deposit rate after update
			assert_eq!(pallet::DepositRate::<Runtime>::get(), FixedU128::from_inner(1),);
		});
	}

	#[test]
	fn update_borrowing_rate_unauthorized_user_action() {
		let mut ext = ExtBuilder::default().build();
		ext.execute_with(|| {
			// Start test from block 1
			run_to_block(1);

			assert_err!(
				Defi::update_borrowing_rate(RuntimeOrigin::signed(ALICE), FixedU128::from_inner(1)),
				Error::<Runtime>::UnauthorizedUserAction
			);
		});
	}

	#[test]
	fn update_borrowing_rate_ok() {
		let mut ext = ExtBuilder::default().build();
		ext.execute_with(|| {
			// Start test from block 1
			run_to_block(1);

			// Check borrowing rate before update
			assert_eq!(pallet::BorrowingRate::<Runtime>::get(), get_default_borrowing_rate(),);

			assert_ok!(Defi::update_borrowing_rate(
				RuntimeOrigin::signed(get_authority_account()),
				FixedU128::from_inner(1)
			),);

			// Check borrowing rate after update
			assert_eq!(pallet::BorrowingRate::<Runtime>::get(), FixedU128::from_inner(1),);
		});
	}

	#[test]
	fn update_collateral_factor_unauthorized_user_action() {
		let mut ext = ExtBuilder::default().build();
		ext.execute_with(|| {
			// Start test from block 1
			run_to_block(1);

			assert_err!(
				Defi::update_collateral_factor(
					RuntimeOrigin::signed(ALICE),
					FixedU128::from_inner(1)
				),
				Error::<Runtime>::UnauthorizedUserAction
			);
		});
	}

	#[test]
	fn update_collateral_factor_ok() {
		let mut ext = ExtBuilder::default().build();
		ext.execute_with(|| {
			// Start test from block 1
			run_to_block(1);

			// Check collateral factor before update
			assert_eq!(pallet::CollateralFactor::<Runtime>::get(), get_default_collateral_factor(),);

			assert_ok!(Defi::update_collateral_factor(
				RuntimeOrigin::signed(get_authority_account()),
				FixedU128::from_inner(1)
			),);

			// Check collateral factor after update
			assert_eq!(pallet::CollateralFactor::<Runtime>::get(), FixedU128::from_inner(1),);
		});
	}

	#[test]
	fn check_liquidate_user_position_ok() {
		let mut ext = ExtBuilder::default().build();
		ext.execute_with(|| {
			// Start test from block 1
			run_to_block(1);

			// Execute prerequired deposits and borrows
			assert_ok!(Defi::deposit(RuntimeOrigin::signed(ALICE), balance!(10)));
			assert_ok!(Defi::borrow(RuntimeOrigin::signed(ALICE), balance!(5)));

			// Change borrowing rate for faster liquidation
			assert_ok!(Defi::update_borrowing_rate(
				RuntimeOrigin::signed(get_authority_account()),
				FixedU128::from_inner(balance!(1))
			),);

			// Run blockchain to block 100
			run_to_block(100);

			// Check if ALICE position before liquidation
			let alice_info = pallet::Accounts::<Runtime>::get(ALICE);
			assert_eq!(alice_info.deposit_principal, balance!(10));
			assert_eq!(alice_info.deposit_date, 1);
			assert_eq!(alice_info.borrow_principal, balance!(5));
			assert_eq!(alice_info.borrow_date, 1);

			assert_ok!(Defi::liquidate_user_position(RuntimeOrigin::signed(BOB), ALICE));

			// Check if ALICE position after liquidation
			let alice_info = pallet::Accounts::<Runtime>::get(ALICE);
			assert_eq!(alice_info.deposit_principal, balance!(0));
			assert_eq!(alice_info.deposit_date, 0);
			assert_eq!(alice_info.borrow_principal, balance!(0));
			assert_eq!(alice_info.borrow_date, 0);
		});
	}
}
