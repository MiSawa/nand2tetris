use crate::jack::token::Identifier;
use anyhow::{bail, Result};
use enum_map::EnumMap;
use std::collections::HashMap;
use crate::jack::symbol_table::Kind::Argument;

#[derive(Debug, Enum, Copy, Clone, Eq, PartialEq)]
pub enum Scope {
    Class = 0,
    Subroutine = 1,
}
#[derive(Debug, Enum, Copy, Clone, Eq, PartialEq)]
pub enum Kind {
    Static = 0,
    Field = 1,
    Argument = 2,
    Local = 3,
}
#[derive(Default, Debug)]
pub struct SymbolTable {
    tables: EnumMap<Scope, HashMap<Identifier, (Kind, VariableType, u16)>>,
    counts: EnumMap<Kind, u16>,
}

#[derive(Debug, Clone)]
pub enum VariableType {
    Int,
    Char,
    Boolean,
    Object(Identifier),
}

impl SymbolTable {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn start_new_class(&mut self) {
        self.tables.iter_mut().for_each(|(_, v)| v.clear());
        self.counts.iter_mut().for_each(|(_, v)| *v = 0);
    }
    pub fn start_new_subroutine(&mut self) {
        self.tables[Scope::Subroutine].clear();
        self.counts[Kind::Argument] = 0;
        self.counts[Kind::Local] = 0;
    }
    fn define_variable(
        &mut self,
        scope: &Scope,
        kind: &Kind,
        t: &VariableType,
        name: Identifier,
    ) -> Result<()> {
        if self.tables[*scope].contains_key(&name) {
            bail!(
                "Variable {:?} is already define in the scope {:?}",
                &name,
                scope
            );
        }
        let id = self.counts[*kind];
        self.counts[*kind] += 1;
        self.tables[*scope].insert(name, (kind.clone(), t.clone(), id));
        Ok(())
    }
    pub fn define_class_variable(
        &mut self,
        kind: &Kind,
        t: &VariableType,
        name: Identifier,
    ) -> Result<()> {
        match kind {
            Kind::Static | Kind::Field => self.define_variable(&Scope::Class, kind, t, name),
            _ => bail!("{:?} is not a class var", kind),
        }
    }
    pub fn define_argument_variable(&mut self, t: &VariableType, name: Identifier) -> Result<()> {
        self.define_variable(&Scope::Subroutine, &Kind::Argument, t, name)
    }
    pub fn define_local_variable(&mut self, t: &VariableType, name: Identifier) -> Result<()> {
        self.define_variable(&Scope::Subroutine, &Kind::Local, t, name)
    }
    pub fn get_count(&self, kind: Kind) -> u16 {
        self.counts[kind]
    }
    pub fn shift_argument_variables_by_one(&mut self) {
        self.counts[Kind::Argument] += 1;
        self.tables[Scope::Subroutine].iter_mut().for_each(|(_, (kind, _, id))| {
            if *kind == Argument {
                *id += 1;
            }
        });
    }

    pub fn lookup(&self, name: &Identifier) -> Option<(Kind, VariableType, u16)> {
        self.tables[Scope::Subroutine]
            .get(&name)
            .or_else(|| self.tables[Scope::Class].get(name))
            .cloned()
    }
}
