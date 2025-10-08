use std::collections::HashMap;
use std::fmt::Write;

use koopa::ir::{BinaryOp, FunctionData, Program, Value, ValueKind};
use koopa::ir::dfg::DataFlowGraph;

pub mod generate_instruction;

/// 集中式代码生成上下文
/// 持有对 Program 的只读引用，以及内部输出缓冲区。
pub struct AssGen<'p> {
    prog: &'p Program,
    out: String,
    // 后续可扩展：当前函数信息/寄存器分配/块标签映射/目标选项等
    // 简易符号表：koopa::ir::Value 映射到一个 String（分配到的寄存器）
    symbol_table: HashMap<Value, String>,
    // 寄存器分配状态：寄存器名称-是否占用
    register_status: HashMap<String, bool>
}

impl<'p> AssGen<'p> {
    pub fn new(prog: &'p Program) -> Self {
        let register_info = vec![
            (String::from("t0"), false),
            (String::from("t1"), false),
            (String::from("t2"), false),
            (String::from("t3"), false),
            (String::from("t4"), false),
            (String::from("t5"), false),
            (String::from("t6"), false),
            (String::from("a0"), false),
            (String::from("a1"), false),
            (String::from("a2"), false),
            (String::from("a3"), false),
            (String::from("a4"), false),
            (String::from("a5"), false),
            (String::from("a6"), false),
            (String::from("a7"), false),
        ];
        Self {
            prog,
            out: String::new(),
            symbol_table: HashMap::new(),
            register_status: register_info.into_iter().collect()
        }
    }

    /// 结束并获取最终汇编字符串
    pub fn finish(self) -> String {
        self.out
    }

    /// 生成整个 Program 的汇编
    pub fn generate_program(&mut self) {
        writeln!(self.out, "\t.text").unwrap();
        // 先声明每个函数符号
        for &func in self.prog.func_layout() {
            let f = self.prog.func(func);
            writeln!(self.out, "\t.globl {}", self.strip_symbol_prefix(f.name())).unwrap();
        }
        // 然后生成每个函数体
        for &func in self.prog.func_layout() {
            let f = self.prog.func(func);
            self.generate_function(f);
            // 函数间空行，便于阅读
            writeln!(self.out).unwrap();
        }
    }

    /// 将一个符号名称去除可能具有的前置 @ 和 % 符号。
    fn strip_symbol_prefix<'a>(&self, s: &'a str) -> &'a str {
        if let Some(c) = s.chars().next()
            && (c == '@' || c == '%') {
                return &s[1..];
            }
        s
    }

    /// 返回一个当前没有占用的寄存器。
    /// 如果所有寄存器都已经满了，则返回 None
    /// 请注意：由于哈希表在遍历时的不确定性，寄存器是随机分配的，没有任何顺序和规律。
    fn fresh_register(&mut self, value: &Value) -> Option<String> {
        for (name, used) in self.register_status.iter_mut() {
            if !*used {
                *used = true; // 标记为占用
                self.symbol_table.insert(*value, name.clone());
                return Some(name.clone());
            }
        }
        None
    }

    /// 尝试从符号表中寻找一个值已经被分配的寄存器。如果失败，则分配一个新的寄存器并返回。
    fn find_or_allocate_register(&mut self, value: &Value) -> Option<String> {
        if let Some(result) = self.symbol_table.get(value) {
            Some(result.clone())
        } else {
            Some(self.fresh_register(value)?)
        }
    }

    /// find_or_allocate_register 的简化版本
    /// 此函数尝试为某个 value 分配或者获得寄存器，并且在无法分配时 panic
    /// check_x0：为真时，如果输入为 Integer 的 0，那么为其分配 x0 寄存器。
    fn get_register_for_value(&mut self, dfg: &DataFlowGraph, value: &Value, check_x0: bool) -> String {
        let data = dfg.value(*value);
        if let ValueKind::Integer(i) = data.kind() {
            if i.value() == 0 && check_x0 {
                String::from("x0")
            } else {
                self.find_or_allocate_register(value).expect("程序太复杂了，寄存器数量不足：）")
            }
        } else {
            self.find_or_allocate_register(value).expect("程序太复杂了，寄存器数量不足：）")
        }
    }

    /// 生成单个函数的汇编
    fn generate_function(&mut self, f: &FunctionData) {
        writeln!(self.out, "{}:", self.strip_symbol_prefix(f.name())).unwrap();
        let dfg = f.dfg();

        for (&_bb, bb_node) in f.layout().bbs() {
            // 如需基本块标签，可在此处输出
            for &inst in bb_node.insts().keys() {
                self.generate_instruction(dfg, inst);
            }
        }
    }

    /// 生成一条指令/值对应的汇编
    fn generate_instruction(&mut self, dfg: &DataFlowGraph, inst: Value) {
        match dfg.value(inst).kind() {
            // 处理返回语句
            ValueKind::Return(return_value) => {
                if let Some(return_value) = return_value.value() {
                    match dfg.value(return_value).kind() {
                        ValueKind::Integer(i) => {
                            self.return_value_inst(Some(i.value()));
                        },
                        ValueKind::Undef(_) => {
                            self.return_value_inst(None);
                        },
                        _ => {
                            let register = self.symbol_table.get(&return_value);
                            match register {
                                Some(register) => {
                                    self.return_register_inst(&register.clone());
                                },
                                None => {
                                    println!("{:?}", return_value);
                                    println!("{:?}", dfg.value(return_value));
                                    println!("{:?}", self.symbol_table);
                                    panic!("错误：函数返回值为非 i32 类型，但其不存在于符号表中。")
                                }
                            }
                        }
                    }
                } else {
                    writeln!(self.out, "\t").unwrap();
                }
            },
            // 处理二元运算语句
            ValueKind::Binary(binary) => {
                match binary.op() {
                    BinaryOp::Eq => {
                        let result_register = self.fresh_register(&inst).expect("程序太复杂了，寄存器数量不足以存放所有变量。");
                        // 操作数表达式本身
                        let left_exp = dfg.value(binary.lhs());
                        let right_exp = dfg.value(binary.rhs());
                        // 操作数的寄存器
                        // 如果操作数是 0，省略寄存器分配
                        let left_register = self.get_register_for_value(dfg, &binary.lhs(), true);
                        let right_register = self.get_register_for_value(dfg, &binary.rhs(), true);
                        
                        // 如果操作数表达式是立即数，那么我们就生成立即数赋值语句 li rs,?
                        if let ValueKind::Integer(i) = left_exp.kind() {
                            self.init_register(&left_register, i.value());
                        }
                        if let ValueKind::Integer(i) = right_exp.kind() {
                            self.init_register(&right_register, i.value());
                        }
                        // 清空目标寄存器
                        self.clear_register(&result_register);
                        self.sub_inst(&result_register, &left_register, &right_register);
                        self.eq0_inst(&result_register);
                    },
                    BinaryOp::Sub => {
                        let result_register = self.fresh_register(&inst).expect("程序太复杂了，寄存器数量不足以存放所有变量。");
                        // 操作数表达式本身
                        let left_exp = dfg.value(binary.lhs());
                        let right_exp = dfg.value(binary.rhs());
                        // 操作数的寄存器
                        // 如果操作数是 0，省略寄存器分配
                        let left_register = if let ValueKind::Integer(i) = left_exp.kind() {
                            if i.value() == 0 {
                                String::from("x0")
                            } else {
                                self.find_or_allocate_register(&binary.lhs()).expect("程序太复杂了，寄存器数量不足：）")
                            }
                        } else {
                            self.find_or_allocate_register(&binary.lhs()).expect("程序太复杂了，寄存器数量不足：）")
                        };
                        let right_register = if let ValueKind::Integer(i) = right_exp.kind() {
                            if i.value() == 0 {
                                String::from("x0")
                            } else {
                                self.find_or_allocate_register(&binary.rhs()).expect("程序太复杂了，寄存器数量不足：）")
                            }
                        } else {
                            self.find_or_allocate_register(&binary.rhs()).expect("程序太复杂了，寄存器数量不足：）")
                        };
                        // 如果操作数表达式是立即数，那么我们就生成立即数赋值语句 li rs,?
                        if let ValueKind::Integer(i) = left_exp.kind() {
                            self.init_register(&left_register, i.value());
                        }
                        if let ValueKind::Integer(i) = right_exp.kind() {
                            self.init_register(&right_register, i.value());
                        }
                        self.sub_inst(&result_register, &left_register, &right_register);
                    },
                    _ => todo!()
                }
            }
            _ => todo!()
        }
    }
}