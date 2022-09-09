use super::{EvalError, ProgramCache, Node, ProgramState, RegisterIndex, RegisterIndexAndType, RegisterType, RegisterValue};
use std::collections::HashSet;

pub struct NodeMoveAdvanced {
    target: RegisterIndexAndType,
    source: RegisterIndexAndType,
}

impl NodeMoveAdvanced {
    pub fn new(target: RegisterIndexAndType, source: RegisterIndexAndType) -> Self {
        Self {
            target: target,
            source: source,
        }
    }
}

impl Node for NodeMoveAdvanced {
    fn formatted_instruction(&self) -> String {
        format!("mov {},{}", self.target, self.source)
    }

    fn eval(&self, state: &mut ProgramState, _cache: &mut ProgramCache) -> Result<(), EvalError> {
        let tmp_value: RegisterValue;
        match self.source.register_type {
            RegisterType::Direct => {
                let value: &RegisterValue = state.get_register_value_ref(&self.source.register_index);
                tmp_value = value.clone();
            },
            RegisterType::Indirect => {
                // TODO: deal with indirect
                // let value: &RegisterValue = state.get_register_value_ref(&self.source.register_index);
                // // TODO: convert from value to register index
                // let value2: &RegisterValue = state.get_register_value_ref(value);
                // tmp_value = value2.clone();
                panic!("boom indirect source");
            }
        }
        match self.target.register_type {
            RegisterType::Direct => {
                state.set_register_value(self.target.register_index.clone(), tmp_value);
            },
            RegisterType::Indirect => {
                // TODO: deal with indirect
                panic!("boom indirect target");
            }
        }
        Ok(())
    }

    fn accumulate_register_indexes(&self, register_vec: &mut Vec<RegisterIndex>) {
        // TODO: deal with indirect
        register_vec.push(self.target.register_index.clone());
        register_vec.push(self.source.register_index.clone());
    }
    
    fn live_register_indexes(&self, register_set: &mut HashSet<RegisterIndex>) {
        // TODO: deal with indirect
        if register_set.contains(&self.source.register_index) {
            register_set.insert(self.target.register_index.clone());
        } else {
            // Overwrite content of the target register a non-live register.
            register_set.remove(&self.target.register_index);
        }
    }
}

pub struct NodeMoveRegister {
    target: RegisterIndex,
    source: RegisterIndex,
}

impl NodeMoveRegister {
    pub fn new(target: RegisterIndex, source: RegisterIndex) -> Self {
        Self {
            target: target,
            source: source,
        }
    }
}

impl Node for NodeMoveRegister {
    fn formatted_instruction(&self) -> String {
        format!("mov {},{}", self.target, self.source)
    }

    fn eval(&self, state: &mut ProgramState, _cache: &mut ProgramCache) -> Result<(), EvalError> {
        let value: &RegisterValue = state.get_register_value_ref(&self.source);
        let tmp_value: RegisterValue = value.clone();
        state.set_register_value(self.target.clone(), tmp_value);
        Ok(())
    }

    fn accumulate_register_indexes(&self, register_vec: &mut Vec<RegisterIndex>) {
        register_vec.push(self.target.clone());
        register_vec.push(self.source.clone());
    }
    
    fn live_register_indexes(&self, register_set: &mut HashSet<RegisterIndex>) {
        if register_set.contains(&self.source) {
            register_set.insert(self.target.clone());
        } else {
            // Overwrite content of the target register a non-live register.
            register_set.remove(&self.target);
        }
    }
}

pub struct NodeMoveConstant {
    target: RegisterIndex,
    source: RegisterValue,
}

impl NodeMoveConstant {
    pub fn new(target: RegisterIndex, source: RegisterValue) -> Self {
        Self {
            target: target,
            source: source,
        }
    }
}

impl Node for NodeMoveConstant {
    fn formatted_instruction(&self) -> String {
        format!("mov {},{}", self.target, self.source)
    }

    fn eval(&self, state: &mut ProgramState, _cache: &mut ProgramCache) -> Result<(), EvalError> {
        let value: RegisterValue = self.source.clone();
        state.set_register_value(self.target.clone(), value);
        Ok(())
    }

    fn accumulate_register_indexes(&self, register_vec: &mut Vec<RegisterIndex>) {
        register_vec.push(self.target.clone());
    }
    
    fn live_register_indexes(&self, register_set: &mut HashSet<RegisterIndex>) {
        // Overwrite content of the target register a non-live register.
        register_set.remove(&self.target);
    }
}
