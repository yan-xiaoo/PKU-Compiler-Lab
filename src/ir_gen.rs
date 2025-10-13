use std::{cell::RefCell, collections::HashMap};
use crate::function_ast::{self, CompUnit, FuncDef, FuncType};

use koopa::ir::{builder::{BasicBlockBuilder, LocalInstBuilder}, BasicBlock, FunctionData, Program, Type, Value};

// 将 ir_gen 子模块拆分到 src/ir_gen/ 目录下的多个文件
// 这里显式声明子模块：会加载 "src/ir_gen/unary_statement.rs"
mod unary_statement;
mod arithmetic_statement;
mod logic_statement;

/// IR 生成上下文（面向文本 IR 的阶段性方案）
/// - 负责集中管理全局状态：临时名分配（%0、%1、...）、符号表、标签分配等
/// - 提供一个简单的输出缓冲区，便于逐步迁移现有 Dump 风格的实现
///
/// 后续如果切换为直接构建 Koopa IR（Program/Function/BasicBlock），可以保留本结构，
/// 仅将 `out` 替换/扩展为 koopa::ir::Program + 若干映射/辅助状态。
#[allow(dead_code)]
pub struct IrGen {
    program: RefCell<Program>,
    temp_id: usize,
    label_id: usize,
    /// 可选：标识符到 SSA 名（或 Value 句柄）的映射，后续做语义分析/作用域时可扩展为栈式作用域
    symbols: HashMap<String, String>,
}

impl Default for IrGen {
    fn default() -> Self {
        Self::new()
    }
}

impl IrGen {
    pub fn new() -> Self {
        Self {
            program: RefCell::new(Program::new()),
            temp_id: 0,
            label_id: 0,
            symbols: HashMap::new(),
        }
    }

    pub fn generate_koopa_ir(mut self, parsed_unit: CompUnit) -> Result<Program, String> {
        self.generate_function(parsed_unit.func_def);
        Ok(self.program.take())
    }

    fn generate_function(&mut self, function: FuncDef) {
        let koopa_type = match function.func_type {
            FuncType::Int => Type::get_i32(),
            FuncType::Void => Type::get_unit(),
        };
        let function_data = FunctionData::new(format!("@{}", function.ident), Vec::new(), koopa_type);
        // 必须先添加 function_data 到程序中
        let mut binding = self.program.borrow_mut();
        let func = binding.new_func(function_data);
        let function_data= binding.func_mut(func);
        self.generate_block(function_data, &function.block);
    }

    fn generate_block(&self, function_data: &mut FunctionData, block: &function_ast::Block) {
        // 后续在块内容变复杂之后补充逻辑
        // 生成入口基本块
        let entry = function_data.dfg_mut().new_bb().basic_block(Some("%entry".into()));
        // 添加入口基本块到函数中
        function_data.layout_mut().bbs_mut().extend([entry]);
        let expr = &block.stmt.expr;
        let result = self.generate_expression(function_data, &entry, expr);
        let ret_obj = function_data.dfg_mut().new_value().ret(Some(result));
        function_data.layout_mut().bb_mut(entry).insts_mut().extend([ret_obj]);
    }

    fn generate_expression(&self, function_data: &mut FunctionData, block: &BasicBlock, expr: &function_ast::Exp) -> Value {
        self.generate_lor_statement(function_data, block, &expr.l_or_exp)
    }
}