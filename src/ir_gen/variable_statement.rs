use koopa::ir::{builder::LocalInstBuilder, BasicBlock, FunctionData};

use crate::{error_report::{Label, ProblemInfo}, function_ast::{BType, Exp, LVal, VarDecl, VarDef}, ir_gen::Symbol};
use super::IrGen;


impl IrGen {
    pub(super) fn generate_variable_statement(&self, function_data: &mut FunctionData, block: &BasicBlock, var_decl: &VarDecl) -> Result<(),()> {
        for one in &var_decl.var_def {
            self.generate_variable_definition(function_data, block, &var_decl.b_type, one)?;
        }
        Ok(())
    }

    pub(super) fn generate_variable_definition(&self, function_data: &mut FunctionData, block: &BasicBlock, b_type: &BType, var_def: &VarDef) -> Result<(), ()> {
        // 检查是否存在同样的符号
        if self.find_symbol(&var_def.ident) {
            self.problems.borrow_mut().push(ProblemInfo::error(format!("duplicate symbol '{}' found.", var_def.ident), 
                               vec![Label::primary("Note: duplicate symbol found here.", var_def.span)], None));
            return Err(());
        }
        // 分配一块栈内存
        let ty = match b_type {
            BType::Int => koopa::ir::Type::get_i32()
        };
        let alloc_instruction = function_data.dfg_mut().new_value().alloc(ty);
        // 存储符号表
        self.new_variable_symbol(var_def.ident.clone(), alloc_instruction).unwrap();
        // 如果有初始化语句，对初始化语句求值
        if let Some(var_init) = &var_def.init_val {
            let value = self.generate_expression(function_data, block, &var_init.exp)?;
            // 生成一个赋值语句
            let assign_instruction = function_data.dfg_mut().new_value().store(value, alloc_instruction);
            function_data.layout_mut().bb_mut(*block).insts_mut().extend([alloc_instruction, assign_instruction]);
        } else {
            // 添加这个分配指令
            function_data.layout_mut().bb_mut(*block).insts_mut().extend([alloc_instruction]);
        }
        Ok(())
    }

    pub(super) fn generate_assign_statement(&self, function_data: &mut FunctionData, block: &BasicBlock, l_val: &LVal, exp: &Exp) -> Result<(),()> {
        match l_val {
            LVal::Ident(symbol, span) => {
                let value = self.get_symbol(symbol);
                if let Ok(value) = value {
                    match value {
                        Symbol::Const(_) => {
                            self.problems.borrow_mut().push(ProblemInfo::error(format!("cannot assign to variable '{}' with const-qualified type 'const int'", symbol), 
                            vec![Label::primary("Note: assignment occuried here.", *span)], None));
                            Err(())
                        },
                        Symbol::Var(v) => {
                            // 生成赋值的指令
                            let exp_result = self.generate_expression(function_data, block, exp)?;
                            let assign_statement = function_data.dfg_mut().new_value().store(exp_result, v);
                            function_data.layout_mut().bb_mut(*block).insts_mut().extend([assign_statement]);
                            Ok(())
                        }
                    }
                } else {
                    self.problems.borrow_mut().push(ProblemInfo::error(format!("use of undeclared identifier '{}'", symbol), 
                                                            vec![Label::primary("Note: error occuried here.", *span)], None));
                    Err(())
                }
            }
        }
    }
}