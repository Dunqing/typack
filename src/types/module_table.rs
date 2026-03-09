use std::ops::{Index, IndexMut};

use oxc_index::IndexVec;

use crate::types::ModuleIdx;
use crate::types::module::Module;

pub struct ModuleTable<'a>(IndexVec<ModuleIdx, Module<'a>>);

impl<'a> ModuleTable<'a> {
    pub fn new() -> Self {
        Self(IndexVec::new())
    }

    pub fn from_raw_parts(modules: IndexVec<ModuleIdx, Module<'a>>) -> Self {
        Self(modules)
    }

    pub fn push(&mut self, module: Module<'a>) -> ModuleIdx {
        self.0.push(module)
    }

    pub fn iter(&self) -> impl Iterator<Item = &Module<'a>> {
        self.0.iter()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn get(&self, index: ModuleIdx) -> Option<&Module<'a>> {
        self.0.get(index)
    }
}

impl<'a> Index<ModuleIdx> for ModuleTable<'a> {
    type Output = Module<'a>;
    fn index(&self, index: ModuleIdx) -> &Self::Output {
        &self.0[index]
    }
}

impl IndexMut<ModuleIdx> for ModuleTable<'_> {
    fn index_mut(&mut self, index: ModuleIdx) -> &mut Self::Output {
        &mut self.0[index]
    }
}

impl<'a, 'b> IntoIterator for &'b ModuleTable<'a> {
    type Item = &'b Module<'a>;
    type IntoIter = std::slice::Iter<'b, Module<'a>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}
