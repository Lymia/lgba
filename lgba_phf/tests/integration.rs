use lgba_phf::Mphf;
use quickcheck::quickcheck;
use std::{collections::HashSet, fmt::Debug, hash::Hash, iter::FromIterator};

/// Check that a Minimal perfect hash function (MPHF) is generated for the set xs
fn check_mphf<T>(xs: HashSet<T>) -> bool
where T: Sync + Hash + PartialEq + Eq + Debug + Send {
    let xsv: Vec<T> = xs.into_iter().collect();

    // test single-shot data input
    check_mphf_serial(1.001, &xsv) && check_mphf_serial(1.7, &xsv) && check_mphf_serial(2.5, &xsv)
}

/// Check that a Minimal perfect hash function (MPHF) is generated for the set xs
fn check_mphf_serial<T>(gamma: f32, xsv: &[T]) -> bool
where T: Hash + PartialEq + Eq + Debug {
    // Generate the MPHF
    let phf = Mphf::new(gamma, xsv);

    // Hash all the elements of xs
    let mut hashes: Vec<u64> = xsv.iter().map(|v| phf.hash(v)).collect();

    hashes.sort_unstable();

    // Hashes must equal 0 .. n
    let gt: Vec<u64> = (0..xsv.len() as u64).collect();
    hashes == gt
}

quickcheck! {
    fn check_string(v: HashSet<Vec<String>>) -> bool {
        check_mphf(v)
    }
}

quickcheck! {
    fn check_u32(v: HashSet<u32>) -> bool {
        check_mphf(v)
    }
}

quickcheck! {
    fn check_isize(v: HashSet<isize>) -> bool {
        check_mphf(v)
    }
}

quickcheck! {
    fn check_u64(v: HashSet<u64>) -> bool {
        check_mphf(v)
    }
}

quickcheck! {
    fn check_vec_u8(v: HashSet<Vec<u8>>) -> bool {
        check_mphf(v)
    }
}

#[test]
fn from_ints_serial() {
    let items = (0..1000000).map(|x| x * 2);
    assert!(check_mphf(HashSet::from_iter(items)));
}
