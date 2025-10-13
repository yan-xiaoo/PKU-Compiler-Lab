use koopa::ir::{builder::{LocalInstBuilder, ValueBuilder}, BasicBlock, BinaryOp, FunctionData, Value};

use crate::function_ast::{EqExp, EqOp, LAndExp, LOrExp, RelExp, RelOp};

use super::IrGen;

impl IrGen {
    pub(crate) fn generate_lor_statement(&self, function_data: &mut FunctionData, block: &BasicBlock, expr: &LOrExp) -> Value {
        match expr {
            LOrExp::LAndExp(and_exp) => {
                self.generate_land_statement(function_data, block, and_exp)
            },
            LOrExp::CompoundLOrExp(or_exp, and_exp) => {
                let left_value = self.generate_lor_statement(function_data, block, or_exp);
                let right_value = self.generate_land_statement(function_data, block, and_exp);
                // 生成一条 OR 指令
                // 先 left_value ne 0、 right_value ne 0，再 left_value and right_value
                let zero_value = function_data.dfg_mut().new_value().integer(0);
                let left_value = function_data.dfg_mut().new_value().binary(BinaryOp::NotEq, left_value, zero_value);
                // 结果为 0 说明那就该是 0；结果非 0 就是 1。用上一个结果 Ne 0，得到最终结果。
                let right_value = function_data.dfg_mut().new_value().binary(BinaryOp::NotEq, right_value, zero_value);
                // 按位与一下
                let result = function_data.dfg_mut().new_value().binary(BinaryOp::Or, left_value, right_value);
                function_data.layout_mut().bb_mut(*block).insts_mut().extend([left_value, right_value, result]);
                result
            }
        }
    }

    pub(crate) fn generate_land_statement(&self, function_data: &mut FunctionData, block: &BasicBlock, expr: &LAndExp) -> Value {
        match expr {
            LAndExp::EqExp(eq_exp) => {
                self.generate_eq_statement(function_data, block, eq_exp)
            },
            LAndExp::CompoundLAndExp(and_exp, eq_exp) => {
                let left_value = self.generate_land_statement(function_data, block, and_exp);
                let right_value = self.generate_eq_statement(function_data, block, eq_exp);
                // 生成一条 AND 指令
                // 先 left_value ne 0、 right_value ne 0，再 left_value and right_value
                let zero_value = function_data.dfg_mut().new_value().integer(0);
                let left_value = function_data.dfg_mut().new_value().binary(BinaryOp::NotEq, left_value, zero_value);
                // 结果为 0 说明那就该是 0；结果非 0 就是 1。用上一个结果 Ne 0，得到最终结果。
                let right_value = function_data.dfg_mut().new_value().binary(BinaryOp::NotEq, right_value, zero_value);
                // 按位与一下
                let result = function_data.dfg_mut().new_value().binary(BinaryOp::And, left_value, right_value);
                function_data.layout_mut().bb_mut(*block).insts_mut().extend([left_value, right_value, result]);
                result
            }
        }
    }

    pub(crate) fn generate_eq_statement(&self, function_data: &mut FunctionData, block: &BasicBlock, expr: &EqExp) -> Value {
        match expr {
            EqExp::RelExp(rel_exp) => {
                self.generate_rel_statement(function_data, block, rel_exp)
            },
            EqExp::CompoundEqExp(eq_exp, rel_exp, eq_op) => {
                let left_value = self.generate_eq_statement(function_data, block, eq_exp);
                let right_value = self.generate_rel_statement(function_data, block, rel_exp);

                let result = match eq_op {
                    EqOp::Eq => {
                        // 相等比较指令
                        function_data.dfg_mut().new_value().binary(BinaryOp::Eq, left_value, right_value)
                    },
                    EqOp::Ne => {
                        // 不等比较指令
                        function_data.dfg_mut().new_value().binary(BinaryOp::NotEq, left_value, right_value)
                    }
                };
                function_data.layout_mut().bb_mut(*block).insts_mut().extend([result]);
                result
            }
        }
    }

    pub(crate) fn generate_rel_statement(&self, function_data: &mut FunctionData, block: &BasicBlock, expr: &RelExp) -> Value {
        match expr {
            RelExp::AddExp(add_exp) => {
                self.generate_add_statement(function_data, block, add_exp)
            },
            RelExp::CompoundRelExp(rel_exp, add_exp, rel_op) => {
                let left_value = self.generate_rel_statement(function_data, block, rel_exp);
                let right_value = self.generate_add_statement(function_data, block, add_exp);

                let result = match rel_op {
                    RelOp::Lt => {
                        // 小于比较指令
                        function_data.dfg_mut().new_value().binary(BinaryOp::Lt, left_value, right_value)
                    },
                    RelOp::Le => {
                        // 小于等于
                        function_data.dfg_mut().new_value().binary(BinaryOp::Le, left_value, right_value)
                    },
                    RelOp::Gt => {
                        // 大于
                        function_data.dfg_mut().new_value().binary(BinaryOp::Gt, left_value, right_value)
                    },
                    RelOp::Ge => {
                        // 大于等于
                        function_data.dfg_mut().new_value().binary(BinaryOp::Ge, left_value, right_value)
                    }
                };
                function_data.layout_mut().bb_mut(*block).insts_mut().extend([result]);
                result
            }
        }
    }
}