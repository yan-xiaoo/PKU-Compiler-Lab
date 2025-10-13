/// 此文件存放 AssGen 汇编类中，用于向输出字符串打印各种语句的函数
use std::fmt::Write;
use super::AssGen;

impl<'p> AssGen<'p> {
    /// 创建一个初始化寄存器的语句
    /// register: 寄存器名称
    /// value：初始化值
    pub(crate) fn init_register(&mut self, register: &str, value: i32) {
        // 不处理 x0 寄存器的赋值，因为没用
        if register != "x0" {
            writeln!(self.out, "\tli\t{},{}", register, value).unwrap();
        }
    }

    /// 创建一个清空某寄存器的语句
    /// register: 寄存器名称
    /// 实现上，通过 xor <register>, <register>, x0 来强行清空寄存器
    pub(crate) fn clear_register(&mut self, register: &str) {
        if register != "x0" {
            writeln!(self.out, "\txor\t{},{},x0", register, register).unwrap();
        }
    }


    /// 创建一个减法的指令
    /// 计算 lhr 寄存器 - rhr 寄存器的值，将其存入 result 寄存器中
    pub(crate) fn sub_inst(&mut self, result: &str, lhr: &str, rhr: &str) {
        // 对于 a=a-0 样式的指令，不生成。
        if rhr == "x0" && result == lhr {
            return;
        }
        writeln!(self.out, "\tsub\t{},{},{}", result, lhr, rhr).unwrap();
    }

    /// 创建一个加法的指令
    /// 计算 lhr 寄存器 + rhr 寄存器的值，将其存入 result 寄存器中
    pub(crate) fn add_inst(&mut self, result: &str, lhr: &str, rhr: &str) {
        writeln!(self.out, "\tadd\t{},{},{}", result, lhr, rhr).unwrap();
    }

    /// 创建一个乘法的指令
    /// 计算 lhr 寄存器 * rhr 寄存器的值，将其存入 result 寄存器中
    pub(crate) fn mul_inst(&mut self, result: &str, lhr: &str, rhr: &str) {
        writeln!(self.out, "\tmul\t{},{},{}", result, lhr, rhr).unwrap();
    }

    /// 创建一个除法的指令
    /// 计算 lhr 寄存器 / rhr 寄存器的值，将其存入 result 寄存器中
    pub(crate) fn div_inst(&mut self, result: &str, lhr: &str, rhr: &str) {
        writeln!(self.out, "\tdiv\t{},{},{}", result, lhr, rhr).unwrap();
    }
    
    /// 创建一个取余数的指令
    /// 计算 lhr 寄存器 % rhr 寄存器的值，将其存入 result 寄存器中
    pub(crate) fn mod_inst(&mut self, result: &str, lhr: &str, rhr: &str) {
        writeln!(self.out, "\trem\t{},{},{}", result, lhr, rhr).unwrap();
    }

    /// 创建一个 AND 的指令
    /// 计算 lhr 寄存器 AND rhr 寄存器的值，将其存入 result 寄存器中
    pub(crate) fn and_inst(&mut self, result: &str, lhr: &str, rhr: &str) {
        writeln!(self.out, "\tand\t{},{},{}", result, lhr, rhr).unwrap();
    }

    /// 创建一个 OR 的指令
    /// 计算 lhr 寄存器 OR rhr 寄存器的值，将其存入 result 寄存器中
    pub(crate) fn or_inst(&mut self, result: &str, lhr: &str, rhr: &str) {
        writeln!(self.out, "\tor\t{},{},{}", result, lhr, rhr).unwrap();
    }

    /// 创建一个 neq 0 比较指令
    /// 如果 register 的值不是 0，存储 1 到 register 寄存器中；否则，存储 0 到 register 中。
    pub(crate) fn eq0_inst(&mut self, register: &str) {
        writeln!(self.out, "\tseqz\t{},{}", register, register).unwrap();
    }

    /// 创建一个 eq 0 比较指令
    /// 如果 register 的值是 0，存储 1 到 register 寄存器中；否则，存储 1 到 register 中。
    pub(crate) fn neq0_inst(&mut self, register: &str) {
        writeln!(self.out, "\tsnez\t{},{}", register, register).unwrap();
    }

    /// 创建一个小于比较指令
    /// 如果 lhr < rhr，写入 1 到 result 中，否则写入 0
    pub(crate) fn lt_inst(&mut self, result: &str, lhr: &str, rhr: &str)  {
        writeln!(self.out, "\tslt\t{},{},{}", result, lhr, rhr).unwrap();
    }

    /// 创建一个大于比较指令
    /// 如果 lhr > rhr，写入 1 到 result 中，否则写入 0
    pub(crate) fn gt_inst(&mut self, result: &str, lhr: &str, rhr: &str)  {
        writeln!(self.out, "\tsgt\t{},{},{}", result, lhr, rhr).unwrap();
    }

    /// 创建一个小于等于比较指令
    /// 如果 lhr <= rhr，写入 1 到 result 中，否则写入 0
    pub(crate) fn le_inst(&mut self, result: &str, lhr: &str, rhr: &str)  {
        // 写一个大于指令
        self.gt_inst(result, lhr, rhr);
        // 取反
        writeln!(self.out, "\tseqz\t{},{}", result, result).unwrap();
    }

    /// 创建一个大于等于比较指令
    /// 如果 lhr > rhr，写入 1 到 result 中，否则写入 0
    pub(crate) fn ge_inst(&mut self, result: &str, lhr: &str, rhr: &str)  {
        self.lt_inst(result, lhr, rhr);
        // 取反
        writeln!(self.out, "\tseqz\t{},{}", result, result).unwrap();
    }

    /// 创建一个返回指令
    /// value: 返回值
    pub(crate) fn return_value_inst(&mut self, value: Option<i32>) {
        if let Some(i) = value {
            self.init_register("a0", i);
        }
        writeln!(self.out, "\tret").unwrap();
    }

    /// 创建一个返回指令
    /// register: 返回值目前存储在哪个寄存器中
    pub(crate) fn return_register_inst(&mut self, register: &str) {
        writeln!(self.out, "\tmv\ta0,{}", register).unwrap();
        writeln!(self.out, "\tret").unwrap();
    }
}