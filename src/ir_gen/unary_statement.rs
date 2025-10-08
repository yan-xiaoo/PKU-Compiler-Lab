use koopa::ir::{builder::{LocalInstBuilder, ValueBuilder}, BasicBlock, BinaryOp, FunctionData, Value};

use crate::function_ast::{PrimaryExp, UnaryExp, UnaryOp};

use super::IrGen;

// 这里放与一元表达式相关的 IR 降低/生成逻辑
impl IrGen {
    /// 处理一个一元表达式
    /// 一元表达式最终处理完的结果会存放在返回的 Value 中
    pub(crate) fn generate_unary_statement(&self, function_data: &mut FunctionData, block: &BasicBlock, unary_statement: & UnaryExp) -> Value {
		match unary_statement {
            UnaryExp::PrimaryExp(primary) => {
                // 生成内部表达式的值即可
                self.generate_primary_statement(function_data, block, primary)
            },
            UnaryExp::CompoundUnaryExp(op, unary_exp) => {
                // 先生成内部语句
                let value = self.generate_unary_statement(function_data, block, unary_exp);
                match op {
                    UnaryOp::Plus => {
                        // 直接返回内部的一元表达式，因为 +n=n,即不需要生成指令
                        value
                    },
                    UnaryOp::Minus => {
                        // 生成一条减法指令
                        let zero_value = function_data.dfg_mut().new_value().integer(0);
                        // 再生成一条减法语句
                        // 此减法指令的返回值就是本函数的返回值
                        let value = function_data.dfg_mut().new_value().binary(BinaryOp::Sub, zero_value, value);
                        function_data.layout_mut().bb_mut(*block).insts_mut().extend([value]);
                        value
                    },
                    UnaryOp::Not => {
                        // 生成一条 not 指令
                        // 即输入非零结果就是 0，输入为 0 结果就是 1
                        let zero_value = function_data.dfg_mut().new_value().integer(0);
                        // 生成内部语句
                        let value = function_data.dfg_mut().new_value().binary(BinaryOp::Eq, value, zero_value);
                        function_data.layout_mut().bb_mut(*block).insts_mut().extend([value]);
                        value
                    }
                }
            }
        }
	}

    pub(crate) fn generate_primary_statement(&self, function_data: &mut FunctionData, block: &BasicBlock, primary_statement: &PrimaryExp) -> Value {
        match primary_statement {
            PrimaryExp::Exp(exp) => {
                // 如果此表达式内部包裹了表达式，则去生成内部表达式
                self.generate_expression(function_data, block, exp)
            },
            PrimaryExp::Number(i) => {
                // 如果此表达式的值是一个整数，直接将其放入 IR 并返回就可以了
                // 注意常数是不需要加入 function 的
                function_data.dfg_mut().new_value().integer(*i)
            }
        }
    }
}