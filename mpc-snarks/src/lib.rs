#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

pub mod groth;
pub mod silly;

mod cp;
mod subspace_snark_tests;
