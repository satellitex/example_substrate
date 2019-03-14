/// A runtime module template with necessary imports

use parity_codec::Encode;
use runtime_primitives::traits::Hash;
use support::{decl_module, decl_storage, decl_event, StorageValue, dispatch::Result};

// Enables access to account balances and interacting with signed messages
use {balances, system::{self, ensure_signed}};

/// The module's configuration trait.
pub trait Trait: balances::Trait {}

/// This module's storage items.
decl_storage! {
    trait Store for Module<T: Trait> as Demo {
        Payment get(payment): Option<T::Balance>;
        Pot get(pot): T::Balance;
    }
}

decl_module! {
	/// The module declaration.
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
	    fn play(origin) -> Result {
	        let sender = ensure_signed(origin)?;
			let payment = Self::payment().ok_or("Must have payment amount set")?;

            // == decrease_free_balance.
            let balance = <balances::Module<T>>::free_balance(&sender);
            <balances::Module<T>>::set_free_balance(&sender, balance - payment);

			if (<system::Module<T>>::random_seed(), &sender)
				.using_encoded(<T as system::Trait>::Hashing::hash)
				.using_encoded(|e| e[0] < 128)
			{
			    // == increase_free_balance
			    let balance = <balances::Module<T>>::free_balance(&sender);
				<balances::Module<T>>::set_free_balance(&sender, balance + <Pot<T>>::take());
			}

			<Pot<T>>::mutate(|pot| *pot += payment);

			Ok(())
	    }

        fn set_payment(_origin, value: T::Balance) -> Result {
            if Self::payment().is_none() {
                <Payment<T>>::put(value);
                <Pot<T>>::put(value);
            }
			Ok(())
		}
	}
}

decl_event!(
	/// An event in this module.
	pub enum Event<T> where AccountId = <T as system::Trait>::AccountId {
		// Just a dummy event.
		// Event `Something` is declared with a parameter of the type `u32` and `AccountId`
		// To emit this event, we call the deposit funtion, from our runtime funtions
		SomethingStored(u32, AccountId),
	}
);

/// tests for this module
#[cfg(test)]
mod tests {
	use super::*;

	use runtime_io::with_externalities;
	use primitives::{H256, Blake2Hasher};
	use support::{impl_outer_origin, assert_ok};
	use runtime_primitives::{
		BuildStorage,
		traits::{BlakeTwo256, IdentityLookup},
		testing::{Digest, DigestItem, Header}
	};

	impl_outer_origin! {
		pub enum Origin for Test {}
	}

	// For testing the module, we construct most of a mock runtime. This means
	// first constructing a configuration type (`Test`) which `impl`s each of the
	// configuration traits of modules we want to use.
	#[derive(Clone, Eq, PartialEq)]
	pub struct Test;
	impl system::Trait for Test {
		type Origin = Origin;
		type Index = u64;
		type BlockNumber = u64;
		type Hash = H256;
		type Hashing = BlakeTwo256;
		type Digest = Digest;
		type AccountId = u64;
		type Lookup = IdentityLookup<u64>;
		type Header = Header;
		type Event = ();
		type Log = DigestItem;
	}
	impl Trait for Test {
		type Event = ();
	}
	type demo = Module<Test>;

	// This function basically just builds a genesis storage key/value store according to
	// our desired mockup.
	fn new_test_ext() -> runtime_io::TestExternalities<Blake2Hasher> {
		system::GenesisConfig::<Test>::default().build_storage().unwrap().0.into()
	}

	#[test]
	fn it_works_for_default_value() {
		with_externalities(&mut new_test_ext(), || {
			// Just a dummy test for the dummy funtion `do_something`
			// calling the `do_something` function with a value 42
			assert_ok!(demo::do_something(Origin::signed(1), 42));
			// asserting that the stored value is equal to what we stored
			assert_eq!(demo::something(), Some(42));
		});
	}
}
