use oxc_syntax::symbol::SymbolId;

use crate::types::ModuleIdx;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SymbolRef {
    pub owner: ModuleIdx,
    pub symbol: SymbolId,
}

impl From<(ModuleIdx, SymbolId)> for SymbolRef {
    fn from((owner, symbol): (ModuleIdx, SymbolId)) -> Self {
        Self { owner, symbol }
    }
}
