mod linker;
mod vm;

use num_format::{Locale, ToFormattedString};
use std::time::SystemTime;
use std::{fs::File, io::Read, vec};

use regex::Regex;

use crate::linker::Linker;
use crate::vm::{VM, Callable};
use crate::Node::*;
use crate::ParseEntry::*;

fn main() {

    if true {
        //let f = 1030605077.0;
        //let mut b = 0.0;
        //let time = SystemTime::now();
        //while b < f {
        //    b += 1.0;
        //}
        //let later = SystemTime::now();
        //let length = later.duration_since(time).unwrap().as_millis();
        //println!("Took {}ms in Rust", length);
        //return;
    }

    let lst = Node::construct(&[1, 2, 3, 4, 5, 6]);
    lst.print();

    let mut file = File::open("res/input.txt").unwrap();
    let mut result = String::new();
    file.read_to_string(&mut result)
        .expect("Cant read input File");
    let cfg = ParserConfig::new();
    let mut lines = result.lines();
    let mut time = SystemTime::now();

    let mut list = generate(&mut lines, &cfg).expect("Error generating");
    let mut callables: Vec<Callable> = vec![];
    let mut linker: Linker = Default::default();
    for function in &mut list {
        let adress = linker.instructions.len();
        function.temp_adress = adress as i32;
        let mut referenced= linker.feed_instructions(&function.instructions);
        callables.append(&mut referenced);
    }
    for callable in &mut callables {
        let name = &callable.name;
        let query = list.iter().find(|ele| ele.name.eq(name.as_ref()));
        if let Some(fcn) = query {
            callable.adress = fcn.temp_adress;
            callable.args = fcn.args;
            callable.registers = fcn.size;
        } else {
            panic!("Cant find function {}", name);
        }
    }
    let query = list.iter().find(|ele| ele.name.eq("Main.main")).expect("Find Main Function");
    let mut vm = VM::new(linker.instructions);
    vm.start(query.temp_adress, query.args);
    time = SystemTime::now();
    while vm.running() {
        vm.tick();
    }
    let later = SystemTime::now();
    let length = later.duration_since(time).unwrap().as_millis();
    let length_sec = (length as f64) / 1000.0;
    let format = ((vm.dbg_iter as f64/length_sec) as i64).to_formatted_string(&Locale::en);
    println!("Took {}ms with {} steps ({} Instructions per Second)", length, vm.dbg_iter, format);
}

struct ParserConfig {
    function_regex: Regex,
    label_regex: Regex,
    register_regex: Regex,
    params_regex: Regex,
    define_regex: Regex,
    instruction_regex: Regex,
}
impl ParserConfig {
    fn new() -> ParserConfig {
        ParserConfig {
            function_regex: Regex::new(r"fn\s+([\w.#]+)").unwrap(),
            label_regex: Regex::new(r"#([\w.]+)").unwrap(),
            register_regex: Regex::new(r"registers\s+(.+)?").unwrap(),
            params_regex: Regex::new(r"params\s+(.+)").unwrap(),
            define_regex: Regex::new(r"define\s+(\w+)\s+(.*)").unwrap(),
            instruction_regex: Regex::new(r"\s*(\w+)\s*:\s*(.*)").unwrap(),
        }
    }
}

fn generate(
    lines: &mut dyn Iterator<Item = &str>,
    config: &ParserConfig,
) -> Result<Vec<Function>, String> {
    let mut list = vec![];

    while let Some(arg) = lines.next() {
        if let Some(captures) = config.function_regex.captures(arg) {
            let name = captures.get(1).unwrap().as_str();
            let mut instructions = vec![];
            let mut register_target = None;
            while let Some(arg) = lines.next() {
                if let Some(arg) = config.instruction_regex.captures(arg) {
                    let name = arg.get(1).unwrap().as_str();
                    let args = arg.get(2).unwrap().as_str();
                    let name = name.to_string();
                    let args = args.to_string();
                    let params = transform_arguments(args);
                    instructions.push(ParseInstruction(UnparsedInstruction { name, params }));
                } else if let Some(arg) = config.label_regex.captures(arg) {
                    let jump_label = arg.get(1).unwrap().as_str();
                    instructions.push(ParseLabel(jump_label.trim().to_string()));
                } else {
                    if let Some(params) = config.register_regex.captures(arg) {
                        let parse = params.get(1).unwrap().as_str().to_string().parse::<f64>();
                        if let Ok(double) = parse {
                            register_target = Some(double as i32);
                        } else {
                            return Err("Cant parse register argument as number".to_string());
                        }
                    } else {
                        return Err(format!("Expected \"registers\" parameter, but got {}", arg));
                    }
                    break;
                }
            }

            let params = match lines.next() {
                Some(params) => {
                    if let Some(params) = config.params_regex.captures(params) {
                        let parse = params.get(1).unwrap().as_str().to_string().parse::<f64>();
                        if let Ok(double) = parse {
                            double as i32
                        } else {
                            return Err("Cant parse params argument as number".to_string());
                        }
                    } else {
                        return Err("Expected \"params\" parameter".to_string());
                    }
                }
                None => {
                    return Err("Missing \"params\" parameter".to_string());
                }
            };

            match lines.next() {
                Some(params) => {
                    if params != "end" {
                        return Err("Expected \"end\" parameter".to_string());
                    }
                }
                None => {
                    return Err("Missing \"end\" parameter".to_string());
                }
            };

            list.push(Function {
                name: name.to_string(),
                args: params,
                instructions,
                size: register_target.unwrap(),
                temp_adress: 0,
            });
        }
    }
    Ok(list)
}

struct Function {
    name: String,
    size: i32,
    instructions: Vec<ParseEntry>,
    args: i32,
    temp_adress: i32,
}

pub enum ParseEntry {
    ParseInstruction(UnparsedInstruction),
    ParseLabel(String),
}

//TODO make name and params type &str
pub struct UnparsedInstruction {
    name: String,
    params: Vec<String>,
}

fn transform_arguments(str: String) -> Vec<String> {
    let mut arguments = vec![];
    let mut iter = str.chars();
    let mut inside_str = false;
    let mut bob = String::new();
    while let Some(char) = iter.next() {
        if char == ',' && !inside_str {
            arguments.push(bob.clone());
            bob.clear();
            continue;
        }
        if char.is_whitespace() && !inside_str {
            continue;
        }

        if char == '"' {
            inside_str = !inside_str;
            continue;
        }

        bob.push(char);
        if inside_str && char == '\\' {
            bob.push(iter.next().unwrap());
        }
    }
    if bob.len() != 0 {
        arguments.push(bob);
    }

    arguments
}

enum Node {
    Unit(i32, Box<Node>),
    End,
}

impl Node {
    fn construct(list: &[i32]) -> Node {
        let mut prev = End;
        for element in list.iter().rev() {
            prev = Unit(*element, Box::new(prev))
        }
        prev
    }

    fn print(&self) {
        match self {
            Unit(element, sub_lst) => {
                print!("{} ", element);
                sub_lst.print();
            }
            _ => println!(""),
        }
    }
}
