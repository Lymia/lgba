use core::cmp;
use lgba::save::{Error, SaveAccess};

#[derive(Clone)]
struct Rng(u32);
impl Rng {
    fn iter(&mut self) {
        self.0 = self.0.wrapping_mul(2891336453).wrapping_add(100001);
    }
    fn next_u8(&mut self) -> u8 {
        self.iter();
        (self.0 >> 22) as u8 ^ self.0 as u8
    }
    fn next_under(&mut self, under: u32) -> u32 {
        self.iter();
        let scale = 31 - under.leading_zeros();
        ((self.0 >> scale) ^ self.0) % under
    }
}

const MAX_BLOCK_SIZE: usize = 4 * 1024;

fn check_status<T>(r: Result<T, Error>) -> T {
    match r {
        Ok(v) => v,
        Err(e) => panic!("Error encountered: {:?}", e),
    }
}

fn do_test(seed: Rng, offset: usize, len: usize, block_size: usize) -> Result<(), Error> {
    let mut access = SaveAccess::open()?;
    let mut buffer = [0; MAX_BLOCK_SIZE];

    println!(" - Clearing media...");
    let mut prepared = access.prepare_write(offset..offset + len)?;

    println!(" - Writing media...");
    let cycles = lgba::timer::time_cycles(|| {
        let mut rng = seed.clone();
        let mut current = offset;
        let end = offset + len;
        while current != end {
            let cur_len = cmp::min(end - current, block_size);
            for i in 0..cur_len {
                buffer[i] = rng.next_u8();
            }
            prepared.write(current, &buffer[..cur_len]).unwrap();
            current += cur_len;
        }
    });
    println!("   - Done in {} cycles.", cycles);

    println!(" - Validating media...");
    let cycles = lgba::timer::time_cycles(|| {
        let mut rng = seed.clone();
        let mut current = offset;
        let end = offset + len;
        while current != end {
            let cur_len = cmp::min(end - current, block_size);
            access.read(current, &mut buffer[..cur_len]).unwrap();
            for i in 0..cur_len {
                let cur_byte = rng.next_u8();
                assert!(
                    buffer[i] == cur_byte,
                    "Read does not match earlier write: {} != {} @ 0x{:05x}",
                    buffer[i],
                    cur_byte,
                    current + i,
                );
            }
            current += cur_len;
        }
    });
    println!("   - Done in {} cycles.", cycles);

    Ok(())
}

pub fn run() -> ! {
    // set the save type
    lgba::save::init_flash_128k();

    // check some metainfo on the save type
    let access = check_status(SaveAccess::open());
    let media_len = access.len();
    println!("Media info: {:#?}", access.media_info());
    println!("Media size: {} bytes", access.len());
    println!();
    drop(access);

    // actually test the save implementation
    if media_len >= (1 << 12) {
        println!("[ Full write, 4KiB blocks ]");
        check_status(do_test(Rng(2000), 0, media_len, 4 * 1024));
    }

    println!("[ Full write, 0.5KiB blocks ]");
    check_status(do_test(Rng(1000), 0, media_len, 512));

    // test with random segments now.
    let mut rng = Rng(12345);
    for i in 0..8 {
        let rand_length = rng.next_under((media_len >> 1) as u32) as usize + 50;
        let rand_offset = rng.next_under(media_len as u32 - rand_length as u32) as usize;
        let block_size = cmp::min(rand_length >> 2, MAX_BLOCK_SIZE - 100);
        let block_size = rng.next_under(block_size as u32) as usize + 50;

        println!(
            "[ Partial, offset = 0x{:06x}, len = {}, bs = {} ]",
            rand_offset, rand_length, block_size,
        );
        check_status(do_test(Rng(i * 10000), rand_offset, rand_length, block_size));
    }

    // show a pattern so we know it worked
    println!("All tests complete!");

    loop {}
}
