use std::time::Instant;

use criterion::black_box;
use criterion::criterion_group;
use criterion::criterion_main;
use criterion::Criterion;
use plonky2::field::goldilocks_field::GoldilocksField;
use plonky2::field::types::Sample;
use plonky2::hash::hash_types::HashOut;
use plonky2::hash::hash_types::HashOutTarget;
use plonky2::iop::witness::PartialWitness;
use plonky2::iop::witness::WitnessWrite;
use plonky2::plonk::circuit_builder::CircuitBuilder;
use plonky2::plonk::circuit_data::CircuitConfig;
use plonky2::plonk::config::Hasher;
use plonky2::plonk::config::PoseidonGoldilocksConfig;
use plonky2_5::common::poseidon2::poseidon2::Poseidon2Hash;
use plonky2_5::p3::commit::MerkleTreeMmcs;

const D: usize = 2;
type C = PoseidonGoldilocksConfig;
type F = GoldilocksField;
type H = Poseidon2Hash;

fn benchmark_my_function(c: &mut Criterion) {
    for size in (12..=15).map(|exp| 2usize.pow(exp)) {
        c.bench_function(format!("poseidon2_circuit_{}", size).as_str(), |b| {
            b.iter_batched(
                // Setup code
                || {
                    let config = CircuitConfig::standard_recursion_config();
                    let mut builder = CircuitBuilder::<F, D>::new(config);

                    let mut left = Vec::new();
                    let mut right = Vec::new();
                    let mut output = Vec::new();
                    for _ in 0..size {
                        let l = builder.add_virtual_hash();
                        let r = builder.add_virtual_hash();
                        let o = HashOutTarget::from(MerkleTreeMmcs::compress::<H, F, D>(
                            [l.elements, r.elements],
                            &mut builder,
                        ));
                        left.push(l);
                        right.push(r);
                        output.push(o);
                    }

                    let data = builder.build::<C>();

                    let mut pw = PartialWitness::new();
                    for i in 0..size {
                        let l_val = HashOut::rand();
                        let r_val = HashOut::rand();
                        let o_val = H::two_to_one(l_val, r_val);

                        pw.set_hash_target(left[i], l_val);
                        pw.set_hash_target(right[i], r_val);
                        pw.set_hash_target(output[i], o_val);
                    }

                    (data, pw)
                },
                // Code to benchmark
                |(data, pw)| {
                    data.prove(black_box(pw)).unwrap();
                },
                // Benchmarking policy
                criterion::BatchSize::LargeInput,
            );
        });
    }
}

criterion_group!(benches, benchmark_my_function);
criterion_main!(benches);
