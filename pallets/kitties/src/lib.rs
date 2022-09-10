#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;
//
#[cfg(test)]
mod tests;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::pallet_prelude::*;
	use frame_support::traits::{Currency, ExistenceRequirement, Randomness, ReservableCurrency};

	use frame_system::pallet_prelude::*;
	use sp_io::hashing::blake2_128; // 引入哈希包
	use sp_runtime::traits::{AtLeast32BitUnsigned, Bounded, One}; // 引入

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type Randomness: Randomness<Self::Hash, Self::BlockNumber>;
		type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;
		type KittyIndex: Parameter + AtLeast32BitUnsigned + Default + Copy + Bounded + MaxEncodedLen;
		#[pallet::constant]
		type KittyStake: Get<BalanceOf<Self>>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, TypeInfo, MaxEncodedLen)]
	pub struct Kitty(pub [u8; 16]);

	type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	#[pallet::storage]
	#[pallet::getter(fn next_kitty_id)]
	pub type NextKittyId<T: Config> = StorageValue<_, T::KittyIndex, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn kitties)]
	pub type Kitties<T: Config> = StorageMap<_, Blake2_128Concat, T::KittyIndex, Kitty>;

	#[pallet::storage]
	#[pallet::getter(fn kitty_owner)]
	pub type KittyOwner<T: Config> = StorageMap<_, Blake2_128Concat, T::KittyIndex, T::AccountId>;

	#[pallet::storage]
	#[pallet::getter(fn kitties_list_for_sales)]
	pub type KittiesShop<T: Config> =
		StorageMap<_, Blake2_128Concat, T::KittyIndex, Option<BalanceOf<T>>, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		KittyCreated(T::AccountId, T::KittyIndex),
		KittyBred(T::AccountId, T::KittyIndex, Kitty),
		KittyTransferred(T::AccountId, T::AccountId, T::KittyIndex),
		KittyInSell(T::AccountId, T::KittyIndex, Option<BalanceOf<T>>),
	}

	#[pallet::error]
	pub enum Error<T> {
		InvalidKittyId,
		KittyIdOverflow,
		NotOwner,
		SameKittyId,
		NoBuySelf,
		NotForSale,
		NotEnoughBalance,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(10_000)]
		/// 创建Kitties
		pub fn create(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
			// 验证当前操作者账户
			let sender = ensure_signed(origin)?;

			let dna = Self::random_value(&sender);

			Self::new_kitty_with_stake(&sender, dna)?;

			Ok(().into())
		}

		#[pallet::weight(10_000)]
		pub fn breed(
			origin: OriginFor<T>,
			kitty_id_1: T::KittyIndex,
			kitty_id_2: T::KittyIndex,
		) -> DispatchResultWithPostInfo {
			// 获取当前操作者账户
			let sender = ensure_signed(origin)?;

			// 检查父母不能是同一个kitty
			ensure!(kitty_id_1 != kitty_id_2, Error::<T>::SameKittyId);

			// 检查kitty_id是否存在且有效
			let kitty_1 = Self::get_kitty(kitty_id_1).map_err(|_| Error::<T>::InvalidKittyId)?;
			let kitty_2 = Self::get_kitty(kitty_id_2).map_err(|_| Error::<T>::InvalidKittyId)?;

			let dna_1 = kitty_1.0;
			let dna_2 = kitty_2.0;

			// 生成一个随机数，作为子kitty的独有基因
			let selector = Self::random_value(&sender);

			// 通过把父母的基因与子kitty的独有基因进行位与、位或，得到子Kitty的完整基因
			let mut new_dna = [0u8; 16];
			for i in 0..dna_1.len() {
				new_dna[i] = (dna_1[i] & selector[i]) | (dna_2[i] & !selector[i]);
			}
			Self::new_kitty_with_stake(&sender, new_dna)?;

			// 返回OK
			Ok(().into())
		}

		#[pallet::weight(10_000)]
		pub fn transfer(
			origin: OriginFor<T>,
			kitty_id: T::KittyIndex,
			new_owner: T::AccountId,
		) -> DispatchResultWithPostInfo {
			// 获取当前操作者账户
			let sender = ensure_signed(origin)?;

			// 检查kitty_id是否有效
			Self::get_kitty(kitty_id).map_err(|_| Error::<T>::InvalidKittyId)?;

			// 检查是否为kitty的owner
			ensure!(Self::kitty_owner(kitty_id) == Some(sender.clone()), Error::<T>::NotOwner);

			// 获取需要质押的金额
			let stake_amount = T::KittyStake::get();

			// 新的Owner账户进行质押
			T::Currency::reserve(&new_owner, stake_amount)
				.map_err(|_| Error::<T>::NotEnoughBalance)?;

			// 旧的Owner账户解除质押
			T::Currency::unreserve(&sender, stake_amount);

			// 保存kitty的新owner 更新也使用insert，即重新插入一条新记录覆盖原来的老数据
			<KittyOwner<T>>::insert(&kitty_id, &new_owner);

			// 触发事件
			Self::deposit_event(Event::KittyTransferred(sender, new_owner, kitty_id));

			// 返回OK
			Ok(().into())
		}

		#[pallet::weight(1_000)]
		pub fn sell(
			origin: OriginFor<T>,
			kitty_id: T::KittyIndex,
			price: Option<BalanceOf<T>>,
		) -> DispatchResultWithPostInfo {
			// 获取当前操作者账户
			let seller = ensure_signed(origin)?;

			// 检查是否为kitty的owner
			ensure!(Self::kitty_owner(kitty_id) == Some(seller.clone()), Error::<T>::NotOwner);

			// 给指定Kitty报价并上架到店铺
			KittiesShop::<T>::mutate_exists(kitty_id, |p| *p = Some(price));

			// 触发出售事件
			Self::deposit_event(Event::KittyInSell(seller, kitty_id, price));

			Ok(().into())
		}

		#[pallet::weight(1_000)]
		pub fn buy(origin: OriginFor<T>, kitty_id: T::KittyIndex) -> DispatchResultWithPostInfo {
			// 获取买家账户
			let buyer = ensure_signed(origin)?;

			// 获取卖家账户，即Kitty的Owner
			let seller = KittyOwner::<T>::get(kitty_id).ok_or(Error::<T>::InvalidKittyId)?;

			// 检查买卖双方是否为同一个人
			ensure!(Some(buyer.clone()) != Some(seller.clone()), Error::<T>::NoBuySelf);

			// 获取指定Kitty的报价，如果报价为None，则该Kitty为非卖品
			let price = KittiesShop::<T>::get(kitty_id).ok_or(Error::<T>::NotForSale)?;

			// 获取买家的账户余额
			let buyer_balance = T::Currency::free_balance(&buyer);

			// 获取需要质押的金额配置
			let stake_amount = T::KittyStake::get();

			// 检查买家的余额是否足够用于购买和质押
			ensure!(buyer_balance > (price + stake_amount), Error::<T>::NotEnoughBalance);

			// 买家质押
			T::Currency::reserve(&buyer, stake_amount).map_err(|_| Error::<T>::NotEnoughBalance)?;

			// 卖家解除质押
			T::Currency::unreserve(&seller, stake_amount);

			// 买家支付token给卖家
			T::Currency::transfer(&buyer, &seller, price, ExistenceRequirement::KeepAlive)?;

			// 将Kitty从店铺下架 删除使用remove
			KittiesShop::<T>::remove(kitty_id);

			// 更新Kitty归属买家
			KittyOwner::<T>::insert(kitty_id, buyer.clone());

			// 触发转移事件
			Self::deposit_event(Event::KittyTransferred(seller, buyer, kitty_id));

			Ok(().into())
		}
	}

	/// 定义pallet的公共函数
	impl<T: Config> Pallet<T> {
		// 获取一个256位的随机数 用作Kitty的DNA
		fn random_value(sender: &T::AccountId) -> [u8; 16] {
			let payload = (
				T::Randomness::random_seed(), // 随机值，保证dna的唯一性
				&sender,
				<frame_system::Pallet<T>>::extrinsic_index(), // //获取当前交易在区块中的index，相当于nonce
			);

			payload.using_encoded(blake2_128) // 对payload进行Scale编码，这里需要引入use sp_io::hashing::blake2_128;
		}

		// 通过id查询Kitty
		fn get_kitty(kitty_id: T::KittyIndex) -> Result<Kitty, ()> {
			match Self::kitties(kitty_id) {
				Some(kitty) => Ok(kitty),
				None => Err(()),
			}
		}

		// 质押并创建Kitty
		fn new_kitty_with_stake(
			sender: &T::AccountId,
			dna: [u8; 16],
		) -> DispatchResultWithPostInfo {
			// 获取需要质押的金额
			let stake_amount = T::KittyStake::get();

			// 质押指定数量的资产，如果资产质押失败则报错
			T::Currency::reserve(&sender, stake_amount)
				.map_err(|_| Error::<T>::NotEnoughBalance)?;

			let kitty_id = Self::next_kitty_id();
			if kitty_id == T::KittyIndex::max_value() {
				return Err(Error::<T>::KittyIdOverflow.into());
			}

			let kitty = Kitty(dna);

			// 保存数据
			Kitties::<T>::insert(kitty_id, &kitty); // 保存kitty信息
			KittyOwner::<T>::insert(kitty_id, &sender); // 保存kitty的owner
			NextKittyId::<T>::set(kitty_id + One::one()); // kitty_id+1

			// 触发事件
			Self::deposit_event(Event::KittyCreated(sender.clone(), kitty_id));

			// 返回OK
			Ok(().into())
		}
	}
}
