#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{
		dispatch::fmt,
		inherent::Vec,
		pallet_prelude::{DispatchResult, ValueQuery, *},
		traits::{Randomness, Time},
	};
	use frame_system::pallet_prelude::{OriginFor, *};

	type Dna = Vec<u8>;

	#[pallet::pallet]
	#[pallet::without_storage_info]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[derive(Clone, Encode, Decode, PartialEq, TypeInfo)]
	#[scale_info(skip_type_params(T))]
	pub struct Kitty<T: Config> {
		dna: Vec<u8>,
		owner: T::AccountId,
		price: u32,
		gender: Gender,
		created_date: <<T as Config>::Time as Time>::Moment,
	}

	impl<T: Config> fmt::Debug for Kitty<T> {
		fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
			f.debug_struct("Kitty")
				.field("dna", &self.dna)
				.field("owner", &self.owner)
				.field("price", &self.price)
				.field("gender", &self.gender)
				.field("created_date", &self.created_date)
				.finish()
		}
	}

	#[derive(Encode, Decode, Debug, Clone, PartialEq, TypeInfo)]
	pub enum Gender {
		Male,
		Female,
	}

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		type KittyRandomness: Randomness<Self::Hash, Self::BlockNumber>;
		type Time: Time;

		#[pallet::constant]
		type MaxKitty: Get<u32>;
	}

	#[pallet::storage]
	#[pallet::getter(fn total_kitties)]
	pub type TotalKitties<T> = StorageValue<_, u64, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn nonce)]
	pub type Nonce<T> = StorageValue<_, u64, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn kitty_info)]
	pub type KittyInfo<T: Config> = StorageMap<_, Blake2_128Concat, T::Hash, Kitty<T>, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn kitty_owner)]
	pub type KittyOwner<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, BoundedVec<Dna, T::MaxKitty>, OptionQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		CreatedNew(T::AccountId, Vec<u8>),
		Transferred(T::AccountId, T::AccountId, Vec<u8>),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		NotFound,
		NotOwner,
		MaxKitty,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(100)]
		pub fn create_kitty(origin: OriginFor<T>) -> DispatchResult {
			let owner = ensure_signed(origin)?;
			let dna_hash = Self::random_hash();
			let dna = dna_hash.as_ref().to_vec();
			let new_kitty = Kitty {
				dna: dna.clone(),
				owner: owner.clone(),
				price: 0u8.into(),
				gender: Kitty::<T>::gender(dna.clone()),
				created_date: T::Time::now(),
			};

			log::info!("{:?}", new_kitty);

			TotalKitties::<T>::set(Self::total_kitties() + 1);
			KittyInfo::<T>::insert(dna_hash, new_kitty);

			let mut kitties = Self::kitty_owner(owner.clone()).unwrap_or_default();
			kitties.try_push(dna.clone()).unwrap();
			KittyOwner::<T>::insert(owner.clone(), kitties);

			Self::deposit_event(Event::CreatedNew(owner, dna));

			Ok(())
		}

		#[pallet::weight(100)]
		pub fn transfer(
			origin: OriginFor<T>,
			receiver: T::AccountId,
			dna: T::Hash,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			let mut kitty = Self::kitty_info(dna.clone()).ok_or(Error::<T>::NotFound)?;

			if kitty.owner != sender {
				return Err(Error::<T>::NotOwner.into());
			}

			kitty.owner = receiver.clone();
			KittyInfo::<T>::insert(dna.clone(), kitty);

			let dna = dna.as_ref().to_vec();

			let mut sender_kitties = Self::kitty_owner(sender.clone()).unwrap_or_default();
			let index = sender_kitties.iter().position(|x| *x == dna.clone()).unwrap();
			sender_kitties.remove(index);
			KittyOwner::<T>::insert(sender.clone(), sender_kitties);

			let mut receiver_kitties = Self::kitty_owner(receiver.clone()).unwrap_or_default();
			receiver_kitties.try_push(dna.clone()).err();
			KittyOwner::<T>::insert(receiver.clone(), receiver_kitties);

			Self::deposit_event(Event::Transferred(sender, receiver, dna));

			Ok(())
		}
	}

	impl<T: Config> Kitty<T> {
		pub fn gender(dna: Vec<u8>) -> Gender {
			if dna.len() % 2 == 0 {
				Gender::Male
			} else {
				Gender::Female
			}
		}
	}

	impl<T: Config> Pallet<T> {
		fn get_and_increment_nonce() -> Vec<u8> {
			let nonce = Nonce::<T>::get();
			Nonce::<T>::put(nonce.wrapping_add(1));
			nonce.encode()
		}

		fn random_hash() -> T::Hash {
			let nonce = Self::get_and_increment_nonce();
			let (random_value, _) = T::KittyRandomness::random(&nonce);

			random_value
		}
	}
}
