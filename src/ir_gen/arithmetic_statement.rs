use koopa::ir::{builder::LocalInstBuilder, BasicBlock, BinaryOp, FunctionData, Value};

use crate::function_ast::{self, AddExp, AddOp, MulExp, MulOp};

// 二元算术表达式语句的 IR 生成
use super::IrGen;


impl IrGen {
    pub(super) fn generate_add_statement(&self, function_data: & mut FunctionData, block: &BasicBlock, expr: &function_ast::AddExp) -> Result<Value, ()> {
        match expr {
            AddExp::MulExp(mul_exp) => {
                self.generate_mul_statement(function_data, block, mul_exp)
            },
            AddExp::CompoundAddExp(add_exp, mul_exp, add_op) => {
                // 求值这两个表达式
                let left_value = self.generate_add_statement(function_data, block, add_exp)?;
                let right_value = self.generate_mul_statement(function_data, block, mul_exp)?;
                // 根据运算符号生成运算语句
                match add_op {
                    AddOp::Plus => {
                        // 插入一条加法指令
                        let inst = function_data.dfg_mut().new_value().binary(BinaryOp::Add, left_value, right_value);
                        function_data.layout_mut().bb_mut(*block).insts_mut().extend([inst]);
                        Ok(inst)
                    },
                    AddOp::Minus => {
                        // 插入一条减法指令
                        let inst = function_data.dfg_mut().new_value().binary(BinaryOp::Sub, left_value, right_value);
                        function_data.layout_mut().bb_mut(*block).insts_mut().extend([inst]);
                        Ok(inst)
                    }
                }
            }
        }
    }

    pub(super) fn generate_mul_statement(&self, function_data: & mut FunctionData, block: &BasicBlock, expr: &MulExp) -> Result<Value, ()> {
        match expr {
            MulExp::UnaryExp(unary_exp) => {
                self.generate_unary_statement(function_data, block, unary_exp)
            },
            MulExp::CompoundMulExp(mul_exp, unary_exp, mul_op) => {
                let left_value = self.generate_mul_statement(function_data, block, mul_exp)?;
                let right_value = self.generate_unary_statement(function_data, block, unary_exp)?;
                match mul_op {
                    MulOp::Mul => {
                        // 插入乘法指令
                        let inst = function_data.dfg_mut().new_value().binary(BinaryOp::Mul, left_value, right_value);
                        function_data.layout_mut().bb_mut(*block).insts_mut().extend([inst]);
                        Ok(inst)
                    },
                    MulOp::Div => {
                        let inst = function_data.dfg_mut().new_value().binary(BinaryOp::Div, left_value, right_value);
                        function_data.layout_mut().bb_mut(*block).insts_mut().extend([inst]);
                        Ok(inst)
                    },
                    MulOp::Mod => {
                        let inst = function_data.dfg_mut().new_value().binary(BinaryOp::Mod, left_value, right_value);
                        function_data.layout_mut().bb_mut(*block).insts_mut().extend([inst]);
                        Ok(inst)
                    }
                }
            }
        }
    }
}