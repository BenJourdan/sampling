use std::mem::MaybeUninit;
use num::Zero;
use rand::distributions::uniform::SampleUniform;
// MARK: Error def
#[allow(clippy::enum_variant_names)]
#[allow(dead_code)]
#[derive(Debug)]
pub enum Error{
    NodeNotFound(Index),
    NodeHasNoParent(ShiftedIndex),
    NodeAlreadyInserted(Index),
    CannotDirectlyUpdateInternalNode(ShiftedIndex),
    EmptyTree,
    NumericalError
}
impl std::fmt::Display for Error{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result{
        match self{
            Error::NodeNotFound(index) => write!(f, "Node with index {} not found", index.0),
            Error::NodeHasNoParent(index) => write!(f, "Node with shifted_index {} has no parent", index.0),
            Error::NodeAlreadyInserted(index) => write!(f, "Node with index {} is already inserted", index.0),
            Error::CannotDirectlyUpdateInternalNode(index) => write!(f, "Cannot directly update internal node with shifted_index {}", index.0),
            Error::EmptyTree => write!(f, "Tree is empty"),
            Error::NumericalError => write!(f, "Numerical error"),
        }
    }
}

impl std::error::Error for Error {}

// MARK: Newtypes
#[derive(Eq, PartialEq, Hash, Copy, Clone, Debug)]
pub struct Index(pub usize);

impl From<usize> for Index{
    fn from(index: usize) -> Self{
        Index(index)
    }
}

#[derive(Eq, PartialEq, Hash, Copy, Clone, Debug)]
pub struct ShiftedIndex(pub usize);

impl From<usize> for ShiftedIndex{
    fn from(index: usize) -> Self{
        ShiftedIndex(index)
    }
}

// MARK: NodeState enum
#[derive(PartialEq)]
pub enum NodeState{
    Internal,
    Leaf
}


// MARK: Node trait
pub trait Node
where Self: Sized
{
    type C;

    fn node_state(shifted_index: ShiftedIndex, storage_size: usize) -> NodeState{
        let num_leaves = (storage_size + 1)/2;
        match shifted_index.0 < num_leaves - 1{
            true => NodeState::Internal,
            false => NodeState::Leaf
        }
    }

    fn left_child(shifted_index: ShiftedIndex) -> ShiftedIndex{
        ShiftedIndex(2*shifted_index.0 + 1)
    }
    fn right_child(shifted_index: ShiftedIndex) -> ShiftedIndex{
        ShiftedIndex(2*shifted_index.0 + 2)
    }
    fn parent(shifted_index: ShiftedIndex) -> Result<ShiftedIndex,Error>{
        if shifted_index.0 == 0{
            return Err(Error::NodeHasNoParent(shifted_index));
        }
        Ok(ShiftedIndex((shifted_index.0 - 1)/2))
    }
    fn contribution(&self) -> Self::C;
    fn new(contribution: Self::C) -> Self;
    fn from_children(left: &Self, right: &Self) -> Self;
    fn update(storage: &mut Vec<Self>, shifted_index: ShiftedIndex,value: Self::C) -> Result<(),Error>;
    fn sample(storage: &[Self], rng: &mut impl rand::Rng) -> Result<ShiftedIndex,Error>;
}

// MARK: Tree struct
pub struct Tree<N>{
    storage: Vec<N>,
    num_leaves: usize,
    num_nodes: usize,
}

impl <N> std::fmt::Debug for Tree<N>
    where N: std::fmt::Debug{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result{
        let internal_nodes: &[N] = &self.storage[..self.num_leaves-1];
        let leaves: &[N] = &self.storage[self.num_leaves-1..];

        write!(f, "Internal nodes: {:?}\nLeaves: {:?}", internal_nodes, leaves)
    }
}

impl<N> Tree<N>
where N: Node
{
    pub fn get_shifted_node_index(&self, node_index: Index) -> Result<ShiftedIndex,Error>{
        let shifted_node_index = node_index.0 + self.num_leaves - 1;
        match shifted_node_index < self.num_nodes{
            false => Err(Error::NodeNotFound(node_index)),
            true => Ok(ShiftedIndex(shifted_node_index))
        }
    }
    pub fn get_node_index(&self, shifted_node_index: ShiftedIndex) -> Result<Index, Error>{
        let node_index = shifted_node_index.0 - (self.num_leaves - 1);
        match node_index < self.num_leaves{
            false => Err(Error::NodeNotFound(Index(node_index))),
            true => Ok(Index(node_index))
        }
    }

    pub fn get_contribution(&self, node_index: Index) -> Result<N::C,Error>{
        let shifted_index = self.get_shifted_node_index(node_index)?;
        Ok(self.storage[shifted_index.0].contribution())
    }

    pub fn from_iterable<I>(mut iterator: I) ->Result<Self,Error>
    where I: Iterator<Item=N::C> + ExactSizeIterator
    {
        let num_leaves = iterator.len();
        if num_leaves == 0{
            return Err(Error::EmptyTree);
        }
        let num_nodes = 2*num_leaves - 1;
        let mut storage: Vec<MaybeUninit<N>> = Vec::with_capacity(num_nodes);
        // SAFTEY: We have reserved enough space for the elements and 
        // we now initialize them
        unsafe{
            storage.set_len(num_nodes);
            // stick the leaves at the end of the storage:
            storage[num_leaves-1..].iter_mut().for_each(|uninit|{
                let uninit_ptr = uninit.as_mut_ptr();
                let leaf = iterator.next().unwrap();
                std::ptr::write(uninit_ptr,N::new(leaf));
            });
            // Now we fill up the rest of the tree backwards:
            (0..num_leaves-1).rev().for_each(|i|{
                let left = N::left_child(i.into());
                let right = N::right_child(i.into());
                let parent = N::from_children(
                    storage[left.0].as_ptr().as_ref().unwrap(),
                    storage[right.0].as_ptr().as_ref().unwrap());
                storage[i] = MaybeUninit::new(parent);
            });
        }
        unsafe{
            // Now transmute the storage to the final form and return:
            let storage: Vec<N> = std::mem::transmute(storage);
            Ok(Self{
                storage,
                num_leaves,
                num_nodes
            })
        }
    }
    pub fn sample(&self, rng: &mut impl rand::Rng) -> Result<Index,Error>{
        N::sample(&self.storage,rng).and_then(|shifted_index|{
            self.get_node_index(shifted_index)
        })
    }
    pub fn update(&mut self, node_index: Index, value: N::C) -> Result<(), Error>{
        let shifted_index = self.get_shifted_node_index(node_index)?;
        N::update(&mut self.storage,shifted_index,value)
    }

    pub fn contribution(&self, node_index: Index) -> Result<N::C,Error>{
        let shifted_index = self.get_shifted_node_index(node_index)?;
        Ok(self.storage[shifted_index.0].contribution())
    }
}



// MARK: UnstableNode
#[derive(Debug)]
pub struct UnstableNode<C>{
    contribution: C
}



impl<C> Node for UnstableNode<C>
where C: Copy + Clone + std::ops::Add<Output = C> 
        + std::ops::Sub<Output = C> + Zero 
        + std::ops::AddAssign + std::ops::SubAssign
        + SampleUniform + std::cmp::PartialOrd
{
    type C = C;
    fn contribution(&self) -> Self::C {
        self.contribution
    }

    fn new(contribution: Self::C) -> Self {
        UnstableNode{
            contribution
        }
    }

    fn from_children(left: &Self, right: &Self) -> Self {
        UnstableNode{
            contribution: left.contribution + right.contribution
        }
    }
    fn update(storage: &mut Vec<Self>, shifted_index: ShiftedIndex,value: Self::C) -> Result<(),Error> {
        let storage_size = storage.len();
        let leaf = storage.get_mut(shifted_index.0).unwrap();
        match Self::node_state(shifted_index, storage_size){
            NodeState::Internal => Err(Error::CannotDirectlyUpdateInternalNode(shifted_index)),
            NodeState::Leaf =>{
                let old_value = &mut leaf.contribution;
                let (abs_diff,sign): (C,bool) = match *old_value <=value{
                    true => (value - *old_value, true),
                    false => (*old_value - value, false)
                };
                if abs_diff.is_zero(){
                    Ok(())
                }else{
                    match sign{
                        true => *old_value += abs_diff,
                        false => *old_value -= abs_diff,
                    }
                    let mut node_shifted_index = shifted_index;
                    while let Ok(parent_shifted_index) = Self::parent(node_shifted_index){
                        let parent = storage.get_mut(parent_shifted_index.0).unwrap();

                        match Self::node_state(parent_shifted_index, storage_size){
                            NodeState::Internal => {
                                match sign{
                                    true => parent.contribution += abs_diff,
                                    false => parent.contribution -= abs_diff,
                                }
                            },
                            NodeState::Leaf => unreachable!("Internal node has leaf parent")
                        }
                        node_shifted_index = parent_shifted_index;
                    }
                    Ok(())
                }
            }
        }
    }

    fn sample(storage: &[Self], rng: &mut impl rand::Rng) -> Result<ShiftedIndex,Error> {
        if storage.is_empty(){
            return Err(Error::EmptyTree);
        }
        let storage_size = storage.len();
        let mut shifted_index = ShiftedIndex(0);
        while Self::node_state(shifted_index, storage_size) == NodeState::Internal{
            let left = Self::left_child(shifted_index);
            let right = Self::right_child(shifted_index);
            let left_contribution = unsafe{storage.get_unchecked(left.0).contribution()};
            let right_contribution = unsafe{storage.get_unchecked(right.0).contribution()};
            let total_contribution = left_contribution + right_contribution;

            let sample:C = rng.gen_range(C::zero()..total_contribution);
            if sample < left_contribution{
                shifted_index = left;
            }else{
                shifted_index = right;
            }
        }
        Ok(shifted_index)
    }
}
