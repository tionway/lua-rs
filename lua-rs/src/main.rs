use binchunk::{Constant, Prototype};
use vm::{instructions::Instruction, opcodes::ArgKind};

pub mod binchunk;
pub mod vm;

fn main() -> std::io::Result<()> {
    let args = std::env::args().collect::<Vec<_>>();

    if args.len() > 1 {
        let data = std::fs::read(&args[1])?;
        let proto = match binchunk::un_dump(&data) {
            Some(p) => p,
            None => {
                println!("Fail to parse lua chunk binary.");
                return Err(std::io::ErrorKind::InvalidInput.into());
            }
        };
        list(&proto);
    }

    Ok(())
}

fn list(proto: &Prototype) {
    print_header(proto);
    print_code(proto);
    print_detail(proto);
    for p in proto.protos.iter() {
        list(p);
    }
}

fn print_header(proto: &Prototype) {
    let mut fn_type = "main";
    if proto.line_defined > 0 {
        fn_type = "function";
    }
    let mut vararg_flag = "";
    if proto.is_vararg > 0 {
        vararg_flag = "+";
    }
    println!(
        "\n{} <{}:{},{}> ({} instructions)",
        fn_type,
        proto.source,
        proto.line_defined,
        proto.last_line_defined,
        proto.code.len()
    );
    print!(
        "{}{} params, {} slots, {} upvalues, ",
        proto.num_params,
        vararg_flag,
        proto.max_stack_size,
        proto.upvalues.len()
    );
    println!(
        "{} locals, {} constants, {} functions.",
        proto.loc_vars.len(),
        proto.constants.len(),
        proto.protos.len()
    );
}

fn print_code(proto: &Prototype) {
    for (index, c) in proto.code.iter().enumerate() {
        let mut line = "-".to_owned();
        if !proto.line_info.is_empty() {
            line = format!("{}", proto.line_info[index]);
        }
        let i = Instruction::new(*c);
        print!("\t{}\t[{}]\t{} \t", index + 1, line, i.opname());
        print_operands(i);
        println!();
    }
}

fn print_operands(i: Instruction) {
    match i.opmode() {
        vm::opcodes::OpMode::IABC => {
            let (a, b, c) = i.ABC();
            print!("{}", a);
            if i.b_mode() != ArgKind::OpArgN {
                if b > 0xFF {
                    print!(" {}", -1 - (b & 0xFF) as i32);
                } else {
                    print!(" {}", b);
                }
            }
            if i.c_mode() != ArgKind::OpArgN {
                if c > 0xFF {
                    print!(" {}", -1 - (c & 0xFF) as i32);
                } else {
                    print!(" {}", c);
                }
            }
        }
        vm::opcodes::OpMode::IABx => {
            let (a, bx) = i.ABx();
            print!("{}", a);
            if i.b_mode() == ArgKind::OpArgK {
                print!(" {}", -1 - bx as i32);
            } else if i.b_mode() == ArgKind::OpArgU {
                print!(" {}", bx);
            }
        }
        vm::opcodes::OpMode::IAsBx => {
            let (a, s_bx) = i.AsBx();
            print!("{} {}", a, s_bx);
        }
        vm::opcodes::OpMode::IAx => {
            print!("{}", -1 - i.Ax() as i32);
        }
    }
}

fn print_detail(proto: &Prototype) {
    println!("constants ({}):", proto.constants.len());
    for (index, constant) in proto.constants.iter().enumerate() {
        println!("\t{}\t{}", index + 1, constant_to_string(constant));
    }

    println!("locals ({}):", proto.loc_vars.len());
    for (index, loc_var) in proto.loc_vars.iter().enumerate() {
        println!(
            "\t{}\t{}\t{}\t{}",
            index,
            loc_var.var_name,
            loc_var.start_pc + 1,
            loc_var.end_pc + 1
        );
    }

    println!("upvalues ({}):", proto.upvalues.len());
    for (index, upval) in proto.upvalues.iter().enumerate() {
        println!(
            "\t{}\t{}\t{}\t{}",
            index,
            upval_name(proto, index),
            upval.in_stack,
            upval.idx
        );
    }
}

fn constant_to_string(c: &Constant) -> String {
    match c {
        Constant::Nil => "nil".to_owned(),
        Constant::Boolean(v) => v.to_string(),
        Constant::Integer(v) => v.to_string(),
        Constant::Number(v) => v.to_string(),
        Constant::ShortString(v) => v.to_string(),
        Constant::LongString(v) => v.to_string(),
    }
}

fn upval_name(proto: &Prototype, index: usize) -> String {
    if !proto.upvalue_names.is_empty() {
        proto.upvalue_names[index].clone()
    } else {
        "-".to_owned()
    }
}
