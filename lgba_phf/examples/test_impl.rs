use boomphf::*;

fn main() {
    // Generate MPHF
    let mut possible_objects = Vec::new();
    for i in 0u32..16000 {
        possible_objects.push(i.wrapping_mul(715827883));
    }


    let n = possible_objects.len();
    let phf = Mphf::new(1.05, &possible_objects);

    println!("{:x?}", phf);

    // Get hash value of all objects
    let mut hashes = Vec::new();
    for v in possible_objects {
        hashes.push(phf.hash(&v));
    }
    hashes.sort();

    // Expected hash output is set of all integers from 0..n
    let expected_hashes: Vec<u64> = (0..n as u64).collect();
    assert!(hashes == expected_hashes);
}