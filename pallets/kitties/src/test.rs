use super::*;
use crate::mock::{
    new_test_ext, Event as TestEvent, Origin, SubstrateKitties as Kitties, System, Test,
};
use frame_support::{assert_noop, assert_ok};

#[test]
fn create_works() {
    new_test_ext().execute_with(|| {
        let accound_id: u64 = 1;
        let kitty_id = 1u32;
        // 创建Kitty
        assert_ok!(Kitties::create(Origin::signed(accound_id)));
        // 检查拥有者
        assert_eq!(Owner::<Test>::get(kitty_id), Some(accound_id));
        // 检查创建成功事件
        assert_has_event!(Event::<Test>::KittyCreate(accound_id, kitty_id));
    });
}

#[test]
fn create_kitty_failed_count_overflow() {
    new_test_ext().execute_with(|| {
        KittiesCount::<Test>::put(u32::max_value());
        let account_id = 1;
        // Kitty的ID已经达到最大值
        assert_noop!(
			Kitties::create(Origin::signed(account_id)),
			Error::<Test>::KittiesCountOverflow
		);
    });
}

#[test]
fn create_kitty_failed_not_enough_balance_for_staking() {
    new_test_ext().execute_with(|| {
        // 质押金额不足
        let account_id: u64 = 100;
        assert_noop!(
			Kitties::create(Origin::signed(account_id)),
			Error::<Test>::NotEnoughBalanceForStaking
		);
    });
}

#[test]
fn breed_works() {
    new_test_ext().execute_with(|| {
        let accound_id: u64 = 1;
        // parent kitty
        let kitty_id_1 = 1u32;
        let kitty_id_2 = 2u32;
        // child kitty
        let kitty_id_3: u32 = 3u32;
        // 创建Kitty1
        assert_ok!(Kitties::create(Origin::signed(accound_id)));
        // 创建Kitty2
        assert_ok!(Kitties::create(Origin::signed(accound_id)));
        // 繁殖
        assert_ok!(Kitties::breed(Origin::signed(accound_id), kitty_id_1, kitty_id_2));
        // 创建成功事件
        assert_has_event!(Event::<Test>::KittyCreate(accound_id, kitty_id_3));
    });
}

#[test]
fn breed_failed_same_parent() {
    new_test_ext().execute_with(|| {
        let accound_id: u64 = 1;
        // parent kitty
        let kitty_id_1 = 1u32;
        // 创建Kitty1
        assert_ok!(Kitties::create(Origin::signed(accound_id)));
        // 繁殖时父母ID相同
        assert_noop!(
			Kitties::breed(Origin::signed(accound_id), kitty_id_1, kitty_id_1),
			Error::<Test>::SameParentIndex
		);
    });
}

#[test]
fn breed_failed_invalid_kitty_index() {
    new_test_ext().execute_with(|| {
        let accound_id: u64 = 1;
        // parent kitty
        let kitty_id_1 = 1u32;
        let kitty_id_2 = 2u32;
        // 繁殖时没有找到Id对应的Kitty
        assert_noop!(
			Kitties::breed(Origin::signed(accound_id), kitty_id_1, kitty_id_2),
			Error::<Test>::InvalidKittyIndex
		);
    });
}

#[test]
fn breed_failed_count_overflow() {
    new_test_ext().execute_with(|| {
        KittiesCount::<Test>::put(u32::max_value() - 2);
        let accound_id: u64 = 1;
        // parent kitty
        let kitty_id_1 = u32::max_value() - 1;
        let kitty_id_2 = u32::max_value() - 2;
        // 创建Kitty1
        assert_ok!(Kitties::create(Origin::signed(accound_id)));
        // 创建Kitty2
        assert_ok!(Kitties::create(Origin::signed(accound_id)));
        // 繁殖时Kitty的ID已经达到最大值
        assert_noop!(
			Kitties::breed(Origin::signed(accound_id), kitty_id_1, kitty_id_2),
			Error::<Test>::KittiesCountOverflow
		);
    });
}
#[test]
fn breed_failed_not_enough_balance_for_staking() {
    new_test_ext().execute_with(|| {
        let accound_id: u64 = 1;
        let other_account_id: u64 = 100;
        // parent kitty
        let kitty_id_1 = 1u32;
        let kitty_id_2 = 2u32;
        // 创建Kitty1
        assert_ok!(Kitties::create(Origin::signed(accound_id)));
        // 创建Kitty2
        assert_ok!(Kitties::create(Origin::signed(accound_id)));
        // 繁殖但token不足
        assert_noop!(
			Kitties::breed(Origin::signed(other_account_id), kitty_id_1, kitty_id_2),
			Error::<Test>::NotEnoughBalanceForStaking
		);
    });
}

#[test]
fn sell_works() {
    new_test_ext().execute_with(|| {
        let account_id: u64 = 1;
        let kitty_id = 1u32;
        let price = 100u128;
        assert_ok!(Kitties::create(Origin::signed(account_id)));
        assert_ok!(Kitties::sell(Origin::signed(account_id), kitty_id, Some(price)));
        // 代售列表中的价格是否一致
        assert_eq!(ListForSale::<Test>::get(kitty_id), Some(price));
        // 卖出事件
        assert_has_event!(Event::<Test>::KittyListed(account_id, kitty_id, Some(price)));
    });
}

#[test]
fn sell_failed_not_owner() {
    new_test_ext().execute_with(|| {
        let accound_id: u64 = 1;
        let other_account_id: u64 = 100;
        let kitty_id = 1u32;
        let price = 100u128;
        assert_ok!(Kitties::create(Origin::signed(accound_id)));
        assert_noop!(
			Kitties::sell(Origin::signed(other_account_id), kitty_id, Some(price)),
			Error::<Test>::NotOwner
		);
    });
}

#[test]
fn transfer_works() {
    new_test_ext().execute_with(|| {
        let account_id_1: u64 = 1;
        let account_id_2: u64 = 2;
        let kitty_id = 1u32;
        // 创建Kitty
        assert_ok!(Kitties::create(Origin::signed(account_id_1)));
        // 转让Kitty
        assert_ok!(Kitties::transfer(Origin::signed(account_id_1), account_id_2, kitty_id));
        // 转让事件
        assert_has_event!(Event::KittyTransfer(account_id_1, account_id_2, kitty_id));
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
        assert_ok!(Kitties::create(Origin::signed(account_id_1)));
        // 不是自己的Kitty
        assert_noop!(
			Kitties::transfer(Origin::signed(account_id_2), account_id_3, kitty_id),
			Error::<Test>::NotOwner
		);
    });
}

#[test]
fn transfer_failed_not_enough_balance_for_staking() {
    new_test_ext().execute_with(|| {
        let account_id_1: u64 = 1;
        let account_id_5: u64 = 5;
        let kitty_id = 1u32;
        // 创建Kitty
        assert_ok!(Kitties::create(Origin::signed(account_id_1)));
        // 余额不足无法质押
        assert_noop!(
			Kitties::transfer(Origin::signed(account_id_1), account_id_5, kitty_id),
			Error::<Test>::NotEnoughBalanceForStaking
		);
    });
}

#[test]
fn buy_works() {
    new_test_ext().execute_with(|| {
        let account_id_1: u64 = 1;
        let account_id_2: u64 = 2;
        let kitty_id = 1u32;
        let price: u128 = 1000;
        // 创建Kitty
        assert_ok!(Kitties::create(Origin::signed(account_id_1)));
        // 卖出Kitty
        assert_ok!(Kitties::sell(Origin::signed(account_id_1), kitty_id, Some(price)));
        // 购买Kitty
        assert_ok!(Kitties::buy(Origin::signed(account_id_2), kitty_id));
        // 购买事件
        assert_has_event!(Event::KittySold(account_id_2, account_id_1, kitty_id));
    });
}

#[test]
fn buy_failed_buyer_is_owner() {
    new_test_ext().execute_with(|| {
        let account_id_1: u64 = 1;
        let kitty_id = 1u32;
        let price: u128 = 1000;
        // 创建Kitty
        assert_ok!(Kitties::create(Origin::signed(account_id_1)));
        // 卖出Kitty
        assert_ok!(Kitties::sell(Origin::signed(account_id_1), kitty_id, Some(price)));
        // 购买者和拥有者是同一人
        assert_noop!(
			Kitties::buy(Origin::signed(account_id_1), kitty_id),
			Error::<Test>::BuyerIsOwner
		);
    });
}

#[test]
fn buy_failed_not_for_sell() {
    new_test_ext().execute_with(|| {
        let account_id_1: u64 = 1;
        let account_id_2: u64 = 2;
        let kitty_id = 1u32;
        // 创建Kitty
        assert_ok!(Kitties::create(Origin::signed(account_id_1)));
        // Kitty未放入代售列表
        assert_noop!(
			Kitties::buy(Origin::signed(account_id_2), kitty_id),
			Error::<Test>::KittyNotForSell
		);
    });
}

#[test]
fn buy_failed_buyer_not_enough_balance_for_buying() {
    new_test_ext().execute_with(|| {
        let account_id_1: u64 = 1;
        let account_id_3: u64 = 3;
        let kitty_id = 1u32;
        let price: u128 = 100000;
        // 创建Kitty
        assert_ok!(Kitties::create(Origin::signed(account_id_1)));
        // 卖出Kitty
        assert_ok!(Kitties::sell(Origin::signed(account_id_1), kitty_id, Some(price)));
        // 买家余额不够
        assert_noop!(
			Kitties::buy(Origin::signed(account_id_3), kitty_id),
			Error::<Test>::NotEnoughBalanceForBuying
		);
    });
}