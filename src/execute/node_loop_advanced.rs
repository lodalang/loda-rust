use super::{Node, Program, ProgramState, ProgramRunnerManager, RegisterIndex, RunMode};

pub struct NodeLoopConstant {
    register: RegisterIndex,
    range_length: u8,
    program: Program,
}

impl NodeLoopConstant {
    pub fn new(register: RegisterIndex, range_length: u8, program: Program) -> Self {
        NodeLoopConstant {
            register: register,
            range_length: range_length,
            program: program,
        }
    }
}

impl Node for NodeLoopConstant {
    fn shorthand(&self) -> &str {
        "loop constant"
    }

    fn formatted_instruction(&self) -> String {
        String::from("")
    }

    fn eval(&self, state: &mut ProgramState) {
        if state.run_mode() == RunMode::Verbose {
            let snapshot = state.register_vec_to_string();
            let instruction = format!("lpb {},{}", self.register, self.range_length);
            println!("{:12} {} => {}", instruction, snapshot, snapshot);
        }

        let mut cycles = 0;
        loop {
            let old_state: ProgramState = state.clone();

            self.program.run(state);

            let is_less: bool = state.is_less(
                &old_state, 
                self.register.clone(), 
                self.range_length
            );

            if !is_less {

                if state.run_mode() == RunMode::Verbose {
                    println!("LOOP CYCLE EXIT");
                }

                // When the loop reaches its end, the previous state is restored.
                *state = old_state.clone();
                break;
            }


            cycles += 1;
            if cycles > 1000 {
                panic!("looped too many times");
                // TODO: propagate info about problematic loops all the way
                // to caller and their caller, and let them decide what to do about it.
            }
            if state.run_mode() == RunMode::Verbose {
                println!("lpe");
            }
        }
    }

    fn accumulate_register_indexes(&self, register_vec: &mut Vec<RegisterIndex>) {
        // Loop doesn't modify any registers
        self.program.accumulate_register_indexes(register_vec);
    }

    fn update_call(&mut self, program_manager: &mut ProgramRunnerManager) {
        self.program.update_call(program_manager);
    }

    fn accumulate_call_dependencies(&self, program_id_vec: &mut Vec<u64>) {
        self.program.accumulate_call_dependencies(program_id_vec);
    }
}
