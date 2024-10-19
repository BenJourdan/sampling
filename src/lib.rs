mod sampling;

pub type SimpleSamplingTree<C> = sampling::Tree<sampling::UnstableNode<C>>;
pub use sampling::UnstableNode;

#[cfg(test)]
mod tests {
    use super::*;
    use rand::Rng;

    #[test]
    fn it_works() {
        let mut rng = rand::thread_rng();
        let n = 100;
        let range = 10000u32;
        let data = (0..n).map(|_|{
            rng.gen_range(0..range)
        });
        let mut sampling_tree: SimpleSamplingTree<_> = SimpleSamplingTree::from_iterable(data).unwrap();
        println!("{:?}",sampling_tree);
        let sample_idx = sampling_tree.sample(&mut rng).unwrap();
        println!("{:?}, {:?}",sample_idx, sampling_tree.contribution(sample_idx).unwrap());
        sampling_tree.update(sample_idx,0).unwrap();
        println!("{:?}",sampling_tree);

        println!("Size of node: {}",std::mem::size_of::<sampling::UnstableNode<u64>>());
        panic!();
    }
}
