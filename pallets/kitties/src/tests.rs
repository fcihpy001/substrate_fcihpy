use super::*;
use crate::mock::{new_test_ext, Event as TestEvent, KittiesModule, Origin, System, Test};
use frame_support::{assert_noop, assert_ok};

/**************************Create_test*******************/
#[test]
fn create_success() {
	new_test_ext().execute_with(|| {
		let account_id: u64 = 1;
		let kitty_id = 0u32;
		// 创建Kitty
		assert_ok!(KittiesModule::create(Origin::signed(account_id)));
		// 检查拥有者
		assert_eq!(KittyOwner::<Test>::get(kitty_id), Some(account_id));
		// 检查创建成功事件
		assert_has_event!(Event::<Test>::KittyCreatedAndBreed(account_id, kitty_id));
	});
}

#[test]
fn test_kitty_next_id() {
	new_test_ext().execute_with(|| {
		let account_id: u64 = 1;
		// kitty_id 从0开始
		// 创建Kitty
		assert_ok!(KittiesModule::create(Origin::signed(account_id)));
		assert_ok!(KittiesModule::create(Origin::signed(account_id)));
		assert_eq!(KittiesModule::next_kitty_id(), 2);
	})
}

#[test]
// fn test_get_id_kitty() {
// 	new_test_ext().execute_with(|| {
// 		let account_id: u64 = 1;
// 		let kitty_id = 0u32;
// 		// 创建Kitty
// 		let kitty = KittiesModule::create(Origin::signed(account_id));
// 		assert_eq!(KittiesModule::kitties_info(kitty_id), kitty);
// 		// assert_eq!(Kitties::<Test>::get(kitty_id), None);
// 	})
// }

#[test]
fn test_stack_count() {
	new_test_ext().execute_with(|| {
		// todo! 如何读取配置文件中的参数值
		// assert_eq!(KittiesModule::KittyStake::get(),200);
		// assert_eq!(200,200);
	})
}

#[test]
fn create_failed_not_enough_balance() {
	new_test_ext().execute_with(|| {
		let account_id: u64 = 3;
		assert_noop!(
			KittiesModule::create(Origin::signed(account_id)),
			Error::<Test>::NotEnoughBalance
		);
	});
}

#[test]
fn create_failed_overflow_id() {
	new_test_ext().execute_with(|| {
		NextKittyId::<Test>::put(u32::max_value());
		let account_id: u64 = 1;
		assert_noop!(
			KittiesModule::create(Origin::signed(account_id)),
			Error::<Test>::KittyIdOverflow
		);
	});
}

/**************************Breed_test*******************/
#[test]
fn breed_success() {
	new_test_ext().execute_with(|| {
		let account_id: u64 = 1;

		let kitty_id_1 = 0u32;
		let kitty_id_2 = 1u32;
		let kitty_id_3: u32 = 2u32;

		// 创建Kitty
		assert_ok!(KittiesModule::create(Origin::signed(account_id)));
		assert_ok!(KittiesModule::create(Origin::signed(account_id)));
		// 繁殖
		assert_ok!(KittiesModule::breed(Origin::signed(account_id), kitty_id_1, kitty_id_2));
		// 检查拥有者
		assert_eq!(KittyOwner::<Test>::get(kitty_id_3), Some(account_id));
		// 检查创建成功事件
		assert_has_event!(Event::<Test>::KittyCreatedAndBreed(account_id, kitty_id_3));
	});
}

#[test]
fn breed_failed_not_enough_balance() {
	new_test_ext().execute_with(|| {
		let account_id: u64 = 4;
		let kitty_id_1 = 0u32;
		let kitty_id_2 = 1u32;
		// 创建Kitty
		assert_ok!(KittiesModule::create(Origin::signed(account_id)));
		assert_ok!(KittiesModule::create(Origin::signed(account_id)));
		assert_ok!(KittiesModule::create(Origin::signed(account_id)));
		// 繁殖
		assert_noop!(
			KittiesModule::breed(Origin::signed(account_id), kitty_id_1, kitty_id_2),
			Error::<Test>::NotEnoughBalance
		);
	});
}

#[test]
fn breed_failed_same_parent() {
	new_test_ext().execute_with(|| {
		let account_id: u64 = 1;
		let kitty_id_1 = 1u32;
		// 创建Kitty
		assert_ok!(KittiesModule::create(Origin::signed(account_id)));
		// 繁殖
		assert_noop!(
			KittiesModule::breed(Origin::signed(account_id), kitty_id_1, kitty_id_1),
			Error::<Test>::SameParentId
		);
	});
}

/**************************Transfer_test*******************/
#[test]
fn transfer_success() {
	new_test_ext().execute_with(|| {
		let account_id_1: u64 = 1;
		let account_id_2: u64 = 2;
		let kitty_id = 0u32;
		// 创建Kitty
		assert_ok!(KittiesModule::create(Origin::signed(account_id_1)));
		// 转让
		assert_ok!(KittiesModule::transfer(Origin::signed(account_id_1), kitty_id, account_id_2));
		// 检查创建成功事件
		assert_has_event!(Event::<Test>::KittyTransferred(account_id_1, account_id_2, kitty_id));
	});
}

#[test]
fn transfer_failed_buyer_not_enough_balance() {
	new_test_ext().execute_with(|| {
		let account_id_1: u64 = 1;
		let account_id_5: u64 = 5;
		let kitty_id = 0u32;
		// 创建Kitty
		assert_ok!(KittiesModule::create(Origin::signed(account_id_1)));
		assert_noop!(
			KittiesModule::transfer(Origin::signed(account_id_1), kitty_id, account_id_5),
			Error::<Test>::NotEnoughBalance
		);
	});
}

#[test]
fn transfer_failed_not_owner() {
	new_test_ext().execute_with(|| {
		let account_id_1: u64 = 1;
		let account_id_2: u64 = 2;
		let account_id_3: u64 = 3;
		let kitty_id = 1u32;
		// 创建Kitty
		assert_ok!(KittiesModule::create(Origin::signed(account_id_1)));
		assert_noop!(
			KittiesModule::transfer(Origin::signed(account_id_2), kitty_id, account_id_3),
			Error::<Test>::NotOwner
		);
	});
}
// // #[test]
// // fn transfer_failed_invalid_id() {
// //     new_test_ext().execute_with(|| {
// //         let account_id: u64 = 1;
// //         let kitty_id = 1u32;
// //         // 创建Kitty
// //         assert_ok!(KittiesModule::create(Origin::signed(account_id)));
// //         // 检查拥有者
// //         assert_eq!(KittyOwner::<Test>::get(kitty_id), Some(account_id));
// //         // 检查创建成功事件
// //         assert_has_event!(Event::<Test>::KittyCreated(account_id, kitty_id));
// //     });
// // }
/**************************Sell_test*******************/
#[test]
fn sell_success() {
	new_test_ext().execute_with(|| {
		let account_id: u64 = 1;
		let kitty_id = 1u32;
		let price = 100u128;

		// 创建Kitty
		assert_ok!(KittiesModule::create(Origin::signed(account_id)));
		//设定出售的价格
		assert_ok!(KittiesModule::sell(Origin::signed(account_id), kitty_id, Some(price)));
		// 检查拥有者
		assert_eq!(KittiesShop::<Test>::get(kitty_id), Some(price));
		// 检查上架成功事件
		assert_has_event!(Event::<Test>::KittyInSell(account_id, kitty_id, Some(price)));
	});
}

#[test]
fn sell_failed_not_owner() {
	new_test_ext().execute_with(|| {
		let account_id: u64 = 1;
		let othere_account_id: u64 = 100;
		let kitty_id = 1u32;
		let price = 100u128;
		// 创建Kitty
		assert_ok!(KittiesModule::create(Origin::signed(account_id)));
		assert_noop!(
			KittiesModule::sell(Origin::signed(account_id), kitty_id, Some(price)),
			Error::<Test>::NotOwner
		);
	});
}

// // #[test]
// // fn sell_failed_not_sale() {
// //     new_test_ext().execute_with(|| {
// //         let account_id: u64 = 1;
// //         let kitty_id = 1u32;
// //         // 创建Kitty
// //         assert_ok!(KittiesModule::create(Origin::signed(account_id)));
// //         // 检查拥有者
// //         assert_eq!(KittyOwner::<Test>::get(kitty_id), Some(account_id));
// //         // 检查创建成功事件
// //         assert_has_event!(Event::<Test>::KittyCreated(account_id, kitty_id));
// //     });
// // }

// // #[test]
// // fn sell_failed_self() {
// //     new_test_ext().execute_with(|| {
// //         let account_id: u64 = 1;
// //         let kitty_id = 1u32;
// //         // 创建Kitty
// //         assert_ok!(KittiesModule::create(Origin::signed(account_id)));
// //         // 检查拥有者
// //         assert_eq!(KittyOwner::<Test>::get(kitty_id), Some(account_id));
// //         // 检查创建成功事件
// //         assert_has_event!(Event::<Test>::KittyCreated(account_id, kitty_id));
// //     });
// // }

/**************************Buy_test*******************/
#[test]
fn buy_success() {
	new_test_ext().execute_with(|| {
		let account_id_1: u64 = 1;
		let account_id_2: u64 = 2;
		let kitty_id = 1u32;
		let price = 1000u128;
		// 创建Kitty
		assert_ok!(KittiesModule::create(Origin::signed(account_id_1)));
		// 检查拥有者
		assert_eq!(KittyOwner::<Test>::get(kitty_id), Some(account_id_1));

		assert_ok!(KittiesModule::sell(Origin::signed(account_id_1), kitty_id, Some(price)));

		assert_ok!(KittiesModule::sell(Origin::signed(account_id_1), kitty_id, Some(price)));

		// 检查拥有者
		assert_eq!(KittyOwner::<Test>::get(kitty_id), Some(account_id_2));
		// 检查创建成功事件
		assert_has_event!(Event::<Test>::KittySold(account_id_1, account_id_2, kitty_id));
	});
}

// // #[test]
// // fn buy_failed_not_enough_balance() {
// //     new_test_ext().execute_with(|| {
// //         let account_id: u64 = 1;
// //         let kitty_id = 1u32;
// //         // 创建Kitty
// //         assert_ok!(KittiesModule::create(Origin::signed(account_id)));
// //         // 检查拥有者
// //         assert_eq!(KittyOwner::<Test>::get(kitty_id), Some(account_id));
// //         // 检查创建成功事件
// //         assert_has_event!(Event::<Test>::KittyCreated(account_id, kitty_id));
// //     });
// // }
