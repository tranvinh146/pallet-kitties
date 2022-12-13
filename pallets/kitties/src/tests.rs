use crate::mock::*;
use frame_support::assert_ok;
use sp_core::H256;

#[test]
fn correct_create_kitty() {
	new_test_ext().execute_with(|| {
		assert_ok!(KittyModule::create_kitty(RuntimeOrigin::signed(1)));
		assert_eq!(KittyModule::kitty_owner(1).unwrap().len(), 1);
	})
}

#[test]
fn correct_transfer_kitty() {
	new_test_ext().execute_with(|| {
		assert_ok!(KittyModule::create_kitty(RuntimeOrigin::signed(1)));
		let dna_kitties = KittyModule::kitty_owner(1u64).unwrap();
		let dna = dna_kitties.first().unwrap();
		assert_ok!(KittyModule::transfer(
			RuntimeOrigin::signed(1),
			2u64,
			H256::from_slice(dna.as_slice())
		));
	})
}
