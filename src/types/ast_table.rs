use std::ops::{Index, IndexMut};

use oxc_ast::ast::Program;
use oxc_index::IndexVec;

use crate::types::ModuleIdx;

pub struct AstTable<'a>(IndexVec<ModuleIdx, Program<'a>>);

impl Default for AstTable<'_> {
    fn default() -> Self {
        Self(IndexVec::new())
    }
}

impl<'a> AstTable<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, program: Program<'a>) -> ModuleIdx {
        self.0.push(program)
    }

    pub fn into_raw_vec(self) -> Vec<Program<'a>> {
        self.0.into_iter().collect()
    }

    pub fn from_raw_parts(programs: IndexVec<ModuleIdx, Program<'a>>) -> Self {
        Self(programs)
    }
}

impl<'a> Index<ModuleIdx> for AstTable<'a> {
    type Output = Program<'a>;
    fn index(&self, index: ModuleIdx) -> &Self::Output {
        &self.0[index]
    }
}

impl IndexMut<ModuleIdx> for AstTable<'_> {
    fn index_mut(&mut self, index: ModuleIdx) -> &mut Self::Output {
        &mut self.0[index]
    }
}
