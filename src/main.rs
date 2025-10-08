use std::io::Result;

mod function_ast;
mod ass_gen;
mod ir_gen;


use lalrpop_util::lalrpop_mod;

lalrpop_mod!(pub sysy);


fn show_help_and_exit() -> ! {
    eprintln!("Usage: cargo run -- [-koopa|-riscv] <input_path> -o <output_path>");
    std::process::exit(-1);
}


fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() < 5 || args[1] != "-koopa" && args[1] != "-riscv" || args[3] != "-o" {
        show_help_and_exit();
    }

    let input  = &args[2];
    let output = &args[4];

    let parser = sysy::CompUnitParser::new();

    let input_string = std::fs::read_to_string(input)?;
    let ast = parser.parse(&input_string);

    let ir_program = match ast {
        Ok(ast) => {
            let koopa_ir_generator = ir_gen::IrGen::new();
            let result = koopa_ir_generator.generate_koopa_ir(ast).unwrap();
            if args[1] == "-koopa" {
                let mut koopa_ir_text_generator = koopa::back::KoopaGenerator::new(Vec::new());
                koopa_ir_text_generator.generate_on(&result).unwrap();
                let text = String::from_utf8(koopa_ir_text_generator.writer()).unwrap();
                std::fs::write(output, text)?;
            }
            result
        },
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(-1)
        }
    };

    if args[1] == "-riscv" {
        let mut compiler = ass_gen::AssGen::new(&ir_program);
        // 如果生成失败，程序会直接崩溃的，不用担心
        compiler.generate_program();
        std::fs::write(output, compiler.finish())?;
    }

    Ok(())
}