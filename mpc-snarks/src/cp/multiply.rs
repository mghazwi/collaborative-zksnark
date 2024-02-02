use crate::Opt;
use ark_bls12_377::{Fr, Parameters};
use ark_ec::{bls12::Bls12, PairingEngine};
use mpc_algebra::malicious_majority::MpcField;
use mpc_algebra::reveal::Reveal;
use mpc_net::{MpcMultiNet, MpcNet};
use std::ops::MulAssign;
use structopt::StructOpt;

fn multiply_shares<E: PairingEngine>(
    a: MpcField<E::Fr>,
    b: MpcField<E::Fr>,
) -> mpc_algebra::MpcField<
    <E as PairingEngine>::Fr,
    mpc_algebra::SpdzFieldShare<<E as PairingEngine>::Fr>,
> {
    let mut result = a.clone();
    result.mul_assign(b);
    result
}

pub fn test_collaborative_mul() {
    let opt = Opt::from_args();
    let party_id = opt.party;

    MpcMultiNet::init_from_file("./data/2", party_id as usize);

    type E = Bls12<Parameters>;

    let inputs = opt
        .args
        .iter()
        .map(|i| MpcField::<Fr>::from_add_shared(Fr::from(*i)))
        .collect::<Vec<_>>();

    let a = inputs[0];
    let b = inputs[1];
    let c = a.clone() * b.clone();

    let a_revealed = a.reveal();
    let b_revealed = b.reveal();
    let c_revealed = c.reveal();

    let result = multiply_shares::<E>(a, b);
    let revealed_result = result.reveal();

    // Assert that multiplying the shares equals multiplying the plain values
    assert_eq!(c_revealed, revealed_result);
    assert_eq!(c_revealed, a_revealed * b_revealed);

    MpcMultiNet::deinit();
}
