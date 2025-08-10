use crate::internal::Block;

#[repr(transparent)] // No clue what this means :)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ID(usize);

impl From<usize> for ID {
    fn from(value: usize) -> Self {
        ID(value)
    }
}
impl From<ID> for usize {
    fn from(id: ID) -> Self {
        id.0
    }
}
impl<T> std::ops::Index<ID> for Vec<T> {
    type Output = T;
    fn index(&self, index: ID) -> &Self::Output {
        &self[index.0]
    }
}
impl<T> std::ops::IndexMut<ID> for Vec<T> {
    fn index_mut(&mut self, index: ID) -> &mut Self::Output {
        &mut self[index.0]
    }
}
impl std::fmt::Display for ID {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}


pub trait UsizeGetID {
    fn get_id_of_idx(self, blocks: &Vec<Block>) -> ID;
}

impl UsizeGetID for usize {
    fn get_id_of_idx(self, blocks: &Vec<Block>) -> ID {
        blocks[self].id
    }
}
