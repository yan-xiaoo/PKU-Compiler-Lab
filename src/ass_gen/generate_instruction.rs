/// 此文件存放 AssGen 汇编类中，用于向输出字符串打印各种语句的函数
use super::AssGen;

impl<'p> AssGen<'p> {
    /// 创建一个初始化寄存器的语句
    /// register: 寄存器名称
    /// value：初始化值
    pub(super) fn init_register(&mut self, register: &str, value: i32) {
        let s = self.init_register_str(register, value);
        if !s.is_empty() {
            self.out.push_str(&s);
        }
    }

    /// String 版本：创建一个初始化寄存器的语句
    pub(super) fn init_register_str(&self, register: &str, value: i32) -> String {
        if register == "x0" {
            String::new()
        } else {
            format!("\tli\t{},{}\n", register, value)
        }
    }

    /// 创建一个清空某寄存器的语句
    /// register: 寄存器名称
    /// 实现上，通过 xor <register>, <register>, x0 来强行清空寄存器
    pub(super) fn clear_register(&mut self, register: &str) {
        let s = self.clear_register_str(register);
        if !s.is_empty() {
            self.out.push_str(&s);
        }
    }

    /// String 版本：创建一个清空某寄存器的语句
    pub(super) fn clear_register_str(&self, register: &str) -> String {
        if register == "x0" {
            String::new()
        } else {
            format!("\txor\t{},{},x0\n", register, register)
        }
    }


    /// 创建一个减法的指令
    /// 计算 lhr 寄存器 - rhr 寄存器的值，将其存入 result 寄存器中
    pub(super) fn sub_inst(&mut self, result: &str, lhr: &str, rhr: &str) {
        let s = self.sub_inst_str(result, lhr, rhr);
        if !s.is_empty() {
            self.out.push_str(&s);
        }
    }

    /// String 版本：创建一个减法的指令
    pub(super) fn sub_inst_str(&self, result: &str, lhr: &str, rhr: &str) -> String {
        if rhr == "x0" && result == lhr {
            String::new()
        } else {
            format!("\tsub\t{},{},{}\n", result, lhr, rhr)
        }
    }

    /// 创建一个加法的指令
    /// 计算 lhr 寄存器 + rhr 寄存器的值，将其存入 result 寄存器中
    pub(super) fn add_inst(&mut self, result: &str, lhr: &str, rhr: &str) {
        let s = self.add_inst_str(result, lhr, rhr);
        self.out.push_str(&s);
    }

    /// String 版本：创建一个加法的指令
    pub(super) fn add_inst_str(&self, result: &str, lhr: &str, rhr: &str) -> String {
        format!("\tadd\t{},{},{}\n", result, lhr, rhr)
    }

    /// 创建一个乘法的指令
    /// 计算 lhr 寄存器 * rhr 寄存器的值，将其存入 result 寄存器中
    pub(super) fn mul_inst(&mut self, result: &str, lhr: &str, rhr: &str) {
        let s = self.mul_inst_str(result, lhr, rhr);
        self.out.push_str(&s);
    }

    /// String 版本：创建一个乘法的指令
    pub(super) fn mul_inst_str(&self, result: &str, lhr: &str, rhr: &str) -> String {
        format!("\tmul\t{},{},{}\n", result, lhr, rhr)
    }

    /// 创建一个除法的指令
    /// 计算 lhr 寄存器 / rhr 寄存器的值，将其存入 result 寄存器中
    pub(super) fn div_inst(&mut self, result: &str, lhr: &str, rhr: &str) {
        let s = self.div_inst_str(result, lhr, rhr);
        self.out.push_str(&s);
    }

    /// String 版本：创建一个除法的指令
    pub(super) fn div_inst_str(&self, result: &str, lhr: &str, rhr: &str) -> String {
        format!("\tdiv\t{},{},{}\n", result, lhr, rhr)
    }
    
    /// 创建一个取余数的指令
    /// 计算 lhr 寄存器 % rhr 寄存器的值，将其存入 result 寄存器中
    pub(super) fn mod_inst(&mut self, result: &str, lhr: &str, rhr: &str) {
        let s = self.mod_inst_str(result, lhr, rhr);
        self.out.push_str(&s);
    }

    /// String 版本：创建一个取余数的指令
    pub(super) fn mod_inst_str(&self, result: &str, lhr: &str, rhr: &str) -> String {
        format!("\trem\t{},{},{}\n", result, lhr, rhr)
    }

    /// 创建一个 AND 的指令
    /// 计算 lhr 寄存器 AND rhr 寄存器的值，将其存入 result 寄存器中
    pub(super) fn and_inst(&mut self, result: &str, lhr: &str, rhr: &str) {
        let s = self.and_inst_str(result, lhr, rhr);
        self.out.push_str(&s);
    }

    /// String 版本：创建一个 AND 的指令
    pub(super) fn and_inst_str(&self, result: &str, lhr: &str, rhr: &str) -> String {
        format!("\tand\t{},{},{}\n", result, lhr, rhr)
    }

    /// 创建一个 OR 的指令
    /// 计算 lhr 寄存器 OR rhr 寄存器的值，将其存入 result 寄存器中
    pub(super) fn or_inst(&mut self, result: &str, lhr: &str, rhr: &str) {
        let s = self.or_inst_str(result, lhr, rhr);
        self.out.push_str(&s);
    }

    /// String 版本：创建一个 OR 的指令
    pub(super) fn or_inst_str(&self, result: &str, lhr: &str, rhr: &str) -> String {
        format!("\tor\t{},{},{}\n", result, lhr, rhr)
    }

    /// 创建一个 neq 0 比较指令
    /// 如果 register 的值不是 0，存储 1 到 register 寄存器中；否则，存储 0 到 register 中。
    pub(super) fn eq0_inst(&mut self, register: &str) {
        let s = self.eq0_inst_str(register);
        self.out.push_str(&s);
    }

    /// String 版本：创建一个 eq 0 比较指令
    pub(super) fn eq0_inst_str(&self, register: &str) -> String {
        format!("\tseqz\t{},{}\n", register, register)
    }

    /// 创建一个 eq 0 比较指令
    /// 如果 register 的值是 0，存储 1 到 register 寄存器中；否则，存储 1 到 register 中。
    pub(super) fn neq0_inst(&mut self, register: &str) {
        let s = self.neq0_inst_str(register);
        self.out.push_str(&s);
    }

    /// String 版本：创建一个 neq 0 比较指令
    pub(super) fn neq0_inst_str(&self, register: &str) -> String {
        format!("\tsnez\t{},{}\n", register, register)
    }

    /// 创建一个小于比较指令
    /// 如果 lhr < rhr，写入 1 到 result 中，否则写入 0
    pub(super) fn lt_inst(&mut self, result: &str, lhr: &str, rhr: &str)  {
        let s = self.lt_inst_str(result, lhr, rhr);
        self.out.push_str(&s);
    }

    /// String 版本：创建一个小于比较指令
    pub(super) fn lt_inst_str(&self, result: &str, lhr: &str, rhr: &str) -> String  {
        format!("\tslt\t{},{},{}\n", result, lhr, rhr)
    }

    /// 创建一个大于比较指令
    /// 如果 lhr > rhr，写入 1 到 result 中，否则写入 0
    pub(super) fn gt_inst(&mut self, result: &str, lhr: &str, rhr: &str)  {
        let s = self.gt_inst_str(result, lhr, rhr);
        self.out.push_str(&s);
    }

    /// String 版本：创建一个大于比较指令
    pub(super) fn gt_inst_str(&self, result: &str, lhr: &str, rhr: &str) -> String  {
        format!("\tsgt\t{},{},{}\n", result, lhr, rhr)
    }

    /// 创建一个小于等于比较指令
    /// 如果 lhr <= rhr，写入 1 到 result 中，否则写入 0
    pub(super) fn le_inst(&mut self, result: &str, lhr: &str, rhr: &str)  {
        let s = self.le_inst_str(result, lhr, rhr);
        self.out.push_str(&s);
    }

    /// String 版本：创建一个小于等于比较指令
    pub(super) fn le_inst_str(&self, result: &str, lhr: &str, rhr: &str)  -> String {
        let mut s = self.gt_inst_str(result, lhr, rhr);
        s.push_str(&format!("\tseqz\t{},{}\n", result, result));
        s
    }

    /// 创建一个大于等于比较指令
    /// 如果 lhr > rhr，写入 1 到 result 中，否则写入 0
    pub(super) fn ge_inst(&mut self, result: &str, lhr: &str, rhr: &str)  {
        let s = self.ge_inst_str(result, lhr, rhr);
        self.out.push_str(&s);
    }

    /// String 版本：创建一个大于等于比较指令
    pub(super) fn ge_inst_str(&self, result: &str, lhr: &str, rhr: &str)  -> String {
        let mut s = self.lt_inst_str(result, lhr, rhr);
        s.push_str(&format!("\tseqz\t{},{}\n", result, result));
        s
    }

    /// 创建一个返回指令
    /// value: 返回值
    pub(super) fn return_value_inst(&mut self, value: Option<i32>) {
        let s = self.return_value_inst_str(value);
        self.out.push_str(&s);
    }

    /// String 版本：创建一个返回指令（可选立即数）
    pub(super) fn return_value_inst_str(&self, value: Option<i32>) -> String {
        let mut s = String::new();
        if let Some(i) = value {
            s.push_str(&self.init_register_str("a0", i));
        }
        s.push_str("\tret\n");
        s
    }

    /// 创建一个返回指令
    /// register: 返回值目前存储在哪个寄存器中
    pub(super) fn return_register_inst(&mut self, register: &str) {
        let s = self.return_register_inst_str(register);
        self.out.push_str(&s);
    }

    /// String 版本：创建一个返回指令（寄存器）
    pub(super) fn return_register_inst_str(&self, register: &str) -> String {
        format!("\tmv\ta0,{}\n", register)
    }

    pub(super) fn return_stack_inst_str(&self, stack: i32) -> String {
        format!("\tlw\ta0,{}(sp)\n", stack)
    }

    /// 创建一个加载（lw）指令
    /// result: 加载后的内容放在哪个寄存器中
    /// offset: 加载栈地址相对 sp 指针的偏移量。此编译器的栈使用规定为：
    /// 函数栈从高地址向低地址生长（RISC-V 要求），在每个函数栈内部，变量从低向高生长
    /// 因为此规定，offset 一定大于 0（不然你就指到其他函数或者鬼知道哪里的栈去了），小于 0 的 offset 会直接引发崩溃。
    /// 这里不用 u32 而使用 i32 是为了和 RISC-V 指令集采用的有符号整数对应。用 u32（32位无符号）的话，可能会超过 RISC-V 的最大立即数（32位有符号）限制
    /// 目前每个变量大小都是 4 字节，没有函数参数
    pub(super) fn load_inst(&mut self, result: &str, offset: i32) {
        let s = self.load_inst_str(result, offset);
        self.out.push_str(&s);
    }

    /// String 版本：创建一个加载（lw）指令
    pub(super) fn load_inst_str(&self, result: &str, offset: i32) -> String {
        if offset < 0 {
            panic!("Offset 必须大于等于 0，得到 {}", offset)
        }
        if offset > 2047 {
            let mut s = String::new();
            s.push_str(&self.init_register_str("t0", offset));
            s.push_str(&self.add_inst_str("t0", "sp", "t0"));
            s.push_str(&format!("\tlw\t{}, 0(t0)\n", result));
            s
        } else {
            format!("\tlw\t{},{}(sp)\n", result, offset)
        }
    }

    /// 创建一个存储（sw）指令
    /// result: 待存储的内容放在哪个寄存器中
    /// offset: 存储栈地址相对 sp 指针的偏移量
    pub(super) fn store_inst(&mut self, source: &str, offset: i32) {
        let s = self.store_inst_str(source, offset);
        self.out.push_str(&s);
    }

    /// String 版本：创建一个存储（sw）指令
    pub(super) fn store_inst_str(&self, source: &str, offset: i32) -> String {
        if offset < 0 {
            panic!("Offset 必须大于等于 0，得到 {}", offset)
        }
        if offset > 2047 {
            let mut s = String::new();
            s.push_str(&self.init_register_str("t0", offset));
            s.push_str(&self.add_inst_str("t0", "sp", "t0"));
            s.push_str(&format!("\tsw\t{}, 0(t0)\n", source));
            s
        } else {
            format!("\tsw\t{},{}(sp)\n", source, offset)
        }
    }

    /// 移动栈指针 sp 的指令
    /// 用于在函数开始前和结束后修改栈边界
    pub(super) fn move_sp_inst(&mut self, value: i32) {
        let s = self.move_sp_inst_str(value);
        self.out.push_str(&s);
    }

    /// String 版本：移动栈指针 sp 的指令
    pub(super) fn move_sp_inst_str(&self, value: i32) -> String {
        if value == 0 {
            String::new()
        } else if !(-2048..=2047).contains(&value) {
            let mut s = String::new();
            s.push_str(&self.init_register_str("t0", value));
            s.push_str(&self.add_inst_str("sp", "sp", "t0"));
            s
        } else {
            format!("\taddi\tsp,sp,{}\n", value)
        }
    }

    pub(super) fn move_register_inst_str(&self, dest: &str, src: &str) -> String {
        if src != dest {
            format!("\tmv\t{},{}\n", dest, src)
        } else {
            String::new()
        }
    }
}