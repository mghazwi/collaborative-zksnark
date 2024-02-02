use crate::Opt;
use ark_bls12_377::{Fr, FrParameters};
use ark_ec::PairingEngine;
use ark_ff::Fp256;
use ark_ff::UniformRand;
use ark_std::test_rng;
use mpc_algebra::malicious_majority::MpcField;
use mpc_algebra::reveal::Reveal;
use mpc_net::{MpcMultiNet, MpcNet};
use std::ops::Mul;
use structopt::StructOpt;

pub fn test_collaborative_commitment<E: PairingEngine>() {
    println!("Generating random matrix...");
    let opt = Opt::from_args();
    let party_id = opt.party;

    MpcMultiNet::init_from_file("./data/2", party_id as usize);

    let rng = &mut test_rng();

    // Create commitment randomness

    let link_v = MpcField::<Fr>::from_add_shared(Fr::rand(rng));

    let inputs = opt
        .args
        .iter()
        .map(|i| MpcField::<Fr>::from_add_shared(Fr::from(*i)))
        .collect::<Vec<_>>();

    let pedersen_bases = vec![
        MpcField::<Fr>::from_public(Fr::from(1u64)),
        MpcField::<Fr>::from_public(Fr::from(2u64)),
        MpcField::<Fr>::from_public(Fr::from(3u64)),
    ];

    let input_assignment_with_one_with_link_hider = [&inputs, &[link_v][..]].concat();

    let pedersen_values = input_assignment_with_one_with_link_hider
        .into_iter()
        .map(|v| v)
        .collect::<Vec<_>>();

    let commitment = pedersen_commitment::<E>(&pedersen_bases, &pedersen_values);

    let commitment = commitment.reveal();

    println!("Commitment: {:?}", commitment);
}

fn pedersen_commitment<E: PairingEngine>(
    pedersen_bases: &Vec<MpcField<Fp256<FrParameters>>>,
    pedersen_values: &Vec<MpcField<Fp256<FrParameters>>>,
) -> mpc_algebra::MpcField<Fp256<FrParameters>, mpc_algebra::SpdzFieldShare<Fp256<FrParameters>>> {
    let mut res = MpcField::<Fr>::from_public(Fr::from(0u64));

    for (base, value) in pedersen_bases.iter().zip(pedersen_values.iter()) {
        res += base.mul(value);
    }

    res
}
