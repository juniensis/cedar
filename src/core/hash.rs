use std::{fs, ops::BitXor, path::Path};

const K: usize = 0x517cc1b727220a95;

#[inline]
pub fn fx_hash(x: &[u8]) -> u64 {
    x.iter().fold(0u64, |acc, &byte| {
        acc.rotate_left(5)
            .bitxor(byte as u64)
            .wrapping_mul(K as u64)
    })
}

#[inline]
pub fn fx_word_hash(x: &[u8]) -> u64 {
    let mut hash = 0u64;
    let len = x.len();
    let over = len % WORD_SIZE;
    let mut ptr = x.as_ptr();
    for _ in 0..(len / WORD_SIZE) {
        let x = unsafe { (ptr as *const usize).read_unaligned() };
        hash = hash.rotate_left(5).bitxor(x as u64).wrapping_mul(K as u64);
        ptr = unsafe { ptr.add(WORD_SIZE) };
    }
    for _ in 0..over {
        hash = hash
            .rotate_left(5)
            .bitxor(unsafe { *ptr } as u64)
            .wrapping_mul(K as u64);
        ptr = unsafe { ptr.add(1) }
    }
    hash
}

#[inline]
pub fn fx_content_hash<P: AsRef<Path>>(path: P) -> u64 {
    let bytes = fs::read(path).unwrap();
    fx_hash(&bytes)
}

const WORD_SIZE: usize = (usize::BITS / 8) as usize;

/// Roughly 5x faster.
#[inline]
pub fn fx_content_hash_words<P: AsRef<Path>>(path: P) -> u64 {
    let bytes = fs::read(path).unwrap();
    fx_word_hash(&bytes)
}

#[cfg(test)]
mod core_hash_t {
    use std::fs;

    use crate::core::hash::{fx_content_hash, fx_content_hash_words, fx_hash, fx_word_hash};

    #[test]
    fn core_hash_deterministic_t() {
        let paths = [
            "./tests/loose/random_0016kb_t",
            "./tests/loose/random_0064kb_t",
            "./tests/loose/random_0256kb_t",
            "./tests/loose/random_1024kb_t",
            "./tests/loose/random_4096kb_t",
        ];

        for path in paths {
            let w = fx_content_hash(path);
            let x = fx_content_hash(path);
            let y = fx_content_hash_words(path);
            let z = fx_content_hash_words(path);
            assert_eq!(w, x);
            assert_eq!(y, z);
        }
    }
    #[ignore = "Results: Bytewise: 29.3 Wordwise: 28.8875"]
    #[test]
    fn core_hash_avalanche_t() {
        let flipped_bits = [1, 16, 256, 512];
        let spread_factor = [1, 16, 256, 512];
        let initial_files = [
            "./tests/loose/random_0016kb_t",
            "./tests/loose/random_0064kb_t",
            "./tests/loose/random_0256kb_t",
            "./tests/loose/random_1024kb_t",
            "./tests/loose/random_4096kb_t",
        ]
        .iter()
        .map(|x| fs::read(x).unwrap())
        .collect::<Vec<_>>();

        let mut bytewise_sum_entropy = 0.;
        let mut bytewise_count = 0;
        let mut wordwise_sum_entropy = 0.;
        let mut wordwise_count = 0;
        for (i, file) in initial_files.iter().enumerate() {
            let bytewise_hash = fx_hash(file);
            let wordwise_hash = fx_word_hash(file);
            for flipped in flipped_bits {
                for spread in spread_factor {
                    let mut altered = file.clone();
                    let len = altered.len();
                    let start_idx = (altered.len() + spread * 4) % altered.len();
                    for j in 0..flipped {
                        altered[(start_idx + (j * spread)) % len] ^=
                            0x01u8.rotate_left((flipped + spread) as u32);
                    }
                    let after_hash = fx_hash(&altered);
                    let word_hash = fx_word_hash(&altered);
                    let distance = (bytewise_hash ^ after_hash).count_ones();
                    let word_distance = (wordwise_hash ^ word_hash).count_ones();
                    wordwise_sum_entropy += word_distance as f64;
                    wordwise_count += 1;
                    bytewise_sum_entropy += distance as f64;
                    bytewise_count += 1;
                    println!("file: {i}, flipped: {flipped}, spread: {spread}");
                    println!(
                        "Bytewise: distance: {distance}\n  -> initial: {bytewise_hash:016x}\n  -> altered: {after_hash:016x}\n  -> running avg: {}",
                        bytewise_sum_entropy / bytewise_count as f64
                    );
                    println!(
                        "Wordwise: distance: {word_distance}\n  -> initial: {wordwise_hash:016x}\n  -> altered: {word_hash:016x}\n  -> running avg: {}",
                        wordwise_sum_entropy / wordwise_count as f64
                    );
                }
            }
        }

        println!(
            "Bytewise: {}, Wordwise: {}",
            bytewise_sum_entropy / bytewise_count as f64,
            wordwise_sum_entropy / wordwise_count as f64
        );
    }
}
