use std::sync::Mutex;

//XOR Shift Pseudo-Random Number Generator
static SEED: Mutex<u32> = Mutex::new(1804289383);

pub fn get_random_num() -> u32 {
    let mut number = SEED.lock().unwrap();

    *number ^= *number << 13;
    *number ^= *number >> 17;
    *number ^= *number << 5;

    *number
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_random_num() {
        assert_eq!(get_random_num(), 1741896308);
        assert_eq!(get_random_num(), 321584506);
        assert_eq!(get_random_num(), 3694591032);
        assert_eq!(get_random_num(), 1972257248);
        assert_eq!(get_random_num(), 200407065);
    }
}