use crate::intcode::ComputeResult::{CanContinue, Halt, WaitingForInput};
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::{collections::VecDeque, sync::mpsc};

struct State {
    instruction_pointer: u32,
    intcode: Vec<i64>,
    input: VecDeque<i64>,
    output: Vec<i64>,
    relative_base: i64,
}

enum ComputeResult {
    Halt,
    CanContinue,
    WaitingForInput,
}

fn str_to_intcode(string: &str) -> Vec<i64> {
    string
        .split_terminator(',')
        .map(|s| s.parse().unwrap())
        .collect()
}

fn state_from_string(string: &str) -> State {
    State {
        instruction_pointer: 0,
        intcode: str_to_intcode(string),
        input: VecDeque::new(),
        output: vec![],
        relative_base: 0,
    }
}

//todo turn into an enumeration instead of using u8 for the parameter modes?
fn parameter_modes(opcode: i64) -> (u8, u8, u8, u8) {
    let a = opcode / 10000;
    let b = (opcode - a * 10000) / 1000;
    let c = (opcode - a * 10000 - b * 1000) / 100;
    let d = opcode - a * 10000 - b * 1000 - c * 100;

    (a as u8, b as u8, c as u8, d as u8)
}

//todo how to deal with the situation when the state is invalid or the parameter mode isn't supported?
fn get_value(parameter_mode: u8, pointer: u32, state: &State) -> i64 {
    //position mode
    if parameter_mode == 0 {
        let at_index = state.intcode[pointer as usize];
        state.intcode[at_index as usize]
    }
    //immediate mode
    else if parameter_mode == 1 {
        state.intcode[pointer as usize]
    } else if parameter_mode == 2 {
        let at_index = state.intcode[pointer as usize] + state.relative_base as i64;
        state.intcode[at_index as usize]
    } else {
        panic!("parameter mode {} not supported", parameter_mode)
    }
}

fn extend_memory(parameters: Vec<(u8, u32)>, state: &mut State) {
    //get the maximum memory address
    let memory_index = parameters
        .into_iter()
        //not relevant for immediate mode, so only for position and relative base modes
        .filter(|(mode, _)| *mode == 0 || *mode == 2)
        //calculate the memory address for each parameter
        .map(|(mode, pointer)| get_memory_address(mode, pointer, state))
        .max()
        .unwrap() as usize;

    //extend the memory of necessary
    if memory_index >= state.intcode.len() {
        state.intcode.resize((memory_index + 1) as usize, 0);
    }
}

fn get_memory_address(parameter_mode: u8, pointer: u32, state: &State) -> i64 {
    //position mode
    if parameter_mode == 0 {
        state.intcode[pointer as usize]
    }
    //immediate mode
    else if parameter_mode == 1 {
        panic!("writing to memory will never be in immediate mode")
    }
    //relative mode
    else if parameter_mode == 2 {
        state.intcode[pointer as usize] + state.relative_base as i64
    } else {
        panic!("parameter mode {} not supported", parameter_mode)
    }
}

fn compute(state: &mut State) -> Result<ComputeResult, String> {
    let offset = state.instruction_pointer;

    //todo is this defensive programming a good idea?
    assert!(
        offset < state.intcode.len() as u32,
        "offset {} out of bounds, intcode length {}",
        offset,
        state.intcode.len()
    );
    assert!(!state.intcode.is_empty(), "no intcode to process");

    let (a, b, c, opcode) = parameter_modes(state.intcode[offset as usize]);

    //add
    if opcode == 1 {
        extend_memory(
            vec![(c, offset + 1), (b, offset + 2), (a, offset + 3)],
            state,
        );

        let memory_address = get_memory_address(a, offset + 3, state);
        let first_parameter = get_value(c, offset + 1, state);
        let second_parameter = get_value(b, offset + 2, state);

        state.intcode[memory_address as usize] = first_parameter + second_parameter;
        state.instruction_pointer += 4;

        Ok(CanContinue)
    }
    //multiply
    else if opcode == 2 {
        extend_memory(
            vec![(c, offset + 1), (b, offset + 2), (a, offset + 3)],
            state,
        );

        let memory_address = get_memory_address(a, offset + 3, state);
        let first_parameter = get_value(c, offset + 1, state);
        let second_parameter = get_value(b, offset + 2, state);

        state.intcode[memory_address as usize] = first_parameter * second_parameter;
        state.instruction_pointer += 4;

        Ok(CanContinue)
    }
    //input
    else if opcode == 3 {
        let memory_address = get_memory_address(c, offset + 1, state);

        //attempt to read from the input
        match state.input.pop_front() {
            Some(v) => {
                extend_memory(vec![(c, offset + 1)], state);

                state.intcode[memory_address as usize] = v as i64;
                state.instruction_pointer += 2;

                Ok(CanContinue)
            }
            None => Ok(WaitingForInput),
        }
    }
    //output
    else if opcode == 4 {
        let value_to_output = get_value(c, offset + 1, state);

        state.output.push(value_to_output);
        state.instruction_pointer += 2;

        Ok(CanContinue)
    }
    //jump if true
    else if opcode == 5 {
        let first_parameter = get_value(c, offset + 1, state);
        let second_parameter = get_value(b, offset + 2, state);

        if first_parameter != 0 {
            state.instruction_pointer = second_parameter as u32;
        } else {
            state.instruction_pointer += 3;
        }

        Ok(CanContinue)
    }
    //jump if false
    else if opcode == 6 {
        let first_parameter = get_value(c, offset + 1, state);
        let second_parameter = get_value(b, offset + 2, state);

        if first_parameter == 0 {
            state.instruction_pointer = second_parameter as u32;
        } else {
            state.instruction_pointer += 3;
        }

        Ok(CanContinue)
    }
    //less than
    //todo refactor because the only difference in the logic for opcode 7 and 8 is '<' vs. '==', lambda or something?
    else if opcode == 7 {
        extend_memory(
            vec![(c, offset + 1), (b, offset + 2), (a, offset + 3)],
            state,
        );

        let memory_address = get_memory_address(a, offset + 3, state);
        let first_parameter = get_value(c, offset + 1, state);
        let second_parameter = get_value(b, offset + 2, state);
        let value = if first_parameter < second_parameter {
            1
        } else {
            0
        };

        state.intcode[memory_address as usize] = value;
        state.instruction_pointer += 4;

        Ok(CanContinue)
    }
    //equals
    else if opcode == 8 {
        extend_memory(
            vec![(c, offset + 1), (b, offset + 2), (a, offset + 3)],
            state,
        );

        let memory_address = get_memory_address(a, offset + 3, state);
        let first_parameter = get_value(c, offset + 1, state);
        let second_parameter = get_value(b, offset + 2, state);
        let value = if first_parameter == second_parameter {
            1
        } else {
            0
        };

        state.intcode[memory_address as usize] = value;
        state.instruction_pointer += 4;

        Ok(CanContinue)
    }
    //adjust relative base
    else if opcode == 9 {
        let first_parameter = get_value(c, offset + 1, state);

        state.relative_base += first_parameter;
        state.instruction_pointer += 2;

        Ok(CanContinue)
    } else if opcode == 99 {
        Ok(Halt)
    } else {
        let error = format!("{} {}", "Unknown opcode", opcode);
        Err(error)
    }
}

pub fn computer(intcode: &str, input: Vec<i64>) -> Result<Vec<i64>, String> {
    let mut state = state_from_string(intcode);
    state.input = input.into_iter().collect();

    loop {
        match compute(&mut state) {
            Ok(r) => match r {
                Halt | WaitingForInput => break Ok(state.output),
                CanContinue => continue,
            },
            //todo refactor the nested match and simplify the error mapping
            Err(e) => break Err(e),
        }
    }
}

fn pop_and_send(state: &mut State, rx: &Sender<i64>) {
    //why is this not sending?
    //state.output.drain(..).map(|v| rx.send(v).unwrap());
    for v in state.output.drain(..) {
        rx.send(v).unwrap()
    }
}

pub fn async_computer(intcode: &str, name: &str, rx: Receiver<i64>, tx: Sender<i64>) {
    let mut state = state_from_string(intcode);

    loop {
        match compute(&mut state) {
            Ok(r) => match r {
                Halt => {
                    break pop_and_send(&mut state, &tx);
                }

                WaitingForInput => {
                    pop_and_send(&mut state, &tx);

                    match rx.recv() {
                        Ok(v) => {
                            state.input.push_back(v);
                            continue;
                        }
                        Err(e) => panic!("{} error: {}", name, e),
                    }
                }

                CanContinue => continue,
            },
            Err(e) => panic!("{} error: {}", name, e),
        }
    }
}

fn five_amplifiers_in_sequence(intcode: &str, phase_setting: Vec<i64>) -> i64 {
    computer(intcode, vec![phase_setting[0], 0])
        //todo refactor pop for something immutable?
        .and_then(|mut o| computer(intcode, vec![phase_setting[1], o.pop().unwrap()]))
        .and_then(|mut o| computer(intcode, vec![phase_setting[2], o.pop().unwrap()]))
        .and_then(|mut o| computer(intcode, vec![phase_setting[3], o.pop().unwrap()]))
        .and_then(|mut o| computer(intcode, vec![phase_setting[4], o.pop().unwrap()]))
        //todo extract in a different way?
        .ok()
        .unwrap()
        .pop()
        .unwrap()
}

fn five_amplifiers_in_a_feedback_loop(
    intcode: &'static str,
    phase_setting: Vec<i32>,
) -> Option<i64> {
    assert_eq!(
        phase_setting.len(),
        5,
        "phase sequence of length five expected, while {} provided",
        phase_setting.len()
    );

    let (tx_controller, rx_controller): (Sender<i64>, Receiver<i64>) = mpsc::channel();
    let (tx_a, rx_a): (Sender<i64>, Receiver<i64>) = mpsc::channel();
    let (tx_b, rx_b): (Sender<i64>, Receiver<i64>) = mpsc::channel();
    let (tx_c, rx_c): (Sender<i64>, Receiver<i64>) = mpsc::channel();
    let (tx_d, rx_d): (Sender<i64>, Receiver<i64>) = mpsc::channel();
    let (tx_e, rx_e): (Sender<i64>, Receiver<i64>) = mpsc::channel();

    tx_a.send(phase_setting[0] as i64)
        .and_then(|_| tx_a.send(0))
        .and_then(|_| tx_b.send(phase_setting[1] as i64))
        .and_then(|_| tx_c.send(phase_setting[2] as i64))
        .and_then(|_| tx_d.send(phase_setting[3] as i64))
        .and_then(|_| tx_e.send(phase_setting[4] as i64))
        .unwrap();

    let _a = thread::spawn(move || async_computer(intcode, "A", rx_a, tx_b));
    let _b = thread::spawn(move || async_computer(intcode, "B", rx_b, tx_c));
    let _c = thread::spawn(move || async_computer(intcode, "C", rx_c, tx_d));
    let _d = thread::spawn(move || async_computer(intcode, "D", rx_d, tx_e));
    let _e = thread::spawn(move || async_computer(intcode, "E", rx_e, tx_controller));

    loop {
        match rx_controller.recv() {
            //forward the from E to A
            Ok(v) => match tx_a.send(v) {
                Ok(_) => continue,
                //isn't able to send, because A halted already,
                //so this is the last output from E
                Err(_) => break Some(v),
            },
            //wasn't able to receive because E halted already and closed the channel
            Err(e) => panic!(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::intcode::{
        computer, five_amplifiers_in_a_feedback_loop, five_amplifiers_in_sequence, str_to_intcode,
    };
    use permutohedron::Heap;

    #[test]
    fn can_parse_intcode() {
        assert_eq!(vec![1, 0, 0, 0, 99], str_to_intcode("1,0,0,0,99"));
    }

    #[test]
    fn input_output() {
        assert_output("3,0,4,0,99", Some(55), vec![55])
    }

    #[test]
    fn parameter_modes() {
        assert_output("1002,4,3,4,33", None, vec![])
    }

    #[test]
    fn relative_base() {
        assert_output(
            "109,1,204,-1,1001,100,1,100,1008,100,16,101,1006,101,0,99",
            None,
            vec![
                109, 1, 204, -1, 1001, 100, 1, 100, 1008, 100, 16, 101, 1006, 101, 0, 99,
            ],
        )
    }

    #[test]
    fn large_numbers() {
        assert_output("104,1125899906842624,99", None, vec![1125899906842624]);
        assert_output(
            "1102,34915192,34915192,7,4,7,99,0",
            None,
            vec![1219070632396864],
        )
    }

    fn input_day5() -> &'static str {
        "3,225,1,225,6,6,1100,1,238,225,104,0,1,192,154,224,101,-161,224,224,4,224,102,8,223,223,101,5,224,224,1,223,224,223,1001,157,48,224,1001,224,-61,224,4,224,102,8,223,223,101,2,224,224,1,223,224,223,1102,15,28,225,1002,162,75,224,1001,224,-600,224,4,224,1002,223,8,223,1001,224,1,224,1,224,223,223,102,32,57,224,1001,224,-480,224,4,224,102,8,223,223,101,1,224,224,1,224,223,223,1101,6,23,225,1102,15,70,224,1001,224,-1050,224,4,224,1002,223,8,223,101,5,224,224,1,224,223,223,101,53,196,224,1001,224,-63,224,4,224,102,8,223,223,1001,224,3,224,1,224,223,223,1101,64,94,225,1102,13,23,225,1101,41,8,225,2,105,187,224,1001,224,-60,224,4,224,1002,223,8,223,101,6,224,224,1,224,223,223,1101,10,23,225,1101,16,67,225,1101,58,10,225,1101,25,34,224,1001,224,-59,224,4,224,1002,223,8,223,1001,224,3,224,1,223,224,223,4,223,99,0,0,0,677,0,0,0,0,0,0,0,0,0,0,0,1105,0,99999,1105,227,247,1105,1,99999,1005,227,99999,1005,0,256,1105,1,99999,1106,227,99999,1106,0,265,1105,1,99999,1006,0,99999,1006,227,274,1105,1,99999,1105,1,280,1105,1,99999,1,225,225,225,1101,294,0,0,105,1,0,1105,1,99999,1106,0,300,1105,1,99999,1,225,225,225,1101,314,0,0,106,0,0,1105,1,99999,1108,226,226,224,102,2,223,223,1005,224,329,101,1,223,223,107,226,226,224,1002,223,2,223,1005,224,344,1001,223,1,223,107,677,226,224,102,2,223,223,1005,224,359,101,1,223,223,7,677,226,224,102,2,223,223,1005,224,374,101,1,223,223,108,226,226,224,102,2,223,223,1006,224,389,101,1,223,223,1007,677,677,224,102,2,223,223,1005,224,404,101,1,223,223,7,226,677,224,102,2,223,223,1006,224,419,101,1,223,223,1107,226,677,224,1002,223,2,223,1005,224,434,1001,223,1,223,1108,226,677,224,102,2,223,223,1005,224,449,101,1,223,223,108,226,677,224,102,2,223,223,1005,224,464,1001,223,1,223,8,226,677,224,1002,223,2,223,1005,224,479,1001,223,1,223,1007,226,226,224,102,2,223,223,1006,224,494,101,1,223,223,1008,226,677,224,102,2,223,223,1006,224,509,101,1,223,223,1107,677,226,224,1002,223,2,223,1006,224,524,1001,223,1,223,108,677,677,224,1002,223,2,223,1005,224,539,1001,223,1,223,1107,226,226,224,1002,223,2,223,1006,224,554,1001,223,1,223,7,226,226,224,1002,223,2,223,1006,224,569,1001,223,1,223,8,677,226,224,102,2,223,223,1006,224,584,101,1,223,223,1008,677,677,224,102,2,223,223,1005,224,599,101,1,223,223,1007,226,677,224,1002,223,2,223,1006,224,614,1001,223,1,223,8,677,677,224,1002,223,2,223,1005,224,629,101,1,223,223,107,677,677,224,102,2,223,223,1005,224,644,101,1,223,223,1108,677,226,224,102,2,223,223,1005,224,659,101,1,223,223,1008,226,226,224,102,2,223,223,1006,224,674,1001,223,1,223,4,223,99,226"
    }

    #[test]
    fn day5_part_one() {
        assert_output(
            input_day5(),
            Some(1),
            vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 11049715],
        )
    }

    #[test]
    fn day5_part_two() {
        assert_output(input_day5(), Some(5), vec![2140710])
    }

    fn input_day7() -> &'static str {
        "3,8,1001,8,10,8,105,1,0,0,21,42,67,84,97,118,199,280,361,442,99999,3,9,101,4,9,9,102,5,9,9,101,2,9,9,1002,9,2,9,4,9,99,3,9,101,5,9,9,102,5,9,9,1001,9,5,9,102,3,9,9,1001,9,2,9,4,9,99,3,9,1001,9,5,9,1002,9,2,9,1001,9,5,9,4,9,99,3,9,1001,9,5,9,1002,9,3,9,4,9,99,3,9,102,4,9,9,101,4,9,9,102,2,9,9,101,3,9,9,4,9,99,3,9,102,2,9,9,4,9,3,9,1002,9,2,9,4,9,3,9,1001,9,2,9,4,9,3,9,102,2,9,9,4,9,3,9,102,2,9,9,4,9,3,9,1001,9,2,9,4,9,3,9,1002,9,2,9,4,9,3,9,102,2,9,9,4,9,3,9,1001,9,2,9,4,9,3,9,101,2,9,9,4,9,99,3,9,1001,9,1,9,4,9,3,9,101,2,9,9,4,9,3,9,1001,9,2,9,4,9,3,9,1002,9,2,9,4,9,3,9,101,2,9,9,4,9,3,9,1002,9,2,9,4,9,3,9,102,2,9,9,4,9,3,9,1002,9,2,9,4,9,3,9,101,1,9,9,4,9,3,9,101,2,9,9,4,9,99,3,9,101,1,9,9,4,9,3,9,1001,9,1,9,4,9,3,9,1002,9,2,9,4,9,3,9,1002,9,2,9,4,9,3,9,1002,9,2,9,4,9,3,9,1001,9,2,9,4,9,3,9,102,2,9,9,4,9,3,9,102,2,9,9,4,9,3,9,101,2,9,9,4,9,3,9,1001,9,2,9,4,9,99,3,9,102,2,9,9,4,9,3,9,102,2,9,9,4,9,3,9,1001,9,2,9,4,9,3,9,102,2,9,9,4,9,3,9,1001,9,2,9,4,9,3,9,102,2,9,9,4,9,3,9,102,2,9,9,4,9,3,9,101,1,9,9,4,9,3,9,1001,9,2,9,4,9,3,9,1002,9,2,9,4,9,99,3,9,101,1,9,9,4,9,3,9,101,1,9,9,4,9,3,9,102,2,9,9,4,9,3,9,1001,9,2,9,4,9,3,9,1001,9,2,9,4,9,3,9,1002,9,2,9,4,9,3,9,101,1,9,9,4,9,3,9,102,2,9,9,4,9,3,9,1001,9,1,9,4,9,3,9,1001,9,2,9,4,9,99"
    }

    #[test]
    fn day7_examples() {
        assert_eq!(
            five_amplifiers_in_sequence(
                "3,15,3,16,1002,16,10,16,1,16,15,15,4,15,99,0,0",
                vec![4, 3, 2, 1, 0]
            ),
            43210
        );

        assert_eq!(
            five_amplifiers_in_sequence(
                "3,23,3,24,1002,24,10,24,1002,23,-1,23,101,5,23,23,1,24,23,23,4,23,99,0,0",
                vec![0, 1, 2, 3, 4]
            ),
            54321
        );

        assert_eq!(
            five_amplifiers_in_sequence(
                "3,31,3,32,1002,32,10,32,1001,31,-2,31,1007,31,0,33,1002,33,7,33,1,33,31,31,1,32,31,31,4,31,99,0,0,0",
                vec![1, 0, 4, 3, 2]
            ),
            65210
        );

        assert_eq!(
            five_amplifiers_in_a_feedback_loop("3,26,1001,26,-4,26,3,27,1002,27,2,27,1,27,26,27,4,27,1001,28,-1,28,1005,28,6,99,0,0,5",
                                               vec![9, 8, 7, 6, 5]),
            Some(139629729));

        assert_eq!(
            five_amplifiers_in_a_feedback_loop("3,52,1001,52,-5,52,3,53,1,52,56,54,1007,54,5,55,1005,55,26,1001,54,-5,54,1105,1,12,1,53,54,53,1008,54,0,55,1001,55,1,55,2,53,55,53,4,53,1001,56,-1,56,1005,56,6,99,0,0,0,0,10",
                                               vec![9, 7, 8, 5, 6]),
            Some(18216))
    }

    #[test]
    fn day7_part_two() {
        let mut data = vec![5, 6, 7, 8, 9];
        let heap = Heap::new(&mut data);

        let mut permutations = Vec::new();
        for data in heap {
            permutations.push(data.clone());
        }

        let mut res: Vec<i64> = permutations
            .into_iter()
            .map(|phase_setting| {
                five_amplifiers_in_a_feedback_loop(input_day7(), phase_setting).unwrap()
            })
            .collect();

        res.sort();

        assert_eq!(*res.last().unwrap(), 70602018)
    }

    fn input_day9() -> &'static str {
        "1102,34463338,34463338,63,1007,63,34463338,63,1005,63,53,1102,3,1,1000,109,988,209,12,9,1000,209,6,209,3,203,0,1008,1000,1,63,1005,63,65,1008,1000,2,63,1005,63,904,1008,1000,0,63,1005,63,58,4,25,104,0,99,4,0,104,0,99,4,17,104,0,99,0,0,1102,1,30,1010,1102,1,38,1008,1102,1,0,1020,1102,22,1,1007,1102,26,1,1015,1102,31,1,1013,1102,1,27,1014,1101,0,23,1012,1101,0,37,1006,1102,735,1,1028,1102,1,24,1009,1102,1,28,1019,1102,20,1,1017,1101,34,0,1001,1101,259,0,1026,1101,0,33,1018,1102,1,901,1024,1101,21,0,1016,1101,36,0,1011,1102,730,1,1029,1101,1,0,1021,1102,1,509,1022,1102,39,1,1005,1101,35,0,1000,1102,1,506,1023,1101,0,892,1025,1101,256,0,1027,1101,25,0,1002,1102,1,29,1004,1102,32,1,1003,109,9,1202,-3,1,63,1008,63,39,63,1005,63,205,1001,64,1,64,1106,0,207,4,187,1002,64,2,64,109,-2,1208,-4,35,63,1005,63,227,1001,64,1,64,1105,1,229,4,213,1002,64,2,64,109,5,1206,8,243,4,235,1106,0,247,1001,64,1,64,1002,64,2,64,109,14,2106,0,1,1105,1,265,4,253,1001,64,1,64,1002,64,2,64,109,-25,1201,4,0,63,1008,63,40,63,1005,63,285,1106,0,291,4,271,1001,64,1,64,1002,64,2,64,109,14,2107,37,-7,63,1005,63,313,4,297,1001,64,1,64,1106,0,313,1002,64,2,64,109,-7,21101,40,0,5,1008,1013,37,63,1005,63,333,1105,1,339,4,319,1001,64,1,64,1002,64,2,64,109,-7,1207,0,33,63,1005,63,355,1106,0,361,4,345,1001,64,1,64,1002,64,2,64,109,7,21102,41,1,9,1008,1017,41,63,1005,63,387,4,367,1001,64,1,64,1106,0,387,1002,64,2,64,109,-1,21102,42,1,10,1008,1017,43,63,1005,63,411,1001,64,1,64,1106,0,413,4,393,1002,64,2,64,109,-5,21101,43,0,8,1008,1010,43,63,1005,63,435,4,419,1106,0,439,1001,64,1,64,1002,64,2,64,109,16,1206,3,455,1001,64,1,64,1106,0,457,4,445,1002,64,2,64,109,-8,21107,44,45,7,1005,1017,479,4,463,1001,64,1,64,1106,0,479,1002,64,2,64,109,6,1205,5,497,4,485,1001,64,1,64,1106,0,497,1002,64,2,64,109,1,2105,1,6,1105,1,515,4,503,1001,64,1,64,1002,64,2,64,109,-10,2108,36,-1,63,1005,63,535,1001,64,1,64,1105,1,537,4,521,1002,64,2,64,109,-12,2101,0,6,63,1008,63,32,63,1005,63,561,1001,64,1,64,1105,1,563,4,543,1002,64,2,64,109,25,21108,45,46,-2,1005,1018,583,1001,64,1,64,1105,1,585,4,569,1002,64,2,64,109,-23,2108,34,4,63,1005,63,607,4,591,1001,64,1,64,1106,0,607,1002,64,2,64,109,3,1202,7,1,63,1008,63,22,63,1005,63,633,4,613,1001,64,1,64,1106,0,633,1002,64,2,64,109,12,21108,46,46,3,1005,1015,651,4,639,1106,0,655,1001,64,1,64,1002,64,2,64,109,-5,2102,1,-1,63,1008,63,35,63,1005,63,679,1001,64,1,64,1105,1,681,4,661,1002,64,2,64,109,13,21107,47,46,-7,1005,1013,701,1001,64,1,64,1105,1,703,4,687,1002,64,2,64,109,-2,1205,2,715,1106,0,721,4,709,1001,64,1,64,1002,64,2,64,109,17,2106,0,-7,4,727,1105,1,739,1001,64,1,64,1002,64,2,64,109,-23,2107,38,-6,63,1005,63,759,1001,64,1,64,1106,0,761,4,745,1002,64,2,64,109,-3,1207,-4,40,63,1005,63,779,4,767,1105,1,783,1001,64,1,64,1002,64,2,64,109,-8,2101,0,-1,63,1008,63,35,63,1005,63,809,4,789,1001,64,1,64,1105,1,809,1002,64,2,64,109,-6,2102,1,8,63,1008,63,32,63,1005,63,835,4,815,1001,64,1,64,1106,0,835,1002,64,2,64,109,6,1201,5,0,63,1008,63,37,63,1005,63,857,4,841,1106,0,861,1001,64,1,64,1002,64,2,64,109,2,1208,0,32,63,1005,63,883,4,867,1001,64,1,64,1106,0,883,1002,64,2,64,109,23,2105,1,-2,4,889,1001,64,1,64,1106,0,901,4,64,99,21102,27,1,1,21101,0,915,0,1106,0,922,21201,1,55337,1,204,1,99,109,3,1207,-2,3,63,1005,63,964,21201,-2,-1,1,21101,0,942,0,1105,1,922,21202,1,1,-1,21201,-2,-3,1,21102,957,1,0,1105,1,922,22201,1,-1,-2,1106,0,968,21201,-2,0,-2,109,-3,2105,1,0"
    }

    #[test]
    fn day9_part_one() {
        assert_output(input_day9(), Some(1), vec![3765554916])
    }

    fn assert_output(intcode: &str, input: Option<i64>, expected_output: Vec<i64>) {
        assert_eq!(
            computer(intcode, input.map_or(vec![], |v| vec![v])).unwrap(),
            expected_output
        )
    }

    #[test]
    fn day9_part_two() {
        assert_output(input_day9(), Some(2), vec![76642])
    }

    #[test]
    fn extend_memory() {
        let input_works = "1,12,10,1109,4,1109,99,0,0,0,0,0,7";
        assert_output(input_works, None, vec![7]);

        //the regression bug is because only the write parameter is checked, not all of them for the maximum address
        let input_would_break = "1,1109,12,10,4,10,99,0,0,0,0,0,7";
        assert_output(input_would_break, None, vec![7]);
    }
}
