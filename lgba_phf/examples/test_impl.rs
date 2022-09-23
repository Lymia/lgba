use lgba_phf::generator;

fn main() {
    let num_iter = 0u32..1020;

    // Generate PHF
    let mut possible_objects = Vec::new();
    for i in num_iter.clone() {
        possible_objects.push(i.wrapping_mul(715827883));
    }

    let phf = generator::generate_hash(1.0, &possible_objects);

    println!("{}", phf.generate_rust_code("func", "u32"));
    println!("Disp count: {}", phf.disps.len());

    // Verify PHF
    for (i, v) in possible_objects.iter().enumerate() {
        let idx = lgba_phf::hash_dynamic(phf.key, &phf.disps, &v, phf.map.len());
        assert_eq!(phf.map[idx], i as usize);
    }
    println!("Round trip validated!");
}
