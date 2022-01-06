mod definitions;
pub mod error;
pub mod types;
pub mod utils;
pub mod ser;
mod hash_prefixes;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
