
use sampling_tree::SimpleSamplingTree;
use rand::Rng;
use std::time::Duration;
use divan::{Bencher, black_box};





fn main(){
    divan::main();
}


fn construct<H>(data: impl Iterator<Item=H> + std::iter::ExactSizeIterator)
where H: rand::distributions::uniform::SampleUniform
        + From<u16>
        + std::fmt::Debug + Copy
        + std::cmp::PartialOrd
        + std::ops::Add<Output=H>
        + std::ops::Sub<Output=H>
        + num::Zero
        + std::ops::AddAssign
        + std::ops::SubAssign
{
    let sampling_tree = SimpleSamplingTree::from_iterable(data.into_iter()).unwrap();
}

#[divan::bench_group(
    name = "simple_sampling_tree",
    sample_count = 10,
    max_time = Duration::from_secs(5),
    )]
mod simple_sampling_tree{
    use std::time::Duration;

    use divan::{Bencher, black_box, AllocProfiler, counter::ItemsCount};
    use super::*;

    #[divan::bench(
        args = [32,768,	262_144,1_048_576,8_388_608,16_777_216],
        types = [u16,u32,u64,f32,f64],
        max_time = 5
                    )]
    fn bench_construction<H>(bencher: Bencher,n: usize)
    where H: rand::distributions::uniform::SampleUniform
            + From<u16>
            + std::fmt::Debug + Copy + Clone
            + std::cmp::PartialOrd
            + std::ops::Add<Output=H>
            + std::ops::Sub<Output=H>
            + num::Zero
            + std::ops::AddAssign
            + std::ops::SubAssign
    {
    
        bencher.with_inputs(||{
            let mut rng = rand::thread_rng();
            let stop: H = 10000.into();
            let start: H = H::zero();
            (0..n).map(move |_|{
                rng.gen_range(start..stop)
            })
        }).bench_values(|data|{
            black_box(construct(data));
        })
    }


    #[divan::bench(
        args = [32_768,	262_144,1_048_576,8_388_608,16_777_216],
        types = [u16,u32,u64,f32,f64],
        max_time = Duration::from_secs(5),
        sample_count = 10,
        sample_size = 10,
        threads = 4,
                    )]
    fn bench_sampling<H>(bencher: Bencher,n: usize)
    where H: rand::distributions::uniform::SampleUniform
            + From<u16>
            + std::fmt::Debug + Copy + Clone
            + std::cmp::PartialOrd
            + std::ops::Add<Output=H>
            + std::ops::Sub<Output=H>
            + num::Zero
            + std::ops::AddAssign
            + std::ops::SubAssign
    {
        bencher
        .with_inputs(||{
            let mut rng = rand::thread_rng();
            let stop: H = 10000.into();
            let start: H = H::zero();

            let data = (0..n).map(|_|{
                rng.gen_range(start..stop)
            });

            let sampling_tree = SimpleSamplingTree::from_iterable(data).unwrap();
            (sampling_tree,rng)
        }).bench_local_values(|(sampling_tree,mut rng)|{
            black_box(move||{
                let _ = sampling_tree.sample(&mut rng);
                sampling_tree
            })
        })
    }
}

