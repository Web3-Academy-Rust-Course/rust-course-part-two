mod tests {
	use crate::{mock::*, pallet, Error};
	use frame_support::{
		assert_err, assert_ok,
		sp_runtime::{FixedU128},
	};

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
}