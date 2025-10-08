use std::fmt::{self, Write};

use koopa::ir::{FunctionData, Program, Value, ValueKind};
use koopa::ir::dfg::DataFlowGraph;

/// 集中式代码生成上下文
/// 持有对 Program 的只读引用，以及内部输出缓冲区。
pub struct Codegen<'p> {
    prog: &'p Program,
    out: String,
    // 后续可扩展：当前函数信息/寄存器分配/块标签映射/目标选项等
}

impl<'p> Codegen<'p> {
    pub fn new(prog: &'p Program) -> Self {
        Self {
            prog,
            out: String::new(),
        }
    }

    /// 结束并获取最终汇编字符串
    pub fn finish(self) -> String {
        self.out
    }

    /// 生成整个 Program 的汇编
    pub fn gen_program(&mut self) -> fmt::Result {
        writeln!(self.out, "\t.text")?;
        // 先声明每个函数符号
        for &func in self.prog.func_layout() {
            let f = self.prog.func(func);
            writeln!(self.out, "\t.globl {}", self.strip_symbol_prefix(f.name()))?;
        }
        // 然后生成每个函数体
        for &func in self.prog.func_layout() {
            let f = self.prog.func(func);
            self.gen_function(f)?;
            // 函数间空行，便于阅读
            writeln!(self.out)?;
        }
        Ok(())
    }

    /// 将一个符号名称去除可能具有的前置 @ 和 % 符号。
    fn strip_symbol_prefix<'a>(&self, s: &'a str) -> &'a str {
        if let Some(c) = s.chars().next()
            && (c == '@' || c == '%') {
                return &s[1..];
            }
        s
    }

    /// 生成单个函数的汇编
    fn gen_function(&mut self, f: &FunctionData) -> fmt::Result {
        writeln!(self.out, "{}:", self.strip_symbol_prefix(f.name()))?;
        let dfg = f.dfg();

        for (&_bb, bb_node) in f.layout().bbs() {
            // 如需基本块标签，可在此处输出
            for &inst in bb_node.insts().keys() {
                self.gen_inst(dfg, inst)?;
            }
        }
        Ok(())
    }

    /// 生成一条指令/值对应的汇编
    fn gen_inst(&mut self, dfg: &DataFlowGraph, inst: Value) -> fmt::Result {
        match dfg.value(inst).kind() {
            ValueKind::Return(return_value) => {
                if let Some(return_value) = return_value.value() {
                    match dfg.value(return_value).kind() {
                        ValueKind::Integer(i) => {
                            writeln!(self.out, "\tli a0,{}", i.value())?;
                            writeln!(self.out, "\tret")
                        }
                        _ => Err(std::fmt::Error{})
                    }
                } else {
                    writeln!(self.out, "\t")
                }
            }
            _ => Ok(())
        }
    }
}