use crate::ComputeResult::Halt;

fn main() {
    let input = "1,0,0,3,1,1,2,3,1,3,4,3,1,5,0,3,2,6,1,19,1,19,5,23,2,9,23,27,1,5,27,31,1,5,31,35,1,35,13,39,1,39,9,43,1,5,43,47,1,47,6,51,1,51,13,55,1,55,9,59,1,59,13,63,2,63,13,67,1,67,10,71,1,71,6,75,2,10,75,79,2,10,79,83,1,5,83,87,2,6,87,91,1,91,6,95,1,95,13,99,2,99,13,103,1,103,9,107,1,10,107,111,2,111,13,115,1,10,115,119,1,10,119,123,2,13,123,127,2,6,127,131,1,13,131,135,1,135,2,139,1,139,6,0,99,2,0,14,0";

    println!("Hello, world, {:?}", str_to_intcode(input));
    //    println!("{:?}", parameter_modes(1009));
}

fn str_to_intcode(string: &str) -> Vec<i32> {
    string
        .split_terminator(",")
        .map(|s| s.parse().unwrap())
        .collect()
}

struct State {
    instruction_pointer: u32,
    intcode: Vec<i32>,
    input: Vec<i32>,
    output: Vec<i32>,
}

enum ComputeResult {
    Halt,
}

//todo turn into an enumeration instead of using u8 for the parameter modes?
fn parameter_modes(opcode: i32) -> (u8, u8, u8, u8) {
    let a = opcode / 10000;
    let b = (opcode - a * 10000) / 1000;
    let c = (opcode - a * 10000 - b * 1000) / 100;
    let d = opcode - a * 10000 - b * 1000 - c * 100;

    (a as u8, b as u8, c as u8, d as u8)
}

fn state_from_string(string: &str) -> State {
    State {
        instruction_pointer: 0,
        intcode: str_to_intcode(string),
        input: vec![],
        output: vec![],
    }
}

//todo how to deal with the situation when the state is invalid or the parameter mode isn't supported?
fn get_value(parameter_mode: u8, pointer: u32, state: &State) -> i32 {
    //position mode
    if parameter_mode == 0 {
        let at_index = state.intcode[pointer as usize];
        println!(
            "pointer {}, got value {} at index {}",
            pointer, state.intcode[at_index as usize], at_index
        );
        state.intcode[at_index as usize]
    }
    //immediate mode
    else {
        println!(
            "pointer {}, got value {} immediately",
            pointer, state.intcode[pointer as usize]
        );
        state.intcode[pointer as usize]
    }
}

fn computer(state: &mut State) -> Result<ComputeResult, String> {
    let offset = state.instruction_pointer;

    //todo is this defensive programming a good idea?
    assert!(
        offset < state.intcode.len() as u32,
        "offset {} out of bounds, intcode length {}",
        offset,
        state.intcode.len()
    );
    assert!(state.intcode.len() > 0, "no intcode to process");

    let (_, b, c, opcode) = parameter_modes(state.intcode[offset as usize]);

    //add
    if opcode == 1 {
        let pos_to = state.intcode[(offset + 3) as usize];
        let value_c = get_value(c, offset + 1, state);
        let value_b = get_value(b, offset + 2, state);

        state.intcode[pos_to as usize] = value_c + value_b;
        state.instruction_pointer += 4;

        computer(state)
    }
    //multiply
    else if opcode == 2 {
        let pos_to = state.intcode[(offset + 3) as usize];
        let value_c = get_value(c, offset + 1, state);
        let value_b = get_value(b, offset + 2, state);

        state.intcode[pos_to as usize] = value_c * value_b;
        state.instruction_pointer += 4;

        computer(state)
    }
    //input
    else if opcode == 3 {
        let pos_to = state.intcode[(offset + 1) as usize];

        //attempt to read from the input
        match state.input.pop() {
            Some(v) => {
                state.intcode[pos_to as usize] = v;
                state.instruction_pointer += 2;

                computer(state)
            }
            None => {
                let error = format!("Input expected, but none provided.");
                Err(error)
            }
        }
    }
    //output
    else if opcode == 4 {
        let value_to_output = get_value(c, offset + 1, state);

        state.output.push(value_to_output);
        state.instruction_pointer += 2;

        computer(state)
    } else if opcode == 99 {
        Ok(Halt)
    } else {
        let error = format!("{} {}", "Unknown opcode", opcode);
        Err(error)
    }
}

#[cfg(test)]
mod tests {
    use crate::{computer, state_from_string, str_to_intcode};

    #[test]
    fn can_parse_intcode() {
        assert_eq!(vec![1, 0, 0, 0, 99], str_to_intcode("1,0,0,0,99"));
    }

    #[test]
    fn small_programs() {
        // let add = "1,0,0,0,99";
        // let multiply = "2,3,0,3,99";
        // let multi_put_at_the_end = "2,4,4,5,99,0";
        let thirty = "1,1,1,4,99,5,6,0,99";
        let mut state = state_from_string(thirty);

        assert_eq!(computer(&mut state).is_ok(), true);
        assert_eq!(state.intcode.first().unwrap(), &30i32)
    }

    #[test]
    fn day2_part_one() {
        let mut state = state_from_string("1,12,2,3,1,1,2,3,1,3,4,3,1,5,0,3,2,6,1,19,1,19,5,23,2,9,23,27,1,5,27,31,1,5,31,35,1,35,13,39,1,39,9,43,1,5,43,47,1,47,6,51,1,51,13,55,1,55,9,59,1,59,13,63,2,63,13,67,1,67,10,71,1,71,6,75,2,10,75,79,2,10,79,83,1,5,83,87,2,6,87,91,1,91,6,95,1,95,13,99,2,99,13,103,1,103,9,107,1,10,107,111,2,111,13,115,1,10,115,119,1,10,119,123,2,13,123,127,2,6,127,131,1,13,131,135,1,135,2,139,1,139,6,0,99,2,0,14,0");

        assert_eq!(computer(&mut state).is_ok(), true);
        assert_eq!(state.intcode.first().unwrap(), &4090689i32)
    }

    #[test]
    fn input_output() {
        let mut state = state_from_string("3,0,4,0,99");
        state.input.push(55);

        assert_eq!(computer(&mut state).is_ok(), true);
        assert_eq!(state.output.first().unwrap(), &55i32)
    }

    #[test]
    fn parameter_modes() {
        let mut state = state_from_string("1002,4,3,4,33");

        assert_eq!(computer(&mut state).is_ok(), true);
    }

    #[test]
    fn day5_part_one() {
        let mut state = state_from_string("3,225,1,225,6,6,1100,1,238,225,104,0,1,192,154,224,101,-161,224,224,4,224,102,8,223,223,101,5,224,224,1,223,224,223,1001,157,48,224,1001,224,-61,224,4,224,102,8,223,223,101,2,224,224,1,223,224,223,1102,15,28,225,1002,162,75,224,1001,224,-600,224,4,224,1002,223,8,223,1001,224,1,224,1,224,223,223,102,32,57,224,1001,224,-480,224,4,224,102,8,223,223,101,1,224,224,1,224,223,223,1101,6,23,225,1102,15,70,224,1001,224,-1050,224,4,224,1002,223,8,223,101,5,224,224,1,224,223,223,101,53,196,224,1001,224,-63,224,4,224,102,8,223,223,1001,224,3,224,1,224,223,223,1101,64,94,225,1102,13,23,225,1101,41,8,225,2,105,187,224,1001,224,-60,224,4,224,1002,223,8,223,101,6,224,224,1,224,223,223,1101,10,23,225,1101,16,67,225,1101,58,10,225,1101,25,34,224,1001,224,-59,224,4,224,1002,223,8,223,1001,224,3,224,1,223,224,223,4,223,99,0,0,0,677,0,0,0,0,0,0,0,0,0,0,0,1105,0,99999,1105,227,247,1105,1,99999,1005,227,99999,1005,0,256,1105,1,99999,1106,227,99999,1106,0,265,1105,1,99999,1006,0,99999,1006,227,274,1105,1,99999,1105,1,280,1105,1,99999,1,225,225,225,1101,294,0,0,105,1,0,1105,1,99999,1106,0,300,1105,1,99999,1,225,225,225,1101,314,0,0,106,0,0,1105,1,99999,1108,226,226,224,102,2,223,223,1005,224,329,101,1,223,223,107,226,226,224,1002,223,2,223,1005,224,344,1001,223,1,223,107,677,226,224,102,2,223,223,1005,224,359,101,1,223,223,7,677,226,224,102,2,223,223,1005,224,374,101,1,223,223,108,226,226,224,102,2,223,223,1006,224,389,101,1,223,223,1007,677,677,224,102,2,223,223,1005,224,404,101,1,223,223,7,226,677,224,102,2,223,223,1006,224,419,101,1,223,223,1107,226,677,224,1002,223,2,223,1005,224,434,1001,223,1,223,1108,226,677,224,102,2,223,223,1005,224,449,101,1,223,223,108,226,677,224,102,2,223,223,1005,224,464,1001,223,1,223,8,226,677,224,1002,223,2,223,1005,224,479,1001,223,1,223,1007,226,226,224,102,2,223,223,1006,224,494,101,1,223,223,1008,226,677,224,102,2,223,223,1006,224,509,101,1,223,223,1107,677,226,224,1002,223,2,223,1006,224,524,1001,223,1,223,108,677,677,224,1002,223,2,223,1005,224,539,1001,223,1,223,1107,226,226,224,1002,223,2,223,1006,224,554,1001,223,1,223,7,226,226,224,1002,223,2,223,1006,224,569,1001,223,1,223,8,677,226,224,102,2,223,223,1006,224,584,101,1,223,223,1008,677,677,224,102,2,223,223,1005,224,599,101,1,223,223,1007,226,677,224,1002,223,2,223,1006,224,614,1001,223,1,223,8,677,677,224,1002,223,2,223,1005,224,629,101,1,223,223,107,677,677,224,102,2,223,223,1005,224,644,101,1,223,223,1108,677,226,224,102,2,223,223,1005,224,659,101,1,223,223,1008,226,226,224,102,2,223,223,1006,224,674,1001,223,1,223,4,223,99,226");
        state.input.push(1);

        assert_eq!(computer(&mut state).is_ok(), true);
        assert_eq!(state.output, vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 11049715])
    }
}
