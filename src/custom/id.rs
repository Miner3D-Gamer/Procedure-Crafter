#[repr(transparent)]
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
