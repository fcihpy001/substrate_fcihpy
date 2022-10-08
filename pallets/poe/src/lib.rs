#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod test;

#[cfg(test)]
mod mock;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

pub mod weights;

pub use pallet::*;
pub use weights::WeightInfo;
#[frame_support::pallet]
pub mod pallet {
	use crate::weights::WeightInfo;
	pub use frame_support::pallet_prelude::*;
	use frame_support::{ensure, pallet_prelude::DispatchResultWithPostInfo};
	use frame_system::ensure_signed;
	pub use frame_system::pallet_prelude::*;
	pub use sp_std::prelude::*;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		#[pallet::constant]
		type MaxClaimLength: Get<u32>;
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type WeightInfo: WeightInfo;
	}

	#[pallet::storage]
	pub type Proofs<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		BoundedVec<u8, T::MaxClaimLength>,
		(T::AccountId, T::BlockNumber),
	>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		ClaimCreated(T::AccountId, Vec<u8>),
		ClaimRevoked(T::AccountId, Vec<u8>),
		ClaimTransfered(T::AccountId, Vec<u8>),
	}

	#[pallet::error]
	pub enum Error<T> {
		ProofAlreadyExist,
		ClaimTooLong,
		ClaimNotExist,
		NotClaimOwner,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(T::WeightInfo::create_claim(claim.len() as u32))]
		pub fn create_claim(origin: OriginFor<T>, claim: Vec<u8>) -> DispatchResultWithPostInfo {
			// 验证操作者签名信息
			let sender = ensure_signed(origin)?;

			// 检查凭证是否超出最大限度，
			let bounded_claim = BoundedVec::<u8, T::MaxClaimLength>::try_from(claim.clone())
				.map_err(|_| Error::<T>::ClaimTooLong)?;

			// 检查便凭证是否已经存在，不存在则提示错误
			ensure!(!Proofs::<T>::contains_key(&bounded_claim), Error::<T>::ProofAlreadyExist);

			// 获取当前区块号
			let current_block = frame_system::Pallet::<T>::block_number();

			// 向链上存数据
			Proofs::<T>::insert(&bounded_claim, (sender.clone(), current_block));

			// 发布事件
			Self::deposit_event(Event::ClaimCreated(sender, claim));
			Ok(().into())
		}

		#[pallet::weight(T::WeightInfo::revoke_claim(claim.len() as u32))]
		pub fn revoke_claim(origin: OriginFor<T>, claim: Vec<u8>) -> DispatchResultWithPostInfo {
			// 验证操作者权限
			let sender = ensure_signed(origin)?;

			// 检查凭证是否超出最大限度，
			let bounded_claim = BoundedVec::<u8, T::MaxClaimLength>::try_from(claim.clone())
				.map_err(|_| Error::<T>::ClaimTooLong)?;

			//获取存证者所有,如果没有返回数据，则证明是凭证没有存储过，也就不能删除
			let (owner, _) = Proofs::<T>::get(&bounded_claim).ok_or(Error::<T>::ClaimNotExist)?;

			//校验操作者，是否是凭证的所有者
			ensure!(owner == sender, Error::<T>::NotClaimOwner);

			//删除存证项
			Proofs::<T>::remove(&bounded_claim);

			//发布事件
			Self::deposit_event(Event::ClaimRevoked(sender, claim));
			Ok(().into())
		}

		#[pallet::weight(T::WeightInfo::transfer_claim(claim.len() as u32))]
		pub fn transfer_claim(
			origin: OriginFor<T>,
			dest: T::AccountId,
			claim: Vec<u8>,
		) -> DispatchResultWithPostInfo {
			// 验证操作者权限
			let sender = ensure_signed(origin)?;
			let reciver = dest;

			// 检查凭证是否超出最大限度，
			let bounded_claim = BoundedVec::<u8, T::MaxClaimLength>::try_from(claim.clone())
				.map_err(|_| Error::<T>::ClaimTooLong)?;

			//获取存证者所有,如果没有返回数据
			let (owner, _) = Proofs::<T>::get(&bounded_claim).ok_or(Error::<T>::ClaimNotExist)?;

			//校验操作者，是否是凭证的所有者
			ensure!(owner == sender, Error::<T>::NotClaimOwner);

			//删除存证项
			Proofs::<T>::remove(&bounded_claim);

			// 获取当前区块号
			let current_block = frame_system::Pallet::<T>::block_number();

			// 向链上存数据
			Proofs::<T>::insert(&bounded_claim, (reciver.clone(), current_block));

			//发布事件
			Self::deposit_event(Event::ClaimTransfered(reciver, claim));

			Ok(().into())
		}
	}
}
