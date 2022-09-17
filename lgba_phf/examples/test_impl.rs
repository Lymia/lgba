use lgba_phf::generator;

fn main() {
    // Generate MPHF
    let mut possible_objects = Vec::new();
    for i in 0u32..1020 {
        possible_objects.push(i.wrapping_mul(715827883));
    }

    let phf = generator::generate_hash(&possible_objects);

    println!("{:x?}", phf.key);
    println!("{:x?}", phf.disps);
    println!("Disp count: {}", phf.disps.len());
}
