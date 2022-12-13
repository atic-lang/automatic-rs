use std::{
    alloc::Layout,
    cell::Ref,
    collections::LinkedList,
    fmt::Debug,
    mem::{size_of, ManuallyDrop, MaybeUninit},
    ptr,
    rc::Rc,
};

use crate::{linker::Instruction, linker::Instruction::*};

#[derive(Clone, Copy)]
union AticObj<'v> {
    as_number: f64,
    as_text: &'v String,
    as_object: &'v Box<[AticObj<'v>]>,
}

pub struct VM<'a> {
    instructions: Box<[Instruction]>,
    stack: Box<[AticObj<'a>; 10000]>,
    call_buffer: Box<[AticObj<'a>; 255]>,
    call_stack: Box<[usize; 10000]>,
    activation_record_pointer: usize,
    stack_pointer: usize,
    object_list: LinkedList<Box<[AticObj<'a>]>>,
    active_record_size: usize,
    pc: usize,
    running: bool,
    pub dbg_iter: i64,
    exit_code: f64,
}

impl VM<'_> {
    pub fn new(instructions: Vec<Instruction>) -> VM<'static> {
        println!("Starting vm....");
        unsafe {
            let callbuffer = std::alloc::alloc_zeroed(Layout::new::<[AticObj; 255]>());
            let callbuffer = (callbuffer as *mut [AticObj; 255]);

            let array = std::alloc::alloc_zeroed(Layout::new::<[AticObj; 10000]>());
            let array = (array as *mut [AticObj; 10000]);

            let call_stack = std::alloc::alloc_zeroed(Layout::new::<[usize; 10000]>());
            let call_stack = (call_stack as *mut [usize; 10000]);

            VM {
                instructions: instructions.into_boxed_slice(),
                stack: Box::new(*array),
                call_buffer: Box::new(*callbuffer),
                call_stack: Box::new(*call_stack),
                activation_record_pointer: 0,
                object_list: LinkedList::new(),
                stack_pointer: 0,
                active_record_size: 0,
                pc: 0,
                running: true,
                dbg_iter: 0,
                exit_code: 0.0,
            }
        }
    }
    pub fn start(&mut self, adress: usize, arg_size: usize) {
        println!("Starting vm....");

        self.pc += 1;
        self.call_stack[0] = (self.pc + 1) as usize;
        self.call_stack[1] = self.active_record_size as usize;
        self.call_stack[2] = 0;
        self.activation_record_pointer += self.active_record_size;
        self.stack_pointer += 3;
        self.active_record_size += arg_size;
        self.pc = adress;
        println!("Size {}", size_of::<Box<AticObj>>());
        println!("Size {}", size_of::<Rc<AticObj>>());
    }

    pub fn tick(&mut self) {
        println!("Ticking VM");
        self.dbg_iter += 1;
        let instruction = &self.instructions[self.pc];

        match instruction {
            Argument(index, src) => {
                self.call_buffer[*index as usize] =
                    self.stack[self.active_record_size + (*src as usize)];
                self.pc += 1;
            }
            CreateStruct(register, size) => {
                let mut obj = Vec::with_capacity(*size as usize);
                // for i in 0..*size {
                //     obj.push(AticObj{as_number: 0.0});
                // }\
                unsafe {
                    core::ptr::copy(self.call_buffer.as_ptr(), obj.as_mut_ptr(), *size as usize);
                    let obj_a = std::alloc::alloc_zeroed(Layout::new::<Box<AticObj>>());
                    let pt = (obj_a as *mut Box<AticObj<'static>>);
                    let box_ = obj.into_boxed_slice();
                    
                  //  let obj_a = (array as *mut [AticObj; 10000]);
                }

                self.stack[self.activation_record_pointer + *register as usize] =
                    AticObj { as_object: &box_ };

                self.object_list.push_front(box_);
            }
            Debug(register) => {
                let object = &self.stack[(self.activation_record_pointer + (*register) as usize)];
                unsafe {
                    println!("> {}: {}", register, object.as_number);
                }
                self.pc += 1;
            }
            Add(target, src_a, src_b) => {
                let s1 = &self.stack[(self.activation_record_pointer + (*src_a) as usize)];
                let s2 = &self.stack[(self.activation_record_pointer + (*src_b) as usize)];
                unsafe {
                    self.stack[(self.activation_record_pointer + (*target) as usize)] = AticObj {
                        as_number: s1.as_number + s2.as_number,
                    };
                }
                self.pc += 1;
            }
            Smaller(target, src_a, src_b) => {
                let s1 = &self.stack[(self.activation_record_pointer + (*src_a) as usize)];
                let s2 = &self.stack[(self.activation_record_pointer + (*src_b) as usize)];

                unsafe {
                    self.stack[(self.activation_record_pointer + *target as usize)] = AticObj {
                        as_number: if s1.as_number < s2.as_number {
                            1.0
                        } else {
                            0.0
                        },
                    };
                }
                self.pc += 1;
            }
            LoadConst(target, constant) => {
                self.stack[(self.activation_record_pointer + *target as usize)] = AticObj {
                    as_number: *constant,
                };
                self.pc += 1;
            }
            Jump(target) => {
                self.pc = target.adress as usize;
            }
            JumpIfNot(register, target) => {
                let s1 = &self.stack[(self.activation_record_pointer + *register as usize)];
                unsafe {
                    if s1.as_number < 0.5 {
                        self.pc = target.adress as usize;
                    } else {
                        self.pc += 1;
                    };
                }
            }
            Exit(code) => {
                self.running = false;
                self.exit_code = *code;
                println!("Exit with code {}", *code);
            }
            _ => {
                panic!("Instruction not implemented");
            }
        }
    }

    pub fn running(&self) -> bool {
        self.running
    }
    pub fn exit_code(&self) -> f64 {
        self.exit_code
    }
}

pub struct Callable {
    pub name: Box<String>,
    pub registers: i32,
    pub adress: i32,
    pub args: i32,
    pub capture_size: i32,
    pub capture: Box<[Callable]>,
}
