mod sampling;

pub type SimpleSamplingTree<C> = sampling::Tree<sampling::UnstableNode<C>>;
pub use sampling::UnstableNode;

#[cfg(test)]
mod tests {
    use std::{fmt::Write, sync::Arc};

    use super::*;
    use human_units::{FormatSize, Size};
    use indicatif::{self, ProgressState};
    use rand::{rngs::StdRng, Rng, SeedableRng};

    #[test]
    fn it_works() {
        let mut rng = rand::thread_rng();
        let n = 100;
        let range = 10000u32;
        let data = (0..n).map(|_| rng.gen_range(0..range));
        let mut sampling_tree: SimpleSamplingTree<_> =
            SimpleSamplingTree::from_iterable(data).unwrap();
        println!("{:?}", sampling_tree);
        let sample_idx = sampling_tree.sample(&mut rng).unwrap();
        println!(
            "{:?}, {:?}",
            sample_idx,
            sampling_tree.contribution(sample_idx).unwrap()
        );
        sampling_tree.update(sample_idx, 0).unwrap();
        println!("{:?}", sampling_tree);

        println!(
            "Size of node: {}",
            std::mem::size_of::<sampling::UnstableNode<u64>>()
        );
        // panic!();
    }

    #[test]
    fn test_throughput() {
        let mut rng = rand::thread_rng();
        let n = 1_000_000;
        let range = 10000u64;
        let data = (0..n).map(|_| rng.gen_range(0..range));
        let sampling_tree: SimpleSamplingTree<_> = SimpleSamplingTree::from_iterable(data).unwrap();

        // measure throughput of sampling
        let num_samples = 1_000_000;
        let num_threads = 4;

        let mp = Arc::new(indicatif::MultiProgress::new());
        let sty_main = indicatif::ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:40.cyan/blue} {percent}% {iter_per_sec}")
            .unwrap()
            .progress_chars("##-")
            .with_key(
                "iter_per_sec",
                |state: &ProgressState, w: &mut dyn Write| {
                    let speed = state.per_sec() as u64;
                    write!(w, "{}", Size(speed).format_size()).unwrap()
                },
            );

        let pb = Arc::new(mp.add(indicatif::ProgressBar::new(num_samples)));
        pb.set_style(sty_main.clone());
        let sampling: Arc<sampling::Tree<UnstableNode<u64>>> = Arc::new(sampling_tree);

        let handles: Vec<_> = (0..num_threads)
            .map(|_| {
                let pb = pb.clone();
                let mut rng = StdRng::from_entropy();
                let sampling = sampling.clone();

                std::thread::spawn(move || {
                    for _ in 0..num_samples / num_threads {
                        let sample_idx = sampling.sample(&mut rng).unwrap();
                        let _ = sampling.contribution(sample_idx).unwrap();
                        pb.inc(1);
                    }
                })
            })
            .collect();

        for handle in handles {
            handle.join().unwrap();
        }
    }
}
