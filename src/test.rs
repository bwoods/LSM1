use std::collections::BTreeMap;

use itertools::assert_equal;
use quickcheck_macros::quickcheck;

#[quickcheck]
fn property_testing(insertions: Vec<u32>, deletions: Vec<u32>) {
    let mut map = BTreeMap::<Vec<u8>, Vec<u8>>::new();

    let file = temp_file::TempFile::new().unwrap();
    let mut lsm = crate::map::Map::new(file.path().to_str().unwrap()).unwrap();

    for n in insertions.iter() {
        map.insert(n.to_be_bytes().into(), n.to_le_bytes().into());
        lsm.insert(n.to_be_bytes().as_ref(), n.to_le_bytes().as_ref());
    }

    for n in deletions.iter() {
        map.remove(n.to_be_bytes().as_ref());
        lsm.remove(n.to_be_bytes().as_ref());
    }

    assert_equal(
        map.iter().map(|(k, v)| (k.as_slice(), v.as_slice())),
        lsm.iter(),
    );
}
