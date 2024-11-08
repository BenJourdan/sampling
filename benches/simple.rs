use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use num::Zero;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use sampling_tree::SimpleSamplingTree;

const TREE_SAMPLE_COUNT: usize = 1000;

// Define multiple values of n
static N_SIZES: &[usize] = &[32_768, 262_144, 1_048_576, 8_388_608];

fn construct<H>(data: impl Iterator<Item = H> + std::iter::ExactSizeIterator)
where
    H: rand::distributions::uniform::SampleUniform
        + From<u16>
        + std::fmt::Debug
        + Copy
        + std::cmp::PartialOrd
        + std::ops::Add<Output = H>
        + std::ops::Sub<Output = H>
        + num::Zero
        + std::ops::AddAssign
        + std::ops::SubAssign
        + From<u16>,
{
    let _sampling_tree = SimpleSamplingTree::from_iterable(data.into_iter()).unwrap();
}

#[macro_export]
macro_rules! create_construction_benchmarks {
    ($group:expr, $( $type:ty ),*) => {
        $(
            for &n in N_SIZES {
                $group.bench_function(
                    &format!("construct_{}_n={}", stringify!($type), n),
                    |b| {
                        b.iter_with_setup(
                            || {
                                let mut rng = rand::thread_rng();
                                let stop: $type = 10000u16.into();
                                let start: $type = <$type>::zero();
                                (0..n).map(move |_| rng.gen_range(start..stop))
                            },
                            |data| {
                                black_box(construct(data))
                            }
                        )
                    }
                );
            }
        )*
    };
}

#[macro_export]
macro_rules! create_sampling_benchmarks {
    ($group:expr, $( $type:ty ),*) => {
        $(
            for &n in N_SIZES {
                $group.bench_function(
                    &format!("sampling_{}_n={}", stringify!($type), n),
                    |b| {
                         b.iter_with_setup(
                            || {
                                let mut setup_rng = StdRng::from_entropy();
                                let stop: $type = 10000u16.into();
                                let start: $type = <$type>::zero();
                                let data = (0..n).map(move |_| setup_rng.gen_range(start..stop));
                                let sampling_tree = SimpleSamplingTree::from_iterable(data).unwrap();
                                (sampling_tree, StdRng::from_entropy())
                            },
                            |(sampling_tree, mut rng)| {
                                for _ in 0..TREE_SAMPLE_COUNT {
                                    let s = sampling_tree.sample(&mut rng);
                                    black_box(s.unwrap());
                                }
                                sampling_tree
                            }
                        )
                    }
                );
            }
        )*
    };
}

#[macro_export]
macro_rules! create_update_benchmarks {
    ($group:expr, $( $type:ty ),*) => {
        $(
            for &n in N_SIZES {
                $group.bench_function(
                    &format!("sampling_{}_n={}", stringify!($type), n),
                    |b| {
                         b.iter_with_setup(
                            || {
                                let mut setup_rng = StdRng::from_entropy();
                                let stop: $type = 10000u16.into();
                                let start: $type = <$type>::zero();
                                let data = (0..n).map(move |_| setup_rng.gen_range(start..stop));
                                let sampling_tree = SimpleSamplingTree::from_iterable(data).unwrap();
                                (sampling_tree, StdRng::from_entropy())
                            },
                            |(mut sampling_tree, mut rng)| {
                                for _ in 0..TREE_SAMPLE_COUNT {
                                    let index_to_update = rng.gen_range(0..n);
                                    let new_value = rng.gen_range(0 as $type..10000u16.into());
                                    sampling_tree.update(index_to_update.into(), new_value).unwrap();
                                }
                                sampling_tree
                            }
                        )
                    }
                );
            }
        )*
    };
}

fn bench_all(c: &mut Criterion) {
    // Group for construction benchmarks
    let mut group_construct = c.benchmark_group("construct");
    create_construction_benchmarks!(group_construct, f64, u64);
    group_construct.sampling_mode(criterion::SamplingMode::Flat);
    group_construct.sample_size(10);
    group_construct.nresamples(10);
    group_construct.measurement_time(std::time::Duration::from_secs(3));
    group_construct.finish(); // Finish the construction group

    // Group for sampling benchmarks
    let mut group_sampling = c.benchmark_group("sampling");
    group_sampling.throughput(Throughput::Elements(TREE_SAMPLE_COUNT as u64));
    group_sampling.sampling_mode(criterion::SamplingMode::Flat);
    group_sampling.sample_size(10);
    group_sampling.nresamples(10);
    group_sampling.measurement_time(std::time::Duration::from_secs(3));
    create_sampling_benchmarks!(group_sampling, f64, u64);
    group_sampling.finish(); // Finish the sampling group

    // Group for update benchmarks
    let mut group_update = c.benchmark_group("update");
    group_update.throughput(Throughput::Elements(TREE_SAMPLE_COUNT as u64));
    group_update.sampling_mode(criterion::SamplingMode::Flat);
    group_update.sample_size(10);
    group_update.nresamples(10);
    group_update.measurement_time(std::time::Duration::from_secs(3));
    create_update_benchmarks!(group_update, f64, u64);
    group_update.finish(); // Finish the update group
}

criterion_group!(benches, bench_all);
criterion_main!(benches);
