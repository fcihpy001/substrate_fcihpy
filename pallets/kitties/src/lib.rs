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
	use sp_io::hashing::blake2_128;
	use sp_runtime::traits::{AtLeast32BitUnsigned, Bounded, One,CheckedAdd};

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type Randomness: Randomness<Self::Hash, Self::BlockNumber>;
		type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;
		type KittyIndex: Parameter + AtLeast32BitUnsigned + Default + Copy + Bounded + MaxEncodedLen;
		#[pallet::constant]
		type KittyStake: Get<BalanceOf<Self>>;
		#[pallet::constant]
		type MaxKittyIndex: Get<u32>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, TypeInfo, MaxEncodedLen)]
	pub struct Kitty(pub [u8; 16]);

	#[pallet::type_value]
	pub fn GetDefaultValue(T: Config) -> T::KittyIndex {
		0_u8.into()
	}
	type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	#[pallet::storage]
	#[pallet::getter(fn next_kitty_id)]
	pub type NextKittyId<T: Config> = StorageValue<
		_,
		T::KittyIndex,
		ValueQuery,
		GetDefaultValue<T>>;

	#[pallet::storage]
	#[pallet::getter(fn kitties)]
	pub type Kitties<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::KittyIndex,
		Kitty>;

	#[pallet::storage]
	#[pallet::getter(fn kitty_owner)]
	pub type KittyOwner<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::KittyIndex,
		T::AccountId>;

	#[pallet::storage]
	#[pallet::getter(fn kitties_list_for_sales)]
	pub type KittiesShop<T: Config> =
		StorageMap<
			_,
			Blake2_128Concat,
			T::KittyIndex,
			Option<BalanceOf<T>>,
			ValueQuery>;

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
			Self::new_kitty_with_stake(&sender,dna)?;
			Ok(().into())
		}

		#[pallet::weight(10_000)]
		pub fn transfer(
			origin: OriginFor<T>,
			kitty_id: T::KittyIndex,
			new_owner: T::AccountId,
		) -> DispatchResultWithPostInfo {

			let sender = ensure_signed(origin)?;

			Self::get_kitty(kitty_id).map_err(|_| Error::<T>::InvalidKittyId)?;

			ensure!(Self::kitty_owner(kitty_id) == Some(sender.clone()), Error::<T>::NotOwner);

			let stake_amount = T::KittyStake::get();
			T::Currency::reserve(&new_owner, stake_amount)
				.map_err(|_| Error::<T>::NotEnoughBalance)?;
			T::Currency::unreserve(&sender, stake_amount);

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
			ensure!(Self::kitty_owner(kitty_id) == Some(seller.clone()), Error::<T>::NotOwner);

			KittiesShop::<T>::mutate_exists(kitty_id, |p| *p = Some(price));

			Self::deposit_event(Event::KittyInSell(seller, kitty_id, price));
			Ok(().into())
		}

		#[pallet::weight(1_000)]
		pub fn buy(origin: OriginFor<T>, kitty_id: T::KittyIndex) -> DispatchResultWithPostInfo {

			let buyer = ensure_signed(origin)?;
			let seller = KittyOwner::<T>::get(kitty_id).ok_or(Error::<T>::InvalidKittyId)?;
			ensure!(Some(buyer.clone()) != Some(seller.clone()), Error::<T>::NoBuySelf);

			let price = KittiesShop::<T>::get(kitty_id).ok_or(Error::<T>::NotForSale)?;
			let buyer_balance = T::Currency::free_balance(&buyer);
			let stake_amount = T::KittyStake::get();

			ensure!(buyer_balance > (price + stake_amount), Error::<T>::NotEnoughBalance);
			T::Currency::reserve(&buyer, stake_amount).map_err(|_| Error::<T>::NotEnoughBalance)?;
			T::Currency::unreserve(&seller, stake_amount);
			T::Currency::transfer(&buyer, &seller, price, ExistenceRequirement::KeepAlive)?;

			KittiesShop::<T>::remove(kitty_id);
			KittyOwner::<T>::insert(kitty_id, buyer.clone());

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

		fn get_kitty(kitty_id: T::KittyIndex) -> Result<Kitty, ()> {
			match Self::kitties(kitty_id) {
				Some(kitty) => Ok(kitty),
				None => Err(()),
			}
		}

		fn get_next_id() -> Result<T::KittyIndex, ()> {
			let kitty_id = Self::next_kitty_id();
			match kitty_id {
				_ if T::KittyIndex::max_value() <= kitty_id => Err(()),
				value => Ok(val)
			}
		}

		// 质押并创建Kitty
		fn new_kitty_with_stake(
			sender: &T::AccountId,
			dna: [u8; 16]
		) -> DispatchResultWithPostInfo {

			let stake_amount = T::KittyStake::get();

			T::Currency::reserve(&sender, stake_amount)
				.map_err(|_| Error::<T>::NotEnoughBalance)?;

			let kitty_id = Self::get_next_id()
				.map_err(|_| Error::<T>::KittyIdOverflow)?;

			let kitty = Kitty(dna);

			Kitties::<T>::insert(kitty_id, &kitty);
			KittyOwner::<T>::insert(kitty_id, &sender);
			let next_kitty_id = kitty_id
				.checked_add(&(T::KittyIndex::from(1_u8)))
				.ok_or(Error::<T>::KittyIdOverflow)
				.unwrap();
			NextKittyId::<T>::set(next_kitty_id);

			Self::deposit_event(Event::KittyCreated(sender.clone(), kitty_id));

			Ok(().into())
		}
	}
}
