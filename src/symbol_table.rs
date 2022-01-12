use crate::operation::VarScope;

#[derive(Debug, Clone)]
pub struct VariableSymbolItem {
  name: String,
  v_type: String,
  kind: VarScope,
  index: usize,
}

impl VariableSymbolItem {
  pub fn new(name: String, v_type: String, kind: VarScope, index: usize) -> Self {
    Self {
      name,
      v_type,
      kind,
      index,
    }
  }

  pub fn get_kind(&self) -> VarScope {
    self.kind
  }

  pub fn get_idx(&self) -> usize {
    self.index
  }

  pub fn get_type(&self) -> &String {
    &self.v_type
  }
}

pub struct SymbolTable(Vec<VariableSymbolItem>, bool);

impl SymbolTable {
  pub fn new() -> Self {
    Self {
      0: vec![],
      1: false,
    }
  }

  pub fn disable_field(&mut self) {
    self.1 = true;
  }

  pub fn enable_field(&mut self) {
    self.1 = false;
  }

  pub fn push_item(&mut self, name: String, v_type: String, kind: VarScope) {
    let item = VariableSymbolItem::new(name, v_type, kind.clone(), self.scope_item_count(kind));
    self.0.push(item);
  }

  pub fn find_item_by_name(&self, name: &String) -> Option<&VariableSymbolItem> {
    for item in &self.0 {
      if self.1 && item.kind == VarScope::Field {
        continue;
      }
      if item.name == *name {
        return Some(item);
      }
    }
    return None;
  }

  pub fn scope_item_count(&self, kind: VarScope) -> usize {
    let mut idx = 0;
    for item in &self.0 {
      if item.kind == kind {
        idx += 1;
      }
    }
    idx
  }

  pub fn clear(&mut self) {
    self.0.clear();
  }
}
