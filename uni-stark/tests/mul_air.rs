use itertools::Itertools;
use p3_air::{Air, AirBuilder, BaseAir};
use p3_baby_bear::BabyBear;
use p3_challenger::DuplexChallenger;
use p3_commit::ExtensionMmcs;
use p3_dft::Radix2DitParallel;
use p3_field::extension::BinomialExtensionField;
use p3_field::Field;
use p3_fri::{FriBasedPcs, FriConfigImpl, FriLdt};
use p3_goldilocks::Goldilocks;
use p3_ldt::QuotientMmcs;
use p3_matrix::dense::RowMajorMatrix;
use p3_matrix::{Matrix, MatrixRowSlices};
use p3_mds::coset_mds::CosetMds;
use p3_merkle_tree::FieldMerkleTreeMmcs;
use p3_poseidon2::{DiffusionMatrixBabybear, DiffusionMatrixGoldilocks, Poseidon2};
use p3_symmetric::{CompressionFunctionFromHasher, PaddingFreeSponge, SerializingHasher32, TruncatedPermutation};
use p3_uni_stark::{prove, verify, StarkConfigImpl, VerificationError};
use rand::distributions::{Distribution, Standard};
use rand::{thread_rng, Rng};
use tracing_forest::ForestLayer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, Registry};
use p3_keccak::Keccak256Hash;

/// How many `a * b = c` operations to do per row in the AIR.
const REPETITIONS: usize = 911;
const TRACE_WIDTH: usize = REPETITIONS * 3;

struct MulAir;

impl<F> BaseAir<F> for MulAir {
    fn width(&self) -> usize {
        TRACE_WIDTH
    }
}

impl<AB: AirBuilder> Air<AB> for MulAir {
    fn eval(&self, builder: &mut AB) {
        let main = builder.main();
        let main_local = main.row_slice(0);

        for i in 0..REPETITIONS {
            let start = i * 3;
            let a = main_local[start];
            let b = main_local[start + 1];
            let c = main_local[start + 2];
            builder.assert_zero(a * b - c);
        }
    }
}

fn random_valid_trace<F: Field>(rows: usize) -> RowMajorMatrix<F>
where
    Standard: Distribution<F>,
{
    let mut rng = thread_rng();
    let mut trace_values = vec![F::default(); rows * TRACE_WIDTH];
    for (a, b, c) in trace_values.iter_mut().tuples() {
        *a = rng.gen();
        *b = rng.gen();
        *c = *a * *b;
    }
    RowMajorMatrix::new(trace_values, TRACE_WIDTH)
}

// #[test]
// fn test_prove_goldilocks() -> Result<(), VerificationError> {
//     Registry::default()
//         .with(EnvFilter::from_default_env())
//         .with(ForestLayer::default())
//         .init();
//
//     const HEIGHT: usize = 1 << 6;
//
//     type Val = Goldilocks;
//     type Domain = Val;
//     type Challenge = BinomialExtensionField<Val, 2>;
//     type PackedChallenge = BinomialExtensionField<<Domain as Field>::Packing, 2>;
//
//     type MyMds = CosetMds<Val, 8>;
//     let mds = MyMds::default();
//
//     type Perm = Poseidon2<Val, MyMds, DiffusionMatrixGoldilocks, 8, 7>;
//     let perm = Perm::new_from_rng(8, 22, mds, DiffusionMatrixGoldilocks, &mut thread_rng());
//
//     type MyHash = PaddingFreeSponge<Perm, 12, 8, 4>;
//     let hash = MyHash::new(perm.clone());
//
//     type MyCompress = TruncatedPermutation<Perm, 2, 8, 12>;
//     let compress = MyCompress::new(perm.clone());
//
//     type ValMmcs = FieldMerkleTreeMmcs<<Val as Field>::Packing, MyHash, MyCompress, 4>;
//     let val_mmcs = ValMmcs::new(hash, compress);
//
//     type ChallengeMmcs = ExtensionMmcs<Val, Challenge, ValMmcs>;
//     let challenge_mmcs = ChallengeMmcs::new(val_mmcs.clone());
//
//     type Dft = Radix2DitParallel;
//     let dft = Dft {};
//
//     type Challenger = DuplexChallenger<Val, Perm, 8>;
//
//     type Quotient = QuotientMmcs<Domain, Challenge, ValMmcs>;
//     type MyFriConfig = FriConfigImpl<Val, Challenge, Quotient, ChallengeMmcs, Challenger>;
//     let fri_config = MyFriConfig::new(1, 40, 8, challenge_mmcs);
//     let ldt = FriLdt { config: fri_config };
//
//     type Pcs = FriBasedPcs<MyFriConfig, ValMmcs, Dft, Challenger>;
//     type MyConfig = StarkConfigImpl<Val, Challenge, PackedChallenge, Pcs, Challenger>;
//
//     let pcs = Pcs::new(dft, val_mmcs, ldt);
//     let config = StarkConfigImpl::new(pcs);
//     let mut challenger = Challenger::new(perm.clone());
//     let trace = random_valid_trace::<Val>(HEIGHT);
//     let proof = prove::<MyConfig, _>(&config, &MulAir, &mut challenger, trace);
//
//     let serialized_proof = postcard::to_allocvec(&proof).expect("unable to serialize proof");
//     tracing::debug!("serialized_proof len: {} bytes", serialized_proof.len());
//
//     let deserialized_proof =
//         postcard::from_bytes(&serialized_proof).expect("unable to deserialize proof");
//
//     let mut challenger = Challenger::new(perm);
//     verify(&config, &MulAir, &mut challenger, &deserialized_proof)
// }

#[test]
fn test_prove_baby_bear() -> Result<(), VerificationError> {
    Registry::default()
        .with(EnvFilter::from_default_env())
        .with(ForestLayer::default())
        .init();

    const HEIGHT: usize = 1 << 14;

    type Val = BabyBear;
    type Domain = Val;
    type Challenge = BinomialExtensionField<Val, 4>;
    type PackedChallenge = BinomialExtensionField<<Domain as Field>::Packing, 4>;

    type MyMds = CosetMds<Val, 16>;
    let mds = MyMds::default();

    type Perm = Poseidon2<Val, MyMds, DiffusionMatrixBabybear, 16, 7>;
    let perm = Perm::new_from_rng(8, 22, mds, DiffusionMatrixBabybear, &mut thread_rng());

    type MyHash = SerializingHasher32<Keccak256Hash>;
    let hash = MyHash::new(Keccak256Hash {});

    type MyCompress = CompressionFunctionFromHasher<Val, MyHash, 2, 8>;
    let compress = MyCompress::new(hash);

    type ValMmcs = FieldMerkleTreeMmcs<Val, MyHash, MyCompress, 8>;
    let val_mmcs = ValMmcs::new(hash, compress);

    type ChallengeMmcs = ExtensionMmcs<Val, Challenge, ValMmcs>;
    let challenge_mmcs = ChallengeMmcs::new(val_mmcs.clone());

    type Dft = Radix2DitParallel;
    let dft = Dft {};

    type Challenger = DuplexChallenger<Val, Perm, 16>;

    type Quotient = QuotientMmcs<Domain, Challenge, ValMmcs>;
    type MyFriConfig = FriConfigImpl<Val, Challenge, Quotient, ChallengeMmcs, Challenger>;
    let fri_config = MyFriConfig::new(1, 100, 16, challenge_mmcs);
    let ldt = FriLdt { config: fri_config };

    type Pcs = FriBasedPcs<MyFriConfig, ValMmcs, Dft, Challenger>;
    type MyConfig = StarkConfigImpl<Val, Challenge, PackedChallenge, Pcs, Challenger>;

    let pcs = Pcs::new(dft, val_mmcs, ldt);
    let config = StarkConfigImpl::new(pcs);
    let mut challenger = Challenger::new(perm.clone());
    let trace = random_valid_trace::<Val>(HEIGHT);
    tracing::info!(
        "trace height: {}, trace width: {}",
        trace.height(),
        trace.width()
    );
    let proof = prove::<MyConfig, _>(&config, &MulAir, &mut challenger, trace);

    let serialized_proof = postcard::to_allocvec(&proof).expect("unable to serialize proof");
    tracing::info!("serialized_proof len: {} bytes", serialized_proof.len());

    let deserialized_proof =
        postcard::from_bytes(&serialized_proof).expect("unable to deserialize proof");

    let mut challenger = Challenger::new(perm);
    verify(&config, &MulAir, &mut challenger, &deserialized_proof)
}

#[test]
#[ignore] // TODO: Not ready yet.
fn test_prove_mersenne_31() {
    todo!()
}
