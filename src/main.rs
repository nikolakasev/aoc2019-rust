fn main() {
    let input = "1,0,0,3,1,1,2,3,1,3,4,3,1,5,0,3,2,6,1,19,1,19,5,23,2,9,23,27,1,5,27,31,1,5,31,35,1,35,13,39,1,39,9,43,1,5,43,47,1,47,6,51,1,51,13,55,1,55,9,59,1,59,13,63,2,63,13,67,1,67,10,71,1,71,6,75,2,10,75,79,2,10,79,83,1,5,83,87,2,6,87,91,1,91,6,95,1,95,13,99,2,99,13,103,1,103,9,107,1,10,107,111,2,111,13,115,1,10,115,119,1,10,119,123,2,13,123,127,2,6,127,131,1,13,131,135,1,135,2,139,1,139,6,0,99,2,0,14,0";

    println!("Hello, world, {:?}", str_to_intcode(input));
}

fn str_to_intcode(opcode: &str) -> Vec<u32> {
    opcode.split_terminator(",")
        .map(|s| s.parse().unwrap())
        .collect()
}

struct State {
    offset: u32,
    intcode: Vec<u32>,
}

fn computer(state: State) {
    let offset = state.offset;
    assert!(offset < state.intcode.len() as u32, "offset {} out of bounds, intcode length {}", offset, state.intcode.len());
    assert!(state.intcode.len() > 0, "no intcode to process");

    let opcode = state.intcode[offset];
    if opcode == 1 {
        state.intcode[offset + 3] = state.intcode[offset + 1] + state.intcode[offset + 2];
        computer(state);
    } else if opcode == 2 {
        state.intcode[offset + 3] = state.intcode[offset + 1] - state.intcode[offset + 2];
        computer(state);
    } else if opcode == 99 {
        println!("Halt. Result = {:?}", state.intcode);
    } else {
        panic!("Unknown opcode {}", opcode);
    }
}

#[cfg(test)]
mod tests {
    use crate::{str_to_intcode, State, computer};

    #[test]
    fn can_parse_intcode() {
        assert_eq!(vec![1, 0, 0, 0, 99], str_to_intcode("1,0,0,0,99"));
    }

    #[test]
    fn day2_part_one() {
        let input = "1,0,0,3,1,1,2,3,1,3,4,3,1,5,0,3,2,6,1,19,1,19,5,23,2,9,23,27,1,5,27,31,1,5,31,35,1,35,13,39,1,39,9,43,1,5,43,47,1,47,6,51,1,51,13,55,1,55,9,59,1,59,13,63,2,63,13,67,1,67,10,71,1,71,6,75,2,10,75,79,2,10,79,83,1,5,83,87,2,6,87,91,1,91,6,95,1,95,13,99,2,99,13,103,1,103,9,107,1,10,107,111,2,111,13,115,1,10,115,119,1,10,119,123,2,13,123,127,2,6,127,131,1,13,131,135,1,135,2,139,1,139,6,0,99,2,0,14,0";

        let state = State { offset: 0, intcode: str_to_intcode(input) };

        computer(state);
    }
}