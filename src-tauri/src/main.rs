#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use secp256k1::Secp256k1;
use std::fmt::Write;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc;
use std::sync::mpsc::Sender;
use std::sync::Arc;
use std::thread;
use tiny_keccak::Hasher;
use tiny_keccak::Keccak;

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![return_prefix])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
fn return_prefix(input: &str, trs: u32) -> (String, String) {
    run(input, trs)
}

pub struct Wallet {
    pub private_key: String,
    pub public_key: String,
}

impl Wallet {
    pub fn new() -> Wallet {
        let (private_key, public_key) = generate_key_address();
        Wallet {
            private_key,
            public_key,
        }
    }
}

pub fn generate_key_address() -> (String, String) {
    let mut rng = rand::thread_rng();
    let context = Secp256k1::new();
    let (private_key, public_key) = context.generate_keypair(&mut rng);
    let mut private_key_string = String::new();

    for &byte in private_key[..].iter() {
        write!(&mut private_key_string, "{:02x}", byte).expect("Unable to write");
    }

    let mut sha3 = Keccak::v256();
    sha3.update(&public_key.serialize_uncompressed()[1..65]);

    let mut address: [u8; 32] = [0; 32];
    sha3.finalize(&mut address);

    let mut address_string = String::new();
    for &byte in address.iter().skip(12) {
        write!(&mut address_string, "{:02x}", byte).expect("Unable to write");
    }

    (private_key_string, address_string)
}

// pub fn find_address_starting_with(s: &str, tx: Sender<Wallet>, found: Arc<AtomicBool>) -> Wallet {
//     let mut wallet = Wallet::new();
//     loop {
//         if found.load(Ordering::Relaxed) {
//             return Wallet::new();
//         }

//         let mut address;

//         wallet = Wallet::new();
//         address = wallet.public_key.clone();
//         address = checksum(&address);
//         let score = score(&address, s);
//         if score == s.len() as i32 {
//             found.store(true, Ordering::Relaxed);
//             tx.send(wallet);
//         }
//     }
// }

pub fn score(a: &str, s: &String) -> i32 {
    let mut _s = 0;
    for (i, c) in s.chars().enumerate() {
        if a.chars().nth(i).unwrap() == c {
            _s += 1;
        }
    }
    _s
}

pub fn run(s: &str, trs: u32) -> (String, String) {
    let (tx, rx) = mpsc::channel();
    let found = Arc::new(AtomicBool::new(false));
    let mut threads = vec![];
    for _ in 0..trs {
        let _s = String::from(s);
        let thread_tx = tx.clone();
        let thread_found = found.clone();
        threads.push(thread::spawn(move || {
            let mut wallet = Wallet::new();
            loop {
                if thread_found.load(Ordering::Relaxed) {
                    return Wallet::new();
                }

                let mut address;

                wallet = Wallet::new();
                address = wallet.public_key.clone();
                address = checksum(&address);
                let score = score(&address, &_s);
                if score == _s.len() as i32 {
                    thread_found.store(true, Ordering::Relaxed);
                    thread_tx.send(wallet).expect("Err");
                }
            }
        }))
    }

    for t in threads {
        _ = t.join();
    }

    let wallet = rx.recv().unwrap();
    (wallet.public_key, wallet.private_key)
}

pub fn checksum(address: &str) -> String {
    let address = address.to_lowercase();

    let address_hash = {
        let mut hasher = Keccak::v256();
        hasher.update(address.as_bytes());
        let mut address: [u8; 32] = [0; 32];
        hasher.finalize(&mut address);

        let mut address_string = String::new();
        for &byte in address.iter() {
            write!(&mut address_string, "{:02x}", byte).expect("Unable to write");
        }

        address_string
    };

    address
        .char_indices()
        .fold("".to_string(), |mut acc, (index, address_char)| {
            // this cannot fail since it's Keccak256 hashed
            let n = u16::from_str_radix(&address_hash[index..index + 1], 16).unwrap();

            if n > 7 {
                // make char uppercase if ith character is 9..f
                acc.push_str(&address_char.to_uppercase().to_string())
            } else {
                // already lowercased
                acc.push(address_char)
            }

            acc
        })
}

pub fn is_possible_pattern(x: &str) -> bool {
    x.as_bytes()
        .iter()
        .all(|&c| (b'a'..=b'f').contains(&c) || (b'0'..=b'9').contains(&c))
}

pub fn calculate_difficulty(s: &str, case_sensitive: bool) -> u64 {
    if case_sensitive {
        22_u64.pow(s.len() as u32)
    } else {
        16_u64.pow(s.len() as u32)
    }
}

pub fn calculate_estimated_time(speed: u64, difficulty: u64) -> u64 {
    if speed == 0 {
        0
    } else {
        difficulty / speed
    }
}

pub fn time_left(estimated_time: u64, elapsed_time: u64) -> u64 {
    if elapsed_time > estimated_time {
        return 0;
    }
    estimated_time - elapsed_time
}
