use crate::ComputeResult::Halt;

fn main() {
    let input = "1,0,0,3,1,1,2,3,1,3,4,3,1,5,0,3,2,6,1,19,1,19,5,23,2,9,23,27,1,5,27,31,1,5,31,35,1,35,13,39,1,39,9,43,1,5,43,47,1,47,6,51,1,51,13,55,1,55,9,59,1,59,13,63,2,63,13,67,1,67,10,71,1,71,6,75,2,10,75,79,2,10,79,83,1,5,83,87,2,6,87,91,1,91,6,95,1,95,13,99,2,99,13,103,1,103,9,107,1,10,107,111,2,111,13,115,1,10,115,119,1,10,119,123,2,13,123,127,2,6,127,131,1,13,131,135,1,135,2,139,1,139,6,0,99,2,0,14,0";

    //    println!("Hello, world, {:?}", str_to_intcode(input));
    println!("{:?}", parameter_modes(1009));
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

fn parameter_modes(opcode: i32) -> (u8, u8, u8, u8) {
    let a = opcode / 10000;
    let b = (opcode - a * 10000) / 1000;
    let c = (opcode - a * 10000 - b * 1000) / 100;
    let d = (opcode - a * 10000 - b * 1000 - c * 100);

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

fn computer(state: &mut State) -> Result<ComputeResult, String> {
    let offset = state.instruction_pointer;
    assert!(
        offset < state.intcode.len() as u32,
        "offset {} out of bounds, intcode length {}",
        offset,
        state.intcode.len()
    );
    assert!(state.intcode.len() > 0, "no intcode to process");

    let opcode = state.intcode[offset as usize];
    if opcode == 1 {
        let pos_a = state.intcode[(offset + 1) as usize];
        let pos_b = state.intcode[(offset + 2) as usize];
        let pos_to = state.intcode[(offset + 3) as usize];

        state.intcode[pos_to as usize] =
            state.intcode[pos_a as usize] + state.intcode[pos_b as usize];
        state.instruction_pointer += 4;

        computer(state)
    } else if opcode == 2 {
        let pos_a = state.intcode[(offset + 1) as usize];
        let pos_b = state.intcode[(offset + 2) as usize];
        let pos_to = state.intcode[(offset + 3) as usize];

        state.intcode[pos_to as usize] =
            state.intcode[pos_a as usize] * state.intcode[pos_b as usize];
        state.instruction_pointer += 4;

        computer(state)
    } else if opcode == 3 {
        let pos_to = state.intcode[(offset + 1) as usize];

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
    } else if opcode == 4 {
        let pos_from = state.intcode[(offset + 1) as usize];

        state.output.push(state.intcode[pos_from as usize]);
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

        assert_eq!(true, computer(&mut state).is_ok());
        assert_eq!(&30i32, state.intcode.first().unwrap())
    }

    #[test]
    fn day2_part_one() {
        let mut state = state_from_string("1,12,2,3,1,1,2,3,1,3,4,3,1,5,0,3,2,6,1,19,1,19,5,23,2,9,23,27,1,5,27,31,1,5,31,35,1,35,13,39,1,39,9,43,1,5,43,47,1,47,6,51,1,51,13,55,1,55,9,59,1,59,13,63,2,63,13,67,1,67,10,71,1,71,6,75,2,10,75,79,2,10,79,83,1,5,83,87,2,6,87,91,1,91,6,95,1,95,13,99,2,99,13,103,1,103,9,107,1,10,107,111,2,111,13,115,1,10,115,119,1,10,119,123,2,13,123,127,2,6,127,131,1,13,131,135,1,135,2,139,1,139,6,0,99,2,0,14,0");
        assert_eq!(true, computer(&mut state).is_ok());
        assert_eq!(&4090689i32, state.intcode.first().unwrap())
    }

    #[test]
    fn input_output() {
        let mut state = state_from_string("3,0,4,0,99");
        state.input.push(55);

        assert_eq!(true, computer(&mut state).is_ok());
        assert_eq!(&55i32, state.output.first().unwrap())
    }
}
