# sampling-tree
A simple sampling tree implementation for sampling discrete distributions with sparse dynamic updates.
This allows us to sample efficiently from a distribution given the relative importance of each datapoint.
Construction time is O(n), updating is O(log(n)), and sampling is O(log(n)). The memory footprint is no more than twice the size of `n*std::mem::size_of::<T>()` where `T` is weight datatype.

Basic usage:
```rust
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
```

Supports most numeric types for the type of each datapoint's weight.
