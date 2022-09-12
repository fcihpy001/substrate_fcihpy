#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::pallet_prelude::*;
	use frame_support::traits::{Currency, ExistenceRequirement, Randomness, ReservableCurrency};

	use frame_system::pallet_prelude::*;
	use sp_io::hashing::blake2_128;
	use sp_runtime::traits::{AtLeast32BitUnsigned, Bounded, CheckedAdd};

	///接口配置
	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type Randomness: Randomness<Self::Hash, Self::BlockNumber>;
		//质押资产类型
		type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;
		// Parameter 可用于函数参数传递
		// AtLeast32BitUnsigned 转换为u32不会造成数据丢失
		// Copy  实现了copy方法
		// Default  表示有默认址
		// Bounded  包含上下边界
		//MaxEncodedLen  最大编码长
		type KittyIndex: Parameter 
			+ AtLeast32BitUnsigned 
			+ Default 
			+ Copy 
			+ Bounded
			+ MaxEncodedLen;
		// 定义操作前抵押的资产数量
		#[pallet::constant]
		type KittyStake: Get<BalanceOf<Self>>;

		#[pallet::constant]
		type MaxKittyIndex: Get<u32>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, TypeInfo, MaxEncodedLen)]
	pub struct Kitty {
		pub dna: [u8; 16],
	}

	#[pallet::type_value]
	pub fn GetDefaultValue<T: Config>() -> T::KittyIndex {
		0_u8.into()
	}
	//账户余额
	type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	//最新kitty id
	#[pallet::storage]
	#[pallet::getter(fn last_kitty_id)]
	pub type LastKittyId<T: Config> =
		StorageValue<_, T::KittyIndex, ValueQuery, GetDefaultValue<T>>;

	//存储kitty 详情
	#[pallet::storage]
	#[pallet::getter(fn kitties_info)]
	pub type Kitties<T: Config> = StorageMap<_, Blake2_128Concat, T::KittyIndex, Kitty>;

	// 存储kitty与所有者的对应关系
	#[pallet::storage]
	#[pallet::getter(fn kitty_owner)]
	pub type KittyOwner<T: Config> = StorageMap<_, Blake2_128Concat, T::KittyIndex, T::AccountId>;

	// 存储正在销售的kittyid 及价格
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
		pub fn create(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
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
			let sender = ensure_signed(origin)?;
			// 验证父母是不是同一个kitty
			ensure!(kitty_id_1 != kitty_id_2, Error::<T>::SameKittyId);

			// 检查kitty_id是否存在且有效
			let kitty_1 = Self::kitty_of_id(kitty_id_1).map_err(|_| Error::<T>::InvalidKittyId)?;
			let kitty_2 = Self::kitty_of_id(kitty_id_2).map_err(|_| Error::<T>::InvalidKittyId)?;
			let dna_1 = kitty_1.dna;
			let dna_2 = kitty_2.dna;

			// 生成一个随机数，混淆父母的dna,作为子kitty的独有基因
			let selector = Self::random_value(&sender);

			// 通过把父母的基因与子kitty的独有基因进行位与、位或，得到子Kitty的完整基因
			let mut new_dna = [0u8; 16];
			for i in 0..dna_1.len() {
				new_dna[i] = (dna_1[i] & selector[i]) | (dna_2[i] & !selector[i]);
			}
			// 质押并创建一个新kitty
			Self::new_kitty_with_stake(&sender, new_dna)?;
			Ok(().into())
		}

		#[pallet::weight(10_000)]
		pub fn transfer(
			origin: OriginFor<T>,
			kitty_id: T::KittyIndex,
			new_owner: T::AccountId,
		) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;
			// 根据ID获取kiitty
			Self::kitty_of_id(kitty_id).map_err(|_| Error::<T>::InvalidKittyId)?;
			// 验证拥有者是否为当前操作者
			ensure!(Self::kitty_owner(kitty_id) == Some(sender.clone()), Error::<T>::NotOwner);

			// 获取操作要质押的数量，并开始质押
			let stake_amount = T::KittyStake::get();
			T::Currency::reserve(&new_owner, stake_amount)
				.map_err(|_| Error::<T>::NotEnoughBalance)?;
			// 原有拥有者解除质押
			T::Currency::unreserve(&sender, stake_amount);
			// 保存新的拥有者关系
			<KittyOwner<T>>::insert(&kitty_id, &new_owner);

			Self::deposit_event(Event::KittyTransferred(sender, new_owner, kitty_id));
			Ok(().into())
		}

		#[pallet::weight(1_000)]
		pub fn sell(
			origin: OriginFor<T>,
			kitty_id: T::KittyIndex,
			price: Option<BalanceOf<T>>,
		) -> DispatchResultWithPostInfo {
			let seller = ensure_signed(origin)?;
			// 验证操作者是否为拥有者
			ensure!(Self::kitty_owner(kitty_id) == Some(seller.clone()), Error::<T>::NotOwner);
			// 给kitty设定价格，并保存关联有关系
			KittiesShop::<T>::mutate_exists(kitty_id, |p| *p = Some(price));

			Self::deposit_event(Event::KittyInSell(seller, kitty_id, price));
			Ok(().into())
		}

		#[pallet::weight(1_000)]
		pub fn buy(origin: OriginFor<T>, kitty_id: T::KittyIndex) -> DispatchResultWithPostInfo {
			let buyer = ensure_signed(origin)?;
			// 根据ID获取kitty所有者
			let seller = KittyOwner::<T>::get(kitty_id).ok_or(Error::<T>::InvalidKittyId)?;
			// 验证购买者是否为拥有者
			ensure!(Some(buyer.clone()) != Some(seller.clone()), Error::<T>::NoBuySelf);
			// 获取kitty价格
			let price = KittiesShop::<T>::get(kitty_id).ok_or(Error::<T>::NotForSale)?;
			// 获取买家账户余额
			let buyer_balance = T::Currency::free_balance(&buyer);
			// 获取需要质押的金额配置
			let stake_amount = T::KittyStake::get();
			// 检查买家的余额是否足够用于购买和质押
			ensure!(buyer_balance > (price + stake_amount), Error::<T>::NotEnoughBalance);
			// 获取要质押的数量
			let stake_amount = T::KittyStake::get();
			// 买家质押指定的资产数量
			T::Currency::reserve(&buyer, stake_amount).map_err(|_| Error::<T>::NotEnoughBalance)?;
			// 卖家解除质押数量
			T::Currency::unreserve(&seller, stake_amount);
			// 买家支付相应价格的token数给卖家
			T::Currency::transfer(&buyer, &seller, price, ExistenceRequirement::KeepAlive)?;
			// 更新kitty所有者
			KittyOwner::<T>::insert(kitty_id, buyer.clone());
			// 通告事件
			Self::deposit_event(Event::KittyTransferred(seller, buyer, kitty_id));
			Ok(().into())
		}
	}

	impl<T: Config> Pallet<T> {
		// 获取一个256位的随机数 用作Kitty的DNA
		fn random_value(sender: &T::AccountId) -> [u8; 16] {
			let payload = (
				T::Randomness::random_seed(),
				&sender,
				<frame_system::Pallet<T>>::extrinsic_index(), //获取当前交易在区块中的index，相当于nonce
			);
			payload.using_encoded(blake2_128)
		}
		// 根据ID获取kitty
		fn kitty_of_id(kitty_id: T::KittyIndex) -> Result<Kitty, ()> {
			match Self::kitties_info(kitty_id) {
				Some(kitty) => Ok(kitty),
				None => Err(())
			}
		}

		// 质押并创建Kitty
		fn new_kitty_with_stake(
			sender: &T::AccountId,
			dna: [u8; 16],
		) -> DispatchResultWithPostInfo {
			// 获取要质量的资产数量
			let stake_amount = T::KittyStake::get();

			// 开始质押指定数量的资产，失败则报余额不跢的错误
			T::Currency::reserve(&sender, stake_amount)
				.map_err(|_| Error::<T>::NotEnoughBalance)?;

			// 获取最后一个kittyid，并自增加1
			let kitty_id = Self::last_kitty_id();
			let kitty_id = kitty_id
				.checked_add(&(T::KittyIndex::from(1_u8)))
				.ok_or(Error::<T>::KittyIdOverflow)
				.unwrap();

			// 生成新kitty实例
			let kitty = Kitty { dna };

			//保存数据
			// 保存kitty实例与kittyid的对应关系
			Kitties::<T>::insert(kitty_id, &kitty);
			// 保存kittyid与所有者之间的对应关系
			KittyOwner::<T>::insert(kitty_id, &sender);
			// 保存最后一个kittyid
			LastKittyId::<T>::set(kitty_id);

			//通报事件
			Self::deposit_event(Event::KittyCreated(sender.clone(), kitty_id));

			Ok(().into())
		}
	}
}
