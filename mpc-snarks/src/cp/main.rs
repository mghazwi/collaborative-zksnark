pub mod circuit;
pub mod commitment;
pub mod multiply;
pub mod test_groth;

use crate::test_groth::test_groth;
use ark_bls12_377::{Bls12_377, Parameters};
use ark_ec::bls12::Bls12;
use mpc_algebra::{MpcPairingEngine, SpdzPairingShare};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "proof", about = "Standard and MPC proofs")]
struct Opt {
    // Party id
    #[structopt(long)]
    party: u8,

    /// Input arguments
    #[structopt()]
    args: Vec<u64>,
}

fn main() {
    type E = Bls12_377;
    type S = SpdzPairingShare<E>;

    test_groth();
}