use borsh::BorshDeserialize;
use l1x_sdk::contract;
use l1x_sdk::store::Vector;

pub struct Contract {}

#[contract]
impl Contract {
    pub fn new() {}

    pub fn add_name(name: String) {
        let mut names = match l1x_sdk::storage_read(b"b") {
            Some(vec) => Vector::try_from_slice(&vec).unwrap(),
            None => Vector::new(b"a".to_vec()),
        };
        names.push(name);
        names.flush();
        l1x_sdk::storage_write(b"b", &borsh::to_vec(&names).unwrap());
    }

    pub fn get_names() -> Vec<String> {
        let names: Vector<String> =
            Vector::try_from_slice(&l1x_sdk::storage_read(b"b").unwrap()).unwrap();
        let mut result = Vec::new();
        for i in 0..names.len() {
            result.push(names[i].clone());
        }
        result
    }

    pub fn hello() {
        let names: Vector<String> =
            Vector::try_from_slice(&l1x_sdk::storage_read(b"b").unwrap()).unwrap();
        for i in 0..names.len() {
            l1x_sdk::msg(&format!("Hello, {}!", names[i]));
        }
    }

    pub fn info() {
        l1x_sdk::msg(&format!(
            "block_hash={:?}, block_number={}, block_timestamp={}",
            l1x_sdk::block_hash(),
            l1x_sdk::block_number(),
            l1x_sdk::block_timestamp()
        ));
    }
}
