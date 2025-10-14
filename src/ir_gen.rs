use std::{cell::RefCell, collections::HashMap};
use crate::{error_report::{Label, ProblemInfo}, function_ast::{self, BlockItem, CompUnit, FuncDef, FuncType}};

use koopa::ir::{builder::{BasicBlockBuilder, LocalInstBuilder, ValueBuilder}, BasicBlock, FunctionData, Program, Type, Value};

// 将 ir_gen 子模块拆分到 src/ir_gen/ 目录下的多个文件
// 这里显式声明子模块：会加载 "src/ir_gen/unary_statement.rs"
mod unary_statement;
mod arithmetic_statement;
mod logic_statement;
mod const_statement;

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
    /// 常量符号表
    symbols: RefCell<HashMap<String, i32>>,
    /// 编译错误信息
    problems: RefCell<Vec<ProblemInfo>>
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
            symbols: RefCell::new(HashMap::new()),
            problems: RefCell::new(Vec::new())
        }
    }

    pub fn get_problems(&self) -> Vec<ProblemInfo> {
        self.problems.take()
    }

    /// 创建一个新的常量符号（并赋值）
    /// 如果常量符号已经存在，则返回错误
    fn new_symbol(&self, name: String, value: i32) -> Result<(), String> {
        match self.symbols.borrow_mut().entry(name) {
            std::collections::hash_map::Entry::Occupied(_) => Err("已经存在符号".to_string()),
            std::collections::hash_map::Entry::Vacant(entry) => {
                entry.insert(value);
                Ok(())
            }
        }
    }

    /// 查询符号表中是否存在某个常量符号
    #[allow(dead_code)]
    fn find_symbol(&self, name: &str) -> bool {
        self.symbols.borrow().contains_key(name)
    }

    /// 从符号表中尝试获得一个符号。获得不存在的符号会返回一个错误。
    fn get_symbol(&self, name: &str) -> Result<i32, String> {
        if let Some(data) = self.symbols.borrow().get(name) {
            Ok(*data)
        } else {
            Err(format!("符号不存在：{}", name))
        }
    }

    pub fn generate_koopa_ir(&mut self, parsed_unit: CompUnit) -> Option<Program> {
        if self.generate_function(parsed_unit.func_def).is_ok() {
            // 返回程序
            Some(self.program.take())
        } else {
            // 出错了，返回 None，外层自己提取错误
            None
        }
    }

    fn generate_function(&mut self, function: FuncDef) -> Result<(), ()> {
        let koopa_type = match function.func_type {
            FuncType::Int => Type::get_i32(),
            FuncType::Void => Type::get_unit(),
        };
        let function_data = FunctionData::new(format!("@{}", function.ident), Vec::new(), koopa_type);
        // 必须先添加 function_data 到程序中
        let mut binding = self.program.borrow_mut();
        let func = binding.new_func(function_data);
        let function_data= binding.func_mut(func);
        // 后续在块内容变复杂之后补充逻辑
        // 生成入口基本块
        let entry = function_data.dfg_mut().new_bb().basic_block(Some("%entry".into()));
        // 添加入口基本块到函数中
        function_data.layout_mut().bbs_mut().extend([entry]);
        // 有返回值的情况
        // 生成块失败则立刻返回
        if let Some(value) = self.generate_block(function_data, &entry, &function.block)? {
            let ret_obj = function_data.dfg_mut().new_value().ret(Some(value));
            function_data.layout_mut().bb_mut(entry).insts_mut().extend([ret_obj]);
        } else {
            // 没返回值的情况，根据返回类型判定 warning
            // 判断 main 是否为 int 返回值
            if function.ident == "main" && function.func_type == FuncType::Int {
                self.problems.borrow_mut().push(ProblemInfo::warning("'main' function doesn't return an integer.", 
                            vec![Label::primary("Note: 'main' function is defined here.", function.span)], None));
            }
            // 判断返回值为非 void 的函数是否没有返回内容
            if function.func_type != FuncType::Void {
                self.problems.borrow_mut().push(ProblemInfo::warning("non-void function doesn't return a value.", 
                                   vec![Label::primary("Note: function defined here.", function.span)], None));
                // 补充一个 0 返回值
                let zero = function_data.dfg_mut().new_value().integer(0);
                let ret_obj = function_data.dfg_mut().new_value().ret(Some(zero));
                function_data.layout_mut().bb_mut(entry).insts_mut().extend([ret_obj]);
            }
        }
        // 成功
        Ok(())
    }

    fn generate_block(&self, function_data: &mut FunctionData, ir_block: &BasicBlock, block: &function_ast::Block) -> Result<Option<Value>, ()> {
        let mut result = Ok(None);
        for item in &block.block_items {
            match item {
                BlockItem::Decl(decl) => {
                    let r = self.generate_declaration(function_data, ir_block, decl);
                    if r.is_ok() {
                        result = Ok(None);
                    } else {
                        return Err(())
                    }
                },
                BlockItem::Stmt(stmt) => {
                    let r = self.generate_expression(function_data, ir_block, &stmt.expr);
                    if let Ok(d) = r {
                        result = Ok(Some(d));
                    } else {
                        return Err(())
                    }
                }
            };
        }
        result
    }

    #[allow(unused_variables)]
    fn generate_declaration(&self, function_data: &mut FunctionData, block: &BasicBlock, declaration: &function_ast::Decl) -> Result<(),()> {
        self.generate_const_statement(&declaration.const_decl)
    }

    fn generate_expression(&self, function_data: &mut FunctionData, block: &BasicBlock, expr: &function_ast::Exp) -> Result<Value, ()> {
        self.generate_lor_statement(function_data, block, &expr.l_or_exp)
    }
}