use super::{GenomeItem, GenomeMutateContext, LineValue, MutateEvalSequenceCategory, SourceValue, TargetValue, ToGenomeItem, ToGenomeItemVec};
use loda_rust_core::control::DependencyManager;
use loda_rust_core::execute::RegisterType;
use loda_rust_core::parser::{Instruction, InstructionId, InstructionParameter, ParameterType};
use loda_rust_core::parser::ParsedProgram;
use std::collections::HashSet;
use std::fmt;
use rand::Rng;
use rand::seq::SliceRandom;
use std::fs;
use std::path::PathBuf;

#[derive(Debug)]
#[allow(dead_code)]
pub enum MutateGenome {
    ReplaceInstructionWithHistogram,
    InsertInstructionWithConstant,
    IncrementSourceValueWhereTypeIsConstant,
    DecrementSourceValueWhereTypeIsConstant,
    ReplaceSourceConstantWithHistogram,
    SetSourceToConstant,
    SetSourceToDirect,
    DisableLoop,
    SwapRegisters,
    IncrementSourceValueWhereTypeIsDirect,
    DecrementSourceValueWhereTypeIsDirect,
    ReplaceSourceWithHistogram,
    IncrementTargetValueWhereTypeIsDirect,
    DecrementTargetValueWhereTypeIsDirect,
    ReplaceTargetWithHistogram,
    ReplaceLineWithHistogram,
    InsertLineWithHistogram,
    CopyLine,
    ToggleEnabled,
    SwapRows,
    SwapAdjacentRows,
    InsertLoopBeginEnd,
    CallProgramWeightedByPopularity,
    CallMostPopularProgram,
    CallMediumPopularProgram,
    CallLeastPopularProgram,
    CallRecentProgram,
    CallProgramThatUsesIndirectMemoryAccess,
}

pub struct Genome {
    genome_vec: Vec<GenomeItem>,
    message_vec: Vec<String>,
}

impl Genome {
    const MUTATE_RETRIES: usize = 3;

    pub fn new() -> Self {
        Self {
            genome_vec: vec!(),
            message_vec: vec!(),
        }
    }

    #[allow(dead_code)]
    pub fn contains_indirect_memory_access(&self) -> bool {
        for genome_item in &self.genome_vec {
            if genome_item.contains_indirect_memory_access() {
                return true;
            }
        }
        false
    }

    pub fn depends_on_program_ids(&self) -> HashSet<u32> {
        let mut program_ids = HashSet::<u32>::new();
        for genome_item in &self.genome_vec {
            if !genome_item.is_enabled() {
                continue;
            }
            if genome_item.instruction_id() != InstructionId::EvalSequence {
                continue;
            }
            let program_id_raw: i32 = genome_item.source_value();
            if program_id_raw < 0 {
                continue;
            }
            program_ids.insert(program_id_raw as u32);
        }
        program_ids
    }

    pub fn load_program_with_id(dm: &DependencyManager, program_id: u64) -> anyhow::Result<ParsedProgram> {
        let path_to_program: PathBuf = dm.path_to_program(program_id);
        let contents: String = match fs::read_to_string(&path_to_program) {
            Ok(value) => value,
            Err(error) => {
                return Err(anyhow::anyhow!("load_program_with_id program_id: {:?}, cannot read the file: {:?}", program_id, error));
            }
        };
        let parsed_program: ParsedProgram = match ParsedProgram::parse_program(&contents) {
            Ok(value) => value,
            Err(error) => {
                return Err(anyhow::anyhow!("load_program_with_id program_id: {:?}, cannot parse the program: {:?}", program_id, error));
            }
        };
        Ok(parsed_program)
    }

    pub fn set_genome_vec(&mut self, genome_vec: Vec<GenomeItem>) {
        self.genome_vec = genome_vec;
    }

    pub fn to_parsed_program(&self) -> ParsedProgram {
        let mut instruction_vec = Vec::<Instruction>::with_capacity(self.genome_vec.len());

        let mut line_number: usize = 0;
        for genome_item in self.genome_vec.iter() {
            if !genome_item.is_enabled() {
                continue;
            }

            let instruction_id: InstructionId = 
                genome_item.instruction_id();
    
            let parameter_vec: Vec<InstructionParameter> = 
                genome_item.to_parameter_vec();
    
            let instruction = Instruction {
                instruction_id: instruction_id,
                parameter_vec: parameter_vec,
                line_number: line_number,
            };
            instruction_vec.push(instruction);
            line_number += 1;
        }

        ParsedProgram {
            optional_offset: None,
            instruction_vec: instruction_vec
        }
    }

    pub fn message_vec(&self) -> &Vec<String> {
        &self.message_vec
    }
    
    pub fn set_message_vec(&mut self, message_vec: Vec<String>) {
        self.message_vec = message_vec;
    }

    pub fn append_message(&mut self, message: String) {
        self.message_vec.push(message);
    }

    /// Increment the source value.
    ///
    /// Only impact rows where source_type=Constant.
    ///
    /// Return `true` when the mutation was successful.
    /// 
    /// Return `false` in case the mutation had no effect.
    pub fn increment_source_value_where_type_is_constant<R: Rng + ?Sized>(&mut self, rng: &mut R) -> bool {
        let mut indexes: Vec<usize> = vec!();
        for (index, genome_item) in self.genome_vec.iter().enumerate() {
            if genome_item.is_mutation_locked() {
                continue;
            }
            if genome_item.source_type() != ParameterType::Constant {
                continue;
            }
            match genome_item.instruction_id() {
                InstructionId::EvalSequence | 
                InstructionId::LoopBegin | 
                InstructionId::LoopEnd |
                InstructionId::UnofficialFunction { .. } |
                InstructionId::UnofficialLoopBeginSubtract => {
                    continue;
                },
                _ => {}
            }
            indexes.push(index);
        }
        if indexes.is_empty() {
            return false;
        }

        // Mutate one of the instructions
        let index: &usize = indexes.choose(rng).unwrap();
        let genome_item: &mut GenomeItem = &mut self.genome_vec[*index];
        let value: i32 = genome_item.source_value();
        if value >= i32::MAX {
            return false;
        }
        let new_value = value + 1;
        if genome_item.instruction_id() == InstructionId::Divide && new_value == 0 {
            return false;
        }
        if genome_item.instruction_id() == InstructionId::DivideIf && new_value == 0 {
            return false;
        }
        if genome_item.instruction_id() == InstructionId::Modulo && new_value == 0 {
            return false;
        }
        genome_item.set_source_value(new_value);
        true
    }

    /// Decrement the source value.
    ///
    /// Only impact rows where source_type=Constant.
    ///
    /// Return `true` when the mutation was successful.
    /// 
    /// Return `false` in case the mutation had no effect.
    pub fn decrement_source_value_where_type_is_constant<R: Rng + ?Sized>(&mut self, rng: &mut R) -> bool {
        let mut indexes: Vec<usize> = vec!();
        for (index, genome_item) in self.genome_vec.iter().enumerate() {
            if genome_item.is_mutation_locked() {
                continue;
            }
            if genome_item.source_type() != ParameterType::Constant {
                continue;
            }
            match genome_item.instruction_id() {
                InstructionId::EvalSequence | 
                InstructionId::LoopBegin | 
                InstructionId::LoopEnd |
                InstructionId::UnofficialFunction { .. } |
                InstructionId::UnofficialLoopBeginSubtract => {
                    continue;
                },
                _ => {}
            }
            indexes.push(index);
        }
        if indexes.is_empty() {
            return false;
        }

        // Mutate one of the instructions
        let index: &usize = indexes.choose(rng).unwrap();
        let genome_item: &mut GenomeItem = &mut self.genome_vec[*index];
        let value: i32 = genome_item.source_value();
        if value <= i32::MIN {
            return false;
        }
        let new_value = value - 1;
        if genome_item.instruction_id() == InstructionId::Divide && new_value == 0 {
            return false;
        }
        if genome_item.instruction_id() == InstructionId::DivideIf && new_value == 0 {
            return false;
        }
        if genome_item.instruction_id() == InstructionId::Modulo && new_value == 0 {
            return false;
        }
        genome_item.set_source_value(new_value);
        true
    }

    /// Assign a constant, by picking from a histogram.
    /// 
    /// The histogram has knowledge about each instruction.
    /// 
    /// If it's an `add` instruction, then the most used constant is 1.
    /// 
    /// If it's a `mul` instruction, then the most used constant is 2.
    /// 
    /// If it's a `cmp` instruction, then the most used constant is 0.
    ///
    /// There is a high probability that this function assigns a constant
    /// that is highly used across all programs.
    ///
    /// There is a low probablility that this function assigns a constant
    /// that is sporadic used across all programs.
    ///
    /// This function does not assign a constant that has never been
    /// used elsewhere. So it doesn't explore new never tried out magic constants.
    ///
    /// Return `true` when the mutation was successful.
    /// 
    /// Return `false` in case of failure, such as no instructions that use a constant, underflow, overflow.
    pub fn replace_source_constant_with_histogram<R: Rng + ?Sized>(&mut self, rng: &mut R, context: &GenomeMutateContext) -> bool {
        // Bail out if the histogram csv file hasn't been loaded.
        if !context.has_histogram_instruction_constant() {
            return false;
        }

        // Identify the instructions that use constants
        let mut indexes: Vec<usize> = vec!();
        for (index, genome_item) in self.genome_vec.iter().enumerate() {
            if genome_item.is_mutation_locked() {
                continue;
            }
            if genome_item.source_type() != ParameterType::Constant {
                continue;
            }
            match genome_item.instruction_id() {
                InstructionId::EvalSequence | 
                InstructionId::LoopBegin | 
                InstructionId::LoopEnd |
                InstructionId::UnofficialFunction { .. } |
                InstructionId::UnofficialLoopBeginSubtract => {
                    continue;
                },
                _ => {}
            }
            indexes.push(index);
        }
        if indexes.is_empty() {
            return false;
        }

        // Mutate one of the instructions that use a constant
        let index: &usize = indexes.choose(rng).unwrap();
        let genome_item: &mut GenomeItem = &mut self.genome_vec[*index];
        let instruction_id: InstructionId = genome_item.instruction_id();

        // Try a few times
        for _ in 0..Self::MUTATE_RETRIES {
            let picked_value: i32 = match context.choose_constant_with_histogram(rng, instruction_id) {
                Some(value) => value,
                None => {
                    // No entry for this instruction
                    return false;
                }
            };
            if picked_value == genome_item.source_value() {
                // Picked the same as the original, try pick a different value
                continue;
            }
            genome_item.set_source_value(picked_value);
            return true;
        }
        // Too many tries, without picking a different value. No mutation happened.
        false
    }

    /// Increment the source value.
    ///
    /// Only impact rows where source_type=Direct.
    ///
    /// Return `true` when the mutation was successful.
    /// 
    /// Return `false` in case the mutation had no effect.
    pub fn increment_source_value_where_type_is_direct<R: Rng + ?Sized>(&mut self, rng: &mut R) -> bool {
        let mut indexes: Vec<usize> = vec!();
        for (index, genome_item) in self.genome_vec.iter().enumerate() {
            if genome_item.is_mutation_locked() {
                continue;
            }
            if genome_item.source_type() != ParameterType::Direct {
                continue;
            }
            match genome_item.instruction_id() {
                InstructionId::LoopBegin | 
                InstructionId::LoopEnd |
                InstructionId::UnofficialFunction { .. } |
                InstructionId::UnofficialLoopBeginSubtract => {
                    continue;
                },
                _ => {}
            }
            indexes.push(index);
        }
        if indexes.is_empty() {
            return false;
        }

        // Mutate one of the instructions
        let index: &usize = indexes.choose(rng).unwrap();
        let genome_item: &mut GenomeItem = &mut self.genome_vec[*index];
        let value: i32 = genome_item.source_value();
        if value >= i32::MAX {
            return false;
        }
        let new_value = value + 1;
        genome_item.set_source_value(new_value);
        true
    }

    /// Decrement the source value.
    ///
    /// Only impact rows where source_type=Direct.
    ///
    /// Return `true` when the mutation was successful.
    /// 
    /// Return `false` in case the mutation had no effect.
    pub fn decrement_source_value_where_type_is_direct<R: Rng + ?Sized>(&mut self, rng: &mut R) -> bool {
        let mut indexes: Vec<usize> = vec!();
        for (index, genome_item) in self.genome_vec.iter().enumerate() {
            if genome_item.is_mutation_locked() {
                continue;
            }
            if genome_item.source_type() != ParameterType::Direct {
                continue;
            }
            match genome_item.instruction_id() {
                InstructionId::LoopBegin | 
                InstructionId::LoopEnd |
                InstructionId::UnofficialFunction { .. } |
                InstructionId::UnofficialLoopBeginSubtract => {
                    continue;
                },
                _ => {}
            }
            if genome_item.source_value() <= 0 {
                continue;
            }
            indexes.push(index);
        }
        if indexes.is_empty() {
            return false;
        }

        // Mutate one of the instructions
        let index: &usize = indexes.choose(rng).unwrap();
        let genome_item: &mut GenomeItem = &mut self.genome_vec[*index];
        let value: i32 = genome_item.source_value();
        if value <= i32::MIN {
            return false;
        }
        let new_value = value - 1;
        genome_item.set_source_value(new_value);
        true
    }

    fn get_source_value(genome_item: &GenomeItem) -> SourceValue {
        let instruction_id: InstructionId = genome_item.instruction_id();
        if instruction_id == InstructionId::LoopEnd {
            return SourceValue::None;
        }
        let value: i32 = genome_item.source_value();
        match genome_item.source_type() {
            ParameterType::Constant => {
                return SourceValue::Constant(value);
            },
            ParameterType::Direct => {
                return SourceValue::Direct(value);
            },
            ParameterType::Indirect => {
                return SourceValue::Indirect(value);
            }
        }
    }

    /// Return `true` when the mutation was successful.
    /// 
    /// Return `false` in case of failure, such as empty genome, bad parameters for instruction.
    pub fn replace_source_with_histogram<R: Rng + ?Sized>(&mut self, rng: &mut R, context: &GenomeMutateContext) -> bool {
        // Bail out if the trigram.csv file hasn't been loaded.
        if !context.has_suggest_source() {
            return false;
        }

        let mut indexes: Vec<usize> = vec!();
        for (index, genome_item) in self.genome_vec.iter().enumerate() {
            if genome_item.is_mutation_locked() {
                continue;
            }

            // Don't make any changes to the `loop range length` parameter.
            // It makes it hard to make sense of what is going on in the loop.
            // It's a valid construct, but it's not desired.
            // That's why `lpb` is skipped.
            match genome_item.instruction_id() {
                InstructionId::EvalSequence | 
                InstructionId::LoopBegin | 
                InstructionId::LoopEnd |
                InstructionId::UnofficialFunction { .. } |
                InstructionId::UnofficialLoopBeginSubtract => {
                    continue;
                },
                _ => {}
            }
            indexes.push(index);
        }
        if indexes.is_empty() {
            return false;
        }

        // Mutate one of the instructions
        let index1: usize = indexes.choose(rng).unwrap().clone();
        let index0: i32 = (index1 as i32) - 1;
        let index2: usize = index1 + 1;
        let mut prev_word: SourceValue = SourceValue::ProgramStart;
        if index0 >= 0 {
            match self.genome_vec.get(index0 as usize) {
                Some(ref value) => {
                    prev_word = Self::get_source_value(value)
                },
                None => {}
            };
        }
        let mut next_word: SourceValue = SourceValue::ProgramStop;
        match self.genome_vec.get(index2) {
            Some(ref value) => {
                next_word = Self::get_source_value(value)
            },
            None => {}
        };
        if prev_word == SourceValue::ProgramStart && next_word == SourceValue::ProgramStop {
            return false;
        }
        let genome_item: &mut GenomeItem = &mut self.genome_vec[index1];

        // Try a few times
        for _ in 0..Self::MUTATE_RETRIES {
            let suggested_value: SourceValue = match context.suggest_source(rng, prev_word, next_word) {
                Some(value) => value,
                None => {
                    continue;
                }
            };
            let parameter_value: i32;
            let parameter_type: ParameterType;
            match suggested_value {
                SourceValue::Constant(value) => {
                    parameter_type = ParameterType::Constant;
                    parameter_value = value; 
                },
                SourceValue::Direct(value) => {
                    if value < 0 {
                        continue;
                    }
                    parameter_type = ParameterType::Direct;
                    parameter_value = value; 
                },
                SourceValue::Indirect(value) => {
                    if value < 0 {
                        continue;
                    }
                    parameter_type = ParameterType::Indirect;
                    parameter_value = value; 
                },
                _ => {
                    continue;
                }
            };
            let same_value: bool = parameter_value == genome_item.source_value();
            let same_type: bool = parameter_type == genome_item.source_type();
            if same_value && same_type {
                continue;
            }
            genome_item.set_source_value(parameter_value);
            genome_item.set_source_type(parameter_type);

            // Successfully picked a good value/type
            return true;
        }

        // Too many tries, without picking a different value. No mutation happened.
        false
    }
    

    /// Return `true` when the mutation was successful.
    /// 
    /// Return `false` in case of failure, such as empty genome, bad parameters for instruction.
    pub fn replace_line_with_histogram<R: Rng + ?Sized>(&mut self, rng: &mut R, context: &GenomeMutateContext) -> bool {
        // Bail out if the trigram.csv file hasn't been loaded.
        if !context.has_suggest_line() {
            return false;
        }
        let mut indexes: Vec<usize> = vec!();
        for (index, genome_item) in self.genome_vec.iter().enumerate() {
            if genome_item.is_mutation_locked() {
                continue;
            }
            // Don't make changes to the the loop instructions `lpb` and `lpe` and the unofficial `lps`.
            match genome_item.instruction_id() {
                InstructionId::LoopBegin | 
                InstructionId::LoopEnd |
                InstructionId::UnofficialLoopBeginSubtract => {
                    continue;
                },
                _ => {}
            }
            indexes.push(index);
        }
        if indexes.is_empty() {
            return false;
        }

        // Mutate one of the instructions
        let index1: usize = indexes.choose(rng).unwrap().clone();
        let index0: i32 = (index1 as i32) - 1;
        let index2: usize = index1 + 1;
        let mut prev_word: LineValue = LineValue::ProgramStart;
        if index0 >= 0 {
            match self.genome_vec.get(index0 as usize) {
                Some(ref value) => {
                    let s: String = value.to_line_string();
                    prev_word = LineValue::Line(s);
                },
                None => {}
            };
        }
        let mut next_word: LineValue = LineValue::ProgramStop;
        match self.genome_vec.get(index2) {
            Some(ref value) => {
                let s: String = value.to_line_string();
                next_word = LineValue::Line(s);
        },
            None => {}
        };
        if prev_word == LineValue::ProgramStart && next_word == LineValue::ProgramStop {
            return false;
        }
        let suggested_value: LineValue = match context.suggest_line(rng, prev_word, next_word) {
            Some(value) => value,
            None => {
                return false;
            }
        };

        let line_content: String = match suggested_value {
            LineValue::Line(value) => value,
            LineValue::ProgramStart => {
                return false;
            },
            LineValue::ProgramStop => {
                return false;
            },
        };

        let parsed_program: ParsedProgram = match ParsedProgram::parse_program(&line_content) {
            Ok(value) => value,
            Err(_) => {
                return false;
            }
        };
        
        let row0: Instruction = match parsed_program.instruction_vec.first() {
            Some(value) => value.clone(),
            None => {
                return false;
            }
        };

        let genome_item: GenomeItem = match row0.to_genome_item() {
            Some(value) => value,
            None => {
                return false;
            }
        };

        match genome_item.instruction_id() {
            InstructionId::LoopBegin | 
            InstructionId::LoopEnd |
            InstructionId::UnofficialLoopBeginSubtract => {
                return false;
            },
            _ => {}
        }

        self.genome_vec[index1] = genome_item;
        true
    }
    
    /// Insert an entire line considering the surrounding lines.
    /// 
    /// Return `true` when the mutation was successful.
    /// 
    /// Return `false` in case of failure, such as empty genome, bad parameters for instruction.
    pub fn insert_line_with_histogram<R: Rng + ?Sized>(&mut self, rng: &mut R, context: &GenomeMutateContext) -> bool {
        // Bail out if the trigram.csv file hasn't been loaded.
        if !context.has_suggest_line() {
            return false;
        }
        let length: usize = self.genome_vec.len();
        if length < 1 {
            return false;
        }

        // Decide on where to insert a new GenomeItem
        let index1: usize = rng.gen_range(0..length);
        let index0: i32 = (index1 as i32) - 1;
        let index2: usize = index1;

        // Gather info about the "previous" GenomeItem
        let mut prev_word: LineValue = LineValue::ProgramStart;
        if index0 >= 0 {
            match self.genome_vec.get(index0 as usize) {
                Some(ref value) => {
                    let s: String = value.to_line_string();
                    prev_word = LineValue::Line(s);
                },
                None => {}
            };
        }
        // Gather info about the "next" GenomeItem
        let mut next_word: LineValue = LineValue::ProgramStop;
        match self.genome_vec.get(index2) {
            Some(ref value) => {
                let s: String = value.to_line_string();
                next_word = LineValue::Line(s);
        },
            None => {}
        };
        if prev_word == LineValue::ProgramStart && next_word == LineValue::ProgramStop {
            return false;
        }
        let suggested_value: LineValue = match context.suggest_line(rng, prev_word, next_word) {
            Some(value) => value,
            None => {
                return false;
            }
        };

        let line_content: String = match suggested_value {
            LineValue::Line(value) => value,
            LineValue::ProgramStart => {
                return false;
            },
            LineValue::ProgramStop => {
                return false;
            },
        };

        let parsed_program: ParsedProgram = match ParsedProgram::parse_program(&line_content) {
            Ok(value) => value,
            Err(_) => {
                return false;
            }
        };
        
        let row0: Instruction = match parsed_program.instruction_vec.first() {
            Some(value) => value.clone(),
            None => {
                return false;
            }
        };

        let genome_item: GenomeItem = match row0.to_genome_item() {
            Some(value) => value,
            None => {
                return false;
            }
        };

        match genome_item.instruction_id() {
            InstructionId::LoopBegin | 
            InstructionId::LoopEnd |
            InstructionId::UnofficialLoopBeginSubtract => {
                return false;
            },
            _ => {}
        }

        self.genome_vec.insert(index1, genome_item);
        true
    }

    /// Make a copy of an existing line.
    /// 
    /// Return `true` when the mutation was successful.
    /// 
    /// Return `false` in case of failure, such as empty genome, bad parameters for instruction.
    pub fn copy_line<R: Rng + ?Sized>(&mut self, rng: &mut R) -> bool {
        let mut indexes: Vec<usize> = vec!();
        for (index, genome_item) in self.genome_vec.iter().enumerate() {
            // We are not interested in copying loop related instructions, since loop require balancing start/end of the scope.
            match genome_item.instruction_id() {
                InstructionId::LoopBegin | 
                InstructionId::LoopEnd |
                InstructionId::UnofficialLoopBeginSubtract => {
                    continue;
                },
                _ => {}
            }
            indexes.push(index);
        }
        if indexes.is_empty() {
            return false;
        }

        // Make a copy
        let copy_from_index: &usize = indexes.choose(rng).unwrap();
        let genome_item: GenomeItem = self.genome_vec[*copy_from_index].clone();

        let length: usize = self.genome_vec.len();
        if length < 1 {
            return false;
        }

        // Insert a new line into the program
        let insert_index: usize = rng.gen_range(0..length);
        self.genome_vec.insert(insert_index, genome_item);
        true
    }

    /// Increment the target value.
    ///
    /// Only impact rows where target_type=Direct.
    ///
    /// Return `true` when the mutation was successful.
    /// 
    /// Return `false` in case the mutation had no effect.
    pub fn increment_target_value_where_type_is_direct<R: Rng + ?Sized>(&mut self, rng: &mut R) -> bool {
        let mut indexes: Vec<usize> = vec!();
        for (index, genome_item) in self.genome_vec.iter().enumerate() {
            if genome_item.is_mutation_locked() {
                continue;
            }
            if genome_item.target_type() != RegisterType::Direct {
                continue;
            }
            if genome_item.instruction_id() == InstructionId::LoopEnd {
                continue;
            }
            indexes.push(index);
        }
        if indexes.is_empty() {
            return false;
        }

        // Mutate one of the instructions
        let index: &usize = indexes.choose(rng).unwrap();
        let genome_item: &mut GenomeItem = &mut self.genome_vec[*index];
        let value: i32 = genome_item.target_value();
        if value >= i32::MAX {
            return false;
        }
        let new_value = value + 1;
        genome_item.set_target_value(new_value);
        true
    }

    /// Decrement the target value.
    ///
    /// Only impact rows where target_type=Direct.
    ///
    /// Return `true` when the mutation was successful.
    /// 
    /// Return `false` in case the mutation had no effect.
    pub fn decrement_target_value_where_type_is_direct<R: Rng + ?Sized>(&mut self, rng: &mut R) -> bool {
        let mut indexes: Vec<usize> = vec!();
        for (index, genome_item) in self.genome_vec.iter().enumerate() {
            if genome_item.is_mutation_locked() {
                continue;
            }
            if genome_item.target_type() != RegisterType::Direct {
                continue;
            }
            if genome_item.instruction_id() == InstructionId::LoopEnd {
                continue;
            }
            if genome_item.target_value() <= 0 {
                continue;
            }
            indexes.push(index);
        }
        if indexes.is_empty() {
            return false;
        }

        // Mutate one of the instructions
        let index: &usize = indexes.choose(rng).unwrap();
        let genome_item: &mut GenomeItem = &mut self.genome_vec[*index];
        let value: i32 = genome_item.target_value();
        if value <= i32::MIN {
            return false;
        }
        let new_value = value - 1;
        genome_item.set_target_value(new_value);
        true
    }

    fn get_target_value(genome_item: &GenomeItem) -> TargetValue {
        let instruction_id: InstructionId = genome_item.instruction_id();
        if instruction_id == InstructionId::LoopEnd {
            return TargetValue::None;
        }
        let value: i32 = genome_item.target_value();
        match genome_item.target_type() {
            RegisterType::Direct => {
                return TargetValue::Direct(value);
            },
            RegisterType::Indirect => {
                return TargetValue::Indirect(value);
            },
        }
    }

    /// Return `true` when the mutation was successful.
    /// 
    /// Return `false` in case of failure, such as empty genome, bad parameters for instruction.
    pub fn replace_target_with_histogram<R: Rng + ?Sized>(&mut self, rng: &mut R, context: &GenomeMutateContext) -> bool {
        // Bail out if the trigram.csv file hasn't been loaded.
        if !context.has_suggest_target() {
            return false;
        }
        let length: usize = self.genome_vec.len();
        if length < 1 {
            return false;
        }
        let index1: usize = rng.gen_range(0..length);
        let index0: i32 = (index1 as i32) - 1;
        let index2: usize = index1 + 1;
        let mut prev_word: TargetValue = TargetValue::ProgramStart;
        if index0 >= 0 {
            match self.genome_vec.get(index0 as usize) {
                Some(ref value) => {
                    prev_word = Self::get_target_value(value)
                },
                None => {}
            };
        }
        let mut next_word: TargetValue = TargetValue::ProgramStop;
        match self.genome_vec.get(index2) {
            Some(ref value) => {
                next_word = Self::get_target_value(value)
            },
            None => {}
        };
        let genome_item: &mut GenomeItem = &mut self.genome_vec[index1];
        if genome_item.is_mutation_locked() {
            return false;
        }
        if genome_item.instruction_id() == InstructionId::LoopEnd {
            return false;
        }

        // Try a few times
        for _ in 0..Self::MUTATE_RETRIES {
            let suggested_value: TargetValue = match context.suggest_target(rng, prev_word, next_word) {
                Some(value) => value,
                None => {
                    continue;
                }
            };
            let parameter_value: i32;
            let parameter_type: RegisterType;
            match suggested_value {
                TargetValue::Direct(value) => {
                    if value < 0 {
                        continue;
                    }
                    parameter_type = RegisterType::Direct;
                    parameter_value = value; 
                },
                TargetValue::Indirect(value) => {
                    if value < 0 {
                        continue;
                    }
                    parameter_type = RegisterType::Indirect;
                    parameter_value = value; 
                },
                _ => {
                    continue;
                }
            };
            let same_value: bool = parameter_value == genome_item.target_value();
            let same_type: bool = parameter_type == genome_item.target_type();
            if same_value && same_type {
                continue;
            }
            genome_item.set_target_value(parameter_value);
            genome_item.set_target_type(parameter_type);

            // Successfully picked a good value/type
            return true;
        }
        // Too many tries, without picking a different value. No mutation happened.
        false
    }

    /// Return `true` when the mutation was successful.
    /// 
    /// Return `false` in case of failure, such as empty genome, bad parameters for instruction.
    pub fn replace_instruction_with_histogram<R: Rng + ?Sized>(&mut self, rng: &mut R, context: &GenomeMutateContext) -> bool {
        // Bail out if the trigram.csv file hasn't been loaded.
        if !context.has_suggest_instruction() {
            return false;
        }
        let length: usize = self.genome_vec.len();
        if length < 1 {
            return false;
        }
        let index1: usize = rng.gen_range(0..length);
        let index0: i32 = (index1 as i32) - 1;
        let index2: usize = index1 + 1;
        let mut prev_instruction: Option<InstructionId> = None;
        if index0 >= 0 {
            match self.genome_vec.get(index0 as usize) {
                Some(ref value) => {
                    let instruction_id: InstructionId = value.instruction_id();
                    prev_instruction = Some(instruction_id);
                },
                None => {}
            };
        }
        let next_instruction: Option<InstructionId> = match self.genome_vec.get(index2) {
            Some(ref value) => {
                let instruction_id: InstructionId = value.instruction_id();
                Some(instruction_id)
            },
            None => None
        };
        let genome_item: &mut GenomeItem = &mut self.genome_vec[index1];
        if genome_item.is_mutation_locked() {
            return false;
        }
        let original_instruction: InstructionId = genome_item.instruction_id();

        // Try a few times
        for _ in 0..Self::MUTATE_RETRIES {
            let suggested_instruction_id: InstructionId = match context.suggest_instruction(rng, prev_instruction, next_instruction) {
                Some(value) => value,
                None => {
                    return false;
                }
            };
            if original_instruction == suggested_instruction_id {
                // Picked the same as the original, try pick a different value
                continue;
            }
            if !genome_item.set_instruction(suggested_instruction_id) {
                // Picked a terrible instruction, such as `lpb` or `seq`, that requires
                // special attention to the `source_value`. Try pick another instruction.
                continue;
            }
            // Successfully picked a good instruction
            return true;
        }
        // Too many tries, without picking a different value. No mutation happened.
        false
    }

    /// Return `true` when the mutation was successful.
    /// 
    /// Return `false` in case of failure, such as empty genome, bad parameters for instruction.
    pub fn insert_instruction_with_constant<R: Rng + ?Sized>(&mut self, rng: &mut R, context: &GenomeMutateContext) -> bool {
        // Bail out if the histogram csv file hasn't been loaded.
        if !context.has_histogram_instruction_constant() {
            return false;
        }
        // Bail out if the trigram.csv file hasn't been loaded.
        if !context.has_suggest_instruction() {
            return false;
        }
        // Bail out if the trigram.csv file hasn't been loaded.
        if !context.has_suggest_target() {
            return false;
        }
        let length: usize = self.genome_vec.len();
        if length < 1 {
            return false;
        }

        // Decide on where to insert a new GenomeItem
        let index1: usize = rng.gen_range(0..length);
        let index0: i32 = (index1 as i32) - 1;
        let index2: usize = index1;

        // Gather info about the "previous" GenomeItem
        let mut prev_instruction: Option<InstructionId> = None;
        let mut prev_target: TargetValue = TargetValue::ProgramStart;
        if index0 >= 0 {
            match self.genome_vec.get(index0 as usize) {
                Some(ref value) => {
                    let instruction_id: InstructionId = value.instruction_id();
                    prev_instruction = Some(instruction_id);
                    prev_target = Self::get_target_value(value);
                },
                None => {}
            };
        }

        // Gather info about the "next" GenomeItem
        let mut next_instruction: Option<InstructionId> = None;
        let mut next_target: TargetValue = TargetValue::ProgramStop;
        match self.genome_vec.get(index2) {
            Some(ref value) => {
                let instruction_id: InstructionId = value.instruction_id();
                next_instruction = Some(instruction_id);
                next_target = Self::get_target_value(value)
            },
            None => {}
        }

        // Pick an instruction from the histogram
        let suggested_instruction_id: InstructionId = match context.suggest_instruction(rng, prev_instruction, next_instruction) {
            Some(value) => value,
            None => {
                return false;
            }
        };

        // Pick a source constant from the histogram
        let source_value: i32 = match context.choose_constant_with_histogram(rng, suggested_instruction_id) {
            Some(value) => value,
            None => 0
        };

        // Pick a target register from the histogram
        let suggested_target_value: Option<TargetValue> = context.suggest_target(rng, prev_target, next_target);
        let target_value: i32;
        match suggested_target_value {
            Some(TargetValue::Direct(value)) => {
                target_value = value;
            },
            _ => {
                target_value = rng.gen_range(0..5);
            }
        };

        let genome_item = GenomeItem::new(
            suggested_instruction_id, 
            RegisterType::Direct,
            target_value, 
            ParameterType::Constant, 
            source_value
        );
        // No need to sanitize when using histogram
        // println!("insert at {:?} item: {:?}", index1, genome_item);
        self.genome_vec.insert(index1, genome_item);
        true
    }

    /// Flip `source` to `Constant`, and assign a value from histogram.
    /// 
    /// Ignore rows where `source` already is `Constant`.
    /// 
    /// Return `true` when the mutation was successful.
    /// 
    /// Return `false` in case of failure, such as empty genome, bad parameters for instruction.
    pub fn mutate_set_source_to_constant<R: Rng + ?Sized>(&mut self, rng: &mut R, context: &GenomeMutateContext) -> bool {
        let mut indexes: Vec<usize> = vec!();
        for (index, genome_item) in self.genome_vec.iter().enumerate() {
            if genome_item.is_mutation_locked() {
                continue;
            }
            if genome_item.source_type() == ParameterType::Constant {
                continue;
            }
            match genome_item.instruction_id() {
                InstructionId::EvalSequence | 
                InstructionId::LoopBegin | 
                InstructionId::LoopEnd |
                InstructionId::UnofficialFunction { .. } |
                InstructionId::UnofficialLoopBeginSubtract => {
                    continue;
                },
                _ => {}
            }
            indexes.push(index);
        }
        if indexes.is_empty() {
            return false;
        }

        // Mutate one of the instructions
        let index: &usize = indexes.choose(rng).unwrap();
        let genome_item: &mut GenomeItem = &mut self.genome_vec[*index];
        let instruction_id: InstructionId = genome_item.instruction_id();

        let picked_value: i32 = match context.choose_constant_with_histogram(rng, instruction_id) {
            Some(value) => value,
            None => {
                // No entry for this instruction
                return false;
            }
        };
        genome_item.set_source_type(ParameterType::Constant);
        genome_item.set_source_value(picked_value);
        true
    }

    /// Flip `source` to `Direct`, and assign a value from the alive registers.
    /// 
    /// Ignore rows where `source` already is `Direct`.
    /// 
    /// Return `true` when the mutation was successful.
    /// 
    /// Return `false` in case of failure, such as empty genome, bad parameters for instruction.
    pub fn mutate_set_source_to_direct<R: Rng + ?Sized>(&mut self, rng: &mut R) -> bool {
        let mut indexes: Vec<usize> = vec!();
        for (index, genome_item) in self.genome_vec.iter().enumerate() {
            if genome_item.is_mutation_locked() {
                continue;
            }
            if genome_item.source_type() == ParameterType::Direct {
                continue;
            }
            match genome_item.instruction_id() {
                InstructionId::EvalSequence | 
                InstructionId::LoopBegin | 
                InstructionId::LoopEnd |
                InstructionId::UnofficialFunction { .. } |
                InstructionId::UnofficialLoopBeginSubtract => {
                    continue;
                },
                _ => {}
            }
            indexes.push(index);
        }
        if indexes.is_empty() {
            return false;
        }

        // Mutate one of the instructions
        let the_index: usize = *(indexes.choose(rng).unwrap());

        // Determine the registers that may contain meaningful content.
        // Only considering the rows above.
        let mut alive_registers: Vec<i32> = vec!();
        for (index, genome_item) in self.genome_vec.iter().enumerate() {
            if index >= the_index {
                break;
            }
            let register: i32 = genome_item.target_value();
            alive_registers.push(register);
        }
        if alive_registers.is_empty() {
            return false;
        }

        // Pick a random register that is alive
        let the_register: i32 = *(alive_registers.choose(rng).unwrap());

        let genome_item: &mut GenomeItem = &mut self.genome_vec[the_index];
        genome_item.set_source_type(ParameterType::Direct);
        genome_item.set_source_value(the_register);
        true
    }

    /// Turn off an entire block of `lpb`...`lpe` and instructions inbetween.
    /// 
    /// Return `true` when the mutation was successful.
    /// 
    /// Return `false` if nothing was change.
    pub fn mutate_disable_loop<R: Rng + ?Sized>(&mut self, rng: &mut R) -> bool {
        let mut indexes: Vec<usize> = vec!();
        for (index, genome_item) in self.genome_vec.iter().enumerate() {
            if genome_item.is_mutation_locked() {
                continue;
            }
            if genome_item.instruction_id() != InstructionId::LoopBegin {
                continue;
            }
            indexes.push(index);
        }
        if indexes.is_empty() {
            return false;
        }
        let index: &usize = indexes.choose(rng).unwrap();
        let genome_item: &mut GenomeItem = &mut self.genome_vec[*index];
        genome_item.set_source_value(0);
        genome_item.set_source_type(ParameterType::Constant);
        true
    }

    /// Swaps `target_value` with `source_value`.
    /// 
    /// Only impact rows where target_type=Direct and source_type=Direct.
    /// 
    /// Return `true` when the mutation was successful.
    /// 
    /// Return `false` in case of failure, such as empty genome, bad parameters for instruction.
    pub fn mutate_swap_registers<R: Rng + ?Sized>(&mut self, rng: &mut R) -> bool {
        let mut indexes: Vec<usize> = vec!();
        for (index, genome_item) in self.genome_vec.iter().enumerate() {
            if genome_item.is_mutation_locked() {
                continue;
            }
            if genome_item.target_type() != RegisterType::Direct {
                continue;
            }
            if genome_item.source_type() != ParameterType::Direct {
                continue;
            }
            indexes.push(index);
        }
        if indexes.is_empty() {
            return false;
        }

        // Try a few times
        for _ in 0..Self::MUTATE_RETRIES {
            let index: &usize = indexes.choose(rng).unwrap();
            let genome_item: &mut GenomeItem = &mut self.genome_vec[*index];
            if !genome_item.mutate_swap_source_target_value() {
                // Try swap a different row
                continue;
            }
            // Successfully mutated
            return true;
        }

        // To many attempts. No mutation happened.
        false
    }

    /// Return `true` when the mutation was successful.
    /// 
    /// Return `false` in case of failure, such as empty genome, bad parameters for instruction.
    pub fn mutate_enabled<R: Rng + ?Sized>(&mut self, rng: &mut R) -> bool {
        let mut indexes: Vec<usize> = vec!();
        for (index, genome_item) in self.genome_vec.iter().enumerate() {
            if genome_item.is_mutation_locked() {
                continue;
            }
            match genome_item.instruction_id() {
                InstructionId::LoopBegin | 
                InstructionId::LoopEnd |
                InstructionId::UnofficialLoopBeginSubtract => {
                    continue;
                },
                _ => {}
            }
            if genome_item.target_type() == RegisterType::Indirect {
                continue;
            }
            if genome_item.source_type() == ParameterType::Indirect {
                continue;
            }
            indexes.push(index);
        }
        if indexes.is_empty() {
            return false;
        }

        let index: &usize = indexes.choose(rng).unwrap();
        let genome_item: &mut GenomeItem = &mut self.genome_vec[*index];
        let flipped = !genome_item.is_enabled();
        genome_item.set_enabled(flipped);

        // Successfully mutated
        true
    }

    /// Return `true` when the mutation was successful.
    /// 
    /// Return `false` in case of failure, such as empty genome, bad parameters for instruction.
    pub fn mutate_swap_rows<R: Rng + ?Sized>(&mut self, rng: &mut R) -> bool {
        let mut indexes: Vec<usize> = vec!();
        for (index, genome_item) in self.genome_vec.iter().enumerate() {
            if genome_item.is_mutation_locked() {
                continue;
            }
            // Prevent messing with loop begin/end instructions.
            match genome_item.instruction_id() {
                InstructionId::LoopBegin | 
                InstructionId::LoopEnd |
                InstructionId::UnofficialLoopBeginSubtract => {
                    continue;
                },
                _ => {}
            }
            indexes.push(index);
        }
        if indexes.len() < 2 {
            return false;
        }

        let chosen_indexes: Vec<usize> = indexes.choose_multiple(rng, 2).cloned().collect();
        if chosen_indexes.len() < 2 {
            return false;
        }
        let index0: usize = chosen_indexes[0];
        let index1: usize = chosen_indexes[1];
        if index0 == index1 {
            return false;
        }
        self.genome_vec.swap(index0, index1);
        true
    }

    /// Return `true` when the mutation was successful.
    /// 
    /// Return `false` in case of failure, such as empty genome, bad parameters for instruction.
    pub fn mutate_swap_adjacent_rows<R: Rng + ?Sized>(&mut self, rng: &mut R) -> bool {
        let mut indexes: Vec<usize> = vec!();
        for (index, genome_item) in self.genome_vec.iter().enumerate() {
            if genome_item.is_mutation_locked() {
                continue;
            }
            indexes.push(index);
        }
        let length: usize = indexes.len();
        if length < 2 {
            return false;
        }
        let position: usize = rng.gen_range(0..length-1);
        let index0: usize = indexes[position];
        let index1: usize = indexes[position + 1];
        let instruction0: InstructionId = self.genome_vec[index0].instruction_id();
        let instruction1: InstructionId = self.genome_vec[index1].instruction_id();
        // Prevent reversing the order of the loop begin/end instructions.
        let is_loop = 
            matches!(instruction0, InstructionId::LoopBegin | InstructionId::UnofficialLoopBeginSubtract) && 
            instruction1 == InstructionId::LoopEnd;
        if is_loop {
            return false;
        }
        self.genome_vec.swap(index0, index1);
        true
    }

    /// Return `true` when the mutation was successful.
    /// 
    /// Return `false` in case of failure, such as empty genome, bad parameters for instruction.
    pub fn mutate_insert_loop<R: Rng + ?Sized>(&mut self, rng: &mut R) -> bool {
        let length: usize = self.genome_vec.len();
        if length < 2 {
            return false;
        }
        let index0: usize = rng.gen_range(0..length);
        let index1: usize = rng.gen_range(0..length);
        if index0 == index1 {
            return false;
        }

        // first insert loop-end
        {
            let index: usize = index0.max(index1);
            let item = GenomeItem::new(
                InstructionId::LoopEnd, 
                RegisterType::Direct,
                0, 
                ParameterType::Constant, 
                0
            );
            self.genome_vec.insert(index, item);
        }

        // last insert loop-begin
        {
            let index: usize = index0.min(index1);
            let item = GenomeItem::new(
                InstructionId::LoopBegin,
                RegisterType::Direct,
                rng.gen_range(0..5) as i32,
                ParameterType::Constant,
                1
            );
            self.genome_vec.insert(index, item);
        }

        true
    }

    /// Mutate the `seq` instruction, so the dependency is inlined.
    /// 
    /// Return `true` when the mutation was successful.
    /// 
    /// Return `false` in case of failure, such as empty genome, bad parameters for instruction.
    pub fn mutate_inline_seq<R: Rng + ?Sized>(rng: &mut R, dm: &DependencyManager, genome_vec: &mut Vec<GenomeItem>) -> bool {
        let mut indexes: Vec<usize> = vec!();
        for (index, genome_item) in genome_vec.iter().enumerate() {
            if genome_item.source_type() != ParameterType::Constant {
                continue;
            }
            if genome_item.instruction_id() != InstructionId::EvalSequence {
                continue;
            }
            indexes.push(index);
        }
        if indexes.is_empty() {
            return false;
        }

        // Determine how many registers are already used
        let mut max_register: u32 = 0;
        for genome_item in genome_vec.iter() {
            if !genome_item.is_enabled() {
                continue;
            }
            if genome_item.target_type() == RegisterType::Direct {
                let index = genome_item.target_value();
                if index >= 0 {
                    max_register = max_register.max(index as u32);
                }
            }
            if genome_item.source_type() == ParameterType::Direct {
                let index = genome_item.source_value();
                if index >= 0 {
                    max_register = max_register.max(index as u32);
                }
            }
        }
        let offset_by: u32 = max_register;

        // Mutate one of the `seq` instructions
        // let before_snapshot: String = Self::genome_vec_to_formatted_program(&genome_vec);
        let index: &usize = indexes.choose(rng).unwrap();
        let genome_item: &mut GenomeItem = &mut genome_vec[*index];

        if genome_item.instruction_id() != InstructionId::EvalSequence {
            error!("Expected 'seq' instruction");
            return false;
        }
        if genome_item.source_type() != ParameterType::Constant {
            error!("Expected 'seq' instruction's source type to be of type Constant");
            return false;
        }
        let source_value: i32 = genome_item.source_value();
        if source_value < 0 {
            return false;
        }
        let program_id: u64 = source_value as u64;

        let target_value = genome_item.target_value();
        if target_value < 0 {
            return false;
        }
        // The "target" register of "seq $target,oeis_id" is used for transfering input/output
        // between the parent program and the inlined program.
        let in_out_register: u32 = target_value as u32;

        let parsed_program: ParsedProgram = match Self::load_program_with_id(&dm, program_id) {
            Ok(value) => value,
            Err(error) => {
                error!("mutate_inline_seq. Cannot load program: {} error: {:?}", program_id, error);
                return false;
            }
        };

        let mut inline_genome_vec: Vec<GenomeItem> = parsed_program.to_genome_item_vec();

        // Offset registers by `offset_by`
        let mut clear_register_indexes = HashSet::<i32>::new();
        for genome_item in &mut inline_genome_vec {
            if genome_item.target_type() == RegisterType::Direct {
                let index = genome_item.target_value();
                if index == 0 {
                    genome_item.set_target_value(in_out_register as i32);
                }
                if index > 0 {
                    let index_with_offset: i32 = index + (offset_by as i32);
                    genome_item.set_target_value(index_with_offset);
                    clear_register_indexes.insert(index_with_offset);
                }
            }
            if genome_item.source_type() == ParameterType::Direct {
                let index = genome_item.source_value();
                if index == 0 {
                    genome_item.set_source_value(in_out_register as i32);
                }
                if index > 0 {
                    let index_with_offset: i32 = index + (offset_by as i32);
                    genome_item.set_source_value(index_with_offset);
                    clear_register_indexes.insert(index_with_offset);
                }
            }
        }

        // prepend instructions that clears the registers used by this sequence
        // If the `seq` is inside a loop, then we don't want the previous state to 
        // interfere with the next iteration.
        for register_index in clear_register_indexes {
            let genome_item = GenomeItem::new(
                InstructionId::Move,
                RegisterType::Direct,
                register_index,
                ParameterType::Constant,
                0
            );
            inline_genome_vec.insert(0, genome_item);
        }

        // Replace `seq` with the inline_genome_vec
        genome_vec.splice(index..=index, inline_genome_vec.iter().cloned());

        // let after_snapshot: String = Self::genome_vec_to_formatted_program(&genome_vec);
        // println!("; BEFORE\n{}\n; AFTER\n{}", before_snapshot, after_snapshot);

        true
    }

    /// Mutate the `seq` instruction, so it invokes a random program.
    /// 
    /// Only impact rows where source_type=Constant and instruct=seq
    /// 
    /// Return `true` when the mutation was successful.
    /// 
    /// Return `false` in case the mutation had no effect.
    pub fn mutate_instruction_seq<R: Rng + ?Sized>(&mut self, rng: &mut R, context: &GenomeMutateContext, category: MutateEvalSequenceCategory) -> bool {
        let mut indexes: Vec<usize> = vec!();
        for (index, genome_item) in self.genome_vec.iter().enumerate() {
            if genome_item.is_mutation_locked() {
                continue;
            }
            if genome_item.source_type() != ParameterType::Constant {
                continue;
            }
            if genome_item.instruction_id() != InstructionId::EvalSequence {
                continue;
            }
            indexes.push(index);
        }
        if indexes.is_empty() {
            return false;
        }

        // Mutate one of the `seq` instructions
        let index: &usize = indexes.choose(rng).unwrap();
        let genome_item: &mut GenomeItem = &mut self.genome_vec[*index];

        // Try a few times
        for _ in 0..Self::MUTATE_RETRIES {
            if !genome_item.mutate_instruction_seq(rng, context, category) {
                // Picked the same as the original, try pick a different value
                continue;
            }
            // Successfully mutated
            return true;
        }

        // To many attempts. No mutation happened.
        false
    }

    /// Apply a mutation to the genome.
    /// 
    /// Return `true` when the genome got altered.
    /// 
    /// Return `false` in case the mutation didn't change the genome.
    pub fn mutate<R: Rng + ?Sized>(&mut self, rng: &mut R, context: &GenomeMutateContext) -> bool {
        let mutation_vec: Vec<(MutateGenome,usize)> = vec![
            (MutateGenome::ReplaceInstructionWithHistogram, 10),
            (MutateGenome::InsertInstructionWithConstant, 0),
            (MutateGenome::IncrementSourceValueWhereTypeIsConstant, 10),
            (MutateGenome::DecrementSourceValueWhereTypeIsConstant, 10),
            (MutateGenome::ReplaceSourceConstantWithHistogram, 10),
            (MutateGenome::SetSourceToConstant, 10),
            (MutateGenome::SetSourceToDirect, 10),
            (MutateGenome::DisableLoop, 0),
            (MutateGenome::SwapRegisters, 10),
            (MutateGenome::IncrementSourceValueWhereTypeIsDirect, 10),
            (MutateGenome::DecrementSourceValueWhereTypeIsDirect, 10),
            (MutateGenome::ReplaceSourceWithHistogram, 10),
            (MutateGenome::IncrementTargetValueWhereTypeIsDirect, 10),
            (MutateGenome::DecrementTargetValueWhereTypeIsDirect, 10),
            (MutateGenome::ReplaceTargetWithHistogram, 10),
            (MutateGenome::ReplaceLineWithHistogram, 50),
            (MutateGenome::InsertLineWithHistogram, 50),
            (MutateGenome::CopyLine, 10),
            (MutateGenome::ToggleEnabled, 10),
            (MutateGenome::SwapRows, 10),
            (MutateGenome::SwapAdjacentRows, 10),
            (MutateGenome::InsertLoopBeginEnd, 0),
            (MutateGenome::CallProgramWeightedByPopularity, 0),
            (MutateGenome::CallMostPopularProgram, 10),
            (MutateGenome::CallMediumPopularProgram, 20),
            (MutateGenome::CallLeastPopularProgram, 50),
            (MutateGenome::CallRecentProgram, 300),
            (MutateGenome::CallProgramThatUsesIndirectMemoryAccess, 0),
        ];
        let mutation: &MutateGenome = &mutation_vec.choose_weighted(rng, |item| item.1).unwrap().0;

        let did_mutate_ok: bool = match mutation {
            MutateGenome::ReplaceInstructionWithHistogram => {
                self.replace_instruction_with_histogram(rng, context)
            },
            MutateGenome::InsertInstructionWithConstant => {
                self.insert_instruction_with_constant(rng, context)
            },
            MutateGenome::IncrementSourceValueWhereTypeIsConstant => {
                self.increment_source_value_where_type_is_constant(rng)
            },
            MutateGenome::DecrementSourceValueWhereTypeIsConstant => {
                self.decrement_source_value_where_type_is_constant(rng)
            },
            MutateGenome::ReplaceSourceConstantWithHistogram => {
                self.replace_source_constant_with_histogram(rng, context)
            },
            MutateGenome::SetSourceToConstant => {
                self.mutate_set_source_to_constant(rng, context)
            },
            MutateGenome::SetSourceToDirect => {
                self.mutate_set_source_to_direct(rng)
            },
            MutateGenome::DisableLoop => {
                self.mutate_disable_loop(rng)
            },
            MutateGenome::SwapRegisters => {
                self.mutate_swap_registers(rng)
            },
            MutateGenome::IncrementSourceValueWhereTypeIsDirect => {
                self.increment_source_value_where_type_is_direct(rng)
            },
            MutateGenome::DecrementSourceValueWhereTypeIsDirect => {
                self.decrement_source_value_where_type_is_direct(rng)
            },
            MutateGenome::ReplaceSourceWithHistogram => {
                self.replace_source_with_histogram(rng, context)
            },
            MutateGenome::IncrementTargetValueWhereTypeIsDirect => {
                self.increment_target_value_where_type_is_direct(rng)
            },
            MutateGenome::DecrementTargetValueWhereTypeIsDirect => {
                self.decrement_target_value_where_type_is_direct(rng)
            },
            MutateGenome::ReplaceTargetWithHistogram => {
                self.replace_target_with_histogram(rng, context)
            },
            MutateGenome::ReplaceLineWithHistogram => {
                self.replace_line_with_histogram(rng, context)
            },
            MutateGenome::InsertLineWithHistogram => {
                self.insert_line_with_histogram(rng, context)
            },
            MutateGenome::CopyLine => {
                self.copy_line(rng)
            },
            MutateGenome::ToggleEnabled => {
                self.mutate_enabled(rng)
            },
            MutateGenome::SwapRows => {
                self.mutate_swap_rows(rng)
            },
            MutateGenome::SwapAdjacentRows => {
                self.mutate_swap_adjacent_rows(rng)
            },
            MutateGenome::InsertLoopBeginEnd => {
                self.mutate_insert_loop(rng)
            },            
            MutateGenome::CallProgramWeightedByPopularity => {
                self.mutate_instruction_seq(rng, context, MutateEvalSequenceCategory::WeightedByPopularity)
            },
            MutateGenome::CallMostPopularProgram => {
                self.mutate_instruction_seq(rng, context, MutateEvalSequenceCategory::MostPopular)
            },
            MutateGenome::CallMediumPopularProgram => {
                self.mutate_instruction_seq(rng, context, MutateEvalSequenceCategory::MediumPopular)
            },
            MutateGenome::CallLeastPopularProgram => {
                self.mutate_instruction_seq(rng, context, MutateEvalSequenceCategory::LeastPopular)
            },
            MutateGenome::CallRecentProgram => {
                self.mutate_instruction_seq(rng, context, MutateEvalSequenceCategory::Recent)
            },
            MutateGenome::CallProgramThatUsesIndirectMemoryAccess => {
                self.mutate_instruction_seq(rng, context, MutateEvalSequenceCategory::ProgramThatUsesIndirectMemoryAccess)
            }
        };

        if did_mutate_ok {
            self.message_vec.push(format!("mutate: {:?}", mutation));
        } else {
            self.message_vec.push(format!("mutate: {:?}, no change", mutation));
        }

        did_mutate_ok
    }

    fn genome_vec_to_formatted_program(genome_vec: &Vec<GenomeItem>) -> String {
        let rows: Vec<String> = genome_vec.iter().map(|genome_item| {
            genome_item.to_line_string()
        }).collect();
        rows.join("\n")
    }
}

impl fmt::Display for Genome {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let formatted_program: String = Self::genome_vec_to_formatted_program(&self.genome_vec);
        write!(f, "{}", formatted_program)
    }
}
