use super::*;
use crate::{mock::*, Error};
use frame_support::{assert_err, assert_noop, assert_ok, BoundedVec};

#[test]
fn creat_claim_success_test() {
	new_test_ext().execute_with(|| {
		let claim = vec![0, 1];
		assert_ok!(PoeModule::create_claim(Origin::signed(1), claim.clone()));
		let bound_claim =
			BoundedVec::<u8, <Test as Config>::MaxClaimLength>::try_from(claim.clone()).unwrap();

		assert_eq!(
			Proofs::<Test>::get(&bound_claim),
			Some((1, frame_system::Pallet::<Test>::block_number()))
		)
	})
}
fn creat_claim_success_test_template() {
	new_test_ext().execute_with(|| {
		//snip
		// 测试功能的代码
	})
}

#[test]
fn creat_claim_failed_when_claim_arealdy_exist() {
	new_test_ext().execute_with(|| {
		let claim = vec![0, 1];
		PoeModule::create_claim(Origin::signed(1), claim.clone());

		assert_noop!(
			PoeModule::create_claim(Origin::signed(1), claim.clone()),
			Error::<Test>::ProofAlreadyExist
		);
	})
}
#[test]
fn creat_claim_failed_when_claim_toolong() {
	new_test_ext().execute_with(|| {
		let claim = vec![0, 1, 3, 4, 4];

		assert_noop!(
			PoeModule::create_claim(Origin::signed(1), claim.clone()),
			Error::<Test>::ClaimTooLong
		);
	})
}
#[test]
fn revoke_claim_success() {
	new_test_ext().execute_with(|| {
		let claim = vec![0, 1];
		PoeModule::create_claim(Origin::signed(1), claim.clone());

		let bound_claim =
			BoundedVec::<u8, <Test as Config>::MaxClaimLength>::try_from(claim.clone()).unwrap();
		PoeModule::revoke_claim(Origin::signed(1), claim.clone());
		assert_eq!(Proofs::<Test>::get(&bound_claim), None)
	})
}
#[test]
fn revoke_claim_failed_when_claim_notexist() {
	new_test_ext().execute_with(|| {
		let claim = vec![0, 1];

		PoeModule::revoke_claim(Origin::signed(1), claim.clone());
		assert_err!(
			PoeModule::revoke_claim(Origin::signed(2), claim.clone()),
			Error::<Test>::ClaimNotExist
		)
	})
}
#[test]
fn revoke_claim_failed_when_claim_not_owner() {
	new_test_ext().execute_with(|| {
		let claim = vec![0, 1];
		PoeModule::create_claim(Origin::signed(1), claim.clone());
		PoeModule::revoke_claim(Origin::signed(2), claim.clone());
		assert_noop!(
			PoeModule::revoke_claim(Origin::signed(2), claim.clone()),
			Error::<Test>::NotClaimOwner
		);
	})
}

#[test]
fn transfer_claim_success() {
	new_test_ext().execute_with(|| {
		let claim = vec![0, 1];
		PoeModule::create_claim(Origin::signed(1), claim.clone());

		PoeModule::transfer_claim(Origin::signed(1), 666, claim.clone());
		let bound_claim =
			BoundedVec::<u8, <Test as Config>::MaxClaimLength>::try_from(claim.clone()).unwrap();
		assert_eq!(
			Proofs::<Test>::get(&bound_claim),
			Some((666, frame_system::Pallet::<Test>::block_number()))
		)
	})
}
#[test]
fn transfer_claim_failed_when_not_owner() {
	new_test_ext().execute_with(|| {
		let claim = vec![0, 1];
		PoeModule::create_claim(Origin::signed(1), claim.clone());

		let bound_claim =
			BoundedVec::<u8, <Test as Config>::MaxClaimLength>::try_from(claim.clone()).unwrap();
		assert_noop!(
			PoeModule::transfer_claim(Origin::signed(2), 666, claim.clone()),
			Error::<Test>::NotClaimOwner
		);
	})
}
