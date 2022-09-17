#[cfg(test)]
#[macro_use]
extern crate bencher;

use bencher::Bencher;
use lgba_phf::Mphf;

fn build1_ser(bench: &mut Bencher) {
    bench.iter(|| {
        let items: Vec<u64> = (0..10000u64).map(|x| x * 2).collect();
        let _ = Mphf::new(2.0, &items);
    });
}

fn scan1_ser(bench: &mut Bencher) {
    let items: Vec<u64> = (0..10000u64).map(|x| x * 2).collect();
    let phf = Mphf::new(2.0, &items);

    bench.iter(|| {
        for i in (0..10000u64).map(|x| x * 2) {
            phf.hash(&i);
        }
    });
}

benchmark_group!(benches, build1_ser, scan1_ser);
benchmark_main!(benches);
