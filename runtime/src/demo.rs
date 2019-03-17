/// A runtime module template with necessary imports
use parity_codec::Encode;
use runtime_primitives::traits::{Zero, Hash, CheckedAdd, CheckedSub};
use support::{decl_module, decl_storage, dispatch::Result, StorageValue};

// Enables access to account balances and interacting with signed messages
use {
    balances,
    system::{self, ensure_signed},
};

/// The module's configuration trait.
pub trait Trait: balances::Trait {}

/// This module's storage items.
decl_storage! {
    trait Store for Module<T: Trait> as Demo {
        Payment get(payment): Option<T::Balance>;
        Pot get(pot): T::Balance;
        Nonce get(nonce): u64;
    }
}

decl_module! {
    /// The module declaration.
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn play(origin) -> Result {
        	// sender を取得。
            let sender = ensure_signed(origin)?;
            // payment=(賭け金) はいくらかを取得。
            let payment = Self::payment().ok_or("Must have payment amount set")?;

			// 乱数生成に使う nonce を取得。
            let mut nonce = Self::nonce();
            // pot に溜まっている balance を取得。
            let mut pot = Self::pot();
            // sender の持っている balance を取得。
            let mut sender_free_balance = <balances::Module<T>>::free_balance(&sender);

			// sender が賭け金を持っているかをチェック。OKなら実際に引いた値を返す。
	        sender_free_balance = sender_free_balance.checked_sub(&payment)
	        	.ok_or("User does not have enough funds to play the game")?;

            if (<system::Module<T>>::random_seed(), &sender, nonce)
                .using_encoded(<T as system::Trait>::Hashing::hash)
                .using_encoded(|e| e[0] < 128)
            {
                // sender が pot にある balance を総取り。
                sender_free_balance = sender_free_balance.checked_add(&pot)
                	.ok_or("Overflow when adding funds to user account")?;
				pot = Zero::zero();
            }

			// pot に掛け金を加える。
			pot = pot.checked_add(&payment)
				.ok_or("Overflow when adding funds to pot")?;

        	// nonce をインクリメント(オーバーフロー時に 0 に戻る)
        	nonce = nonce.wrapping_add(1);

            // sender に対して最終結果の balance をセットする。
            <balances::Module<T>>::set_free_balance(&sender, sender_free_balance);
            // pot に対して最終結果の balance をセットする。
            <Pot<T>>::put(pot);
            // nonce に対して最終結果の nonce をセットする。
            <Nonce<T>>::put(nonce);

            // Return Ok(())
            Ok(())
        }

        fn set_payment(origin, value: T::Balance) -> Result {
            // 署名チェック。
            let _ = ensure_signed(origin)?;
            // 掛け金がセットされているか否かをチェック。
            if Self::payment().is_none() {
                // 掛け金を value でセット
                <Payment<T>>::put(value);
                // pot にも初期値をセット
                <Pot<T>>::put(value);
            }

            // Return OK
            Ok(())
        }
    }
}

/// tests for this module
#[cfg(test)]
mod tests {
    use super::*;

    use primitives::{Blake2Hasher, H256};
    use runtime_io::with_externalities;
    use runtime_primitives::{
        testing::{Digest, DigestItem, Header},
        traits::{BlakeTwo256, IdentityLookup},
        BuildStorage,
    };
    use support::{assert_ok, impl_outer_origin};

    impl_outer_origin! {
        pub enum Origin for Test {}
    }

    // For testing the module, we construct most of a mock runtime. This means
    // first constructing a configuration type (`Test`) which `impl`s each of the
    // configuration traits of modules we want to use.
    #[derive(Clone, Eq, PartialEq, Debug)]
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
    impl balances::Trait for Test {
        type Balance = u128;
        type OnFreeBalanceZero = ();
        type OnNewAccount = ();
        type Event = ();
    }
    impl Trait for Test {}
    type Demo = Module<Test>;

    // This function basically just builds a genesis storage key/value store according to
    // our desired mockup.
    fn new_test_ext() -> runtime_io::TestExternalities<Blake2Hasher> {
        system::GenesisConfig::<Test>::default()
            .build_storage()
            .unwrap()
            .0
            .into()
    }

    #[test]
    fn it_works_for_default_value() {
        with_externalities(&mut new_test_ext(), || {
            // initialize demo::payment is None
            assert_eq!(Demo::payment(), None);
            // pot is 0
            assert_eq!(Demo::pot(), 0);
            // nonce is 0
            assert_eq!(Demo::nonce(), 0);

            assert_ok!(Demo::set_payment(Origin::signed(1), 100));
            assert_eq!(Demo::payment(), Some(100));
            assert_eq!(Demo::pot(), 100);

            // previous calc, sender add balance.
            <balances::Module<Test>>::set_free_balance(&1, 1000000);

            assert_ok!(Demo::play(Origin::signed(1)));
            let payment = Demo::payment();
            match payment {
                Some(v) => println!("payment: {}", v),
                None => println!("payment: None"),
            }
            println!("pot: {}", Demo::pot());

            assert_eq!(Demo::payment(),Some(100));
            assert_eq!(Demo::nonce(),1);
            // 1 play 後、pot には 100 溜まっている
            match Demo::pot() {
                100 => assert_eq!(<balances::Module<Test>>::free_balance(&1), 1000000+100),
                200 => assert_eq!(<balances::Module<Test>>::free_balance(&1), 1000000-100),
                v => assert!(false, "Unexpected pot {}", v),
            }
        });
    }
}
