use std::collections::HashMap;
use std::fmt::Write;

use koopa::ir::{BinaryOp, FunctionData, Program, Value, ValueKind};
use koopa::ir::dfg::DataFlowGraph;

pub mod generate_instruction;

/// 符号表中存放的符号，可能是一个寄存器或者一个栈地址偏移
#[derive(Debug, Clone)]
pub enum Symbol {
    Register(String),
    Stack(i32)
}

/// 表示一个函数的汇编生成缓冲区
/// 这是为了方便统计栈空间设置的
#[derive(Debug)]
struct FuncAsm {
    name: String,
    prologue: Vec<String>,
    body: Vec<String>,
    epilogue: Vec<String>,
    stack_size: i32,
}

/// 集中式代码生成上下文
/// 持有对 Program 的只读引用，以及内部输出缓冲区。
pub struct AssGen<'p> {
    prog: &'p Program,
    out: String,
    // 后续可扩展：当前函数信息/寄存器分配/块标签映射/目标选项等
    // 简易符号表：koopa::ir::Value 映射到一个 
    symbol_table: HashMap<Value, Symbol>,
    // 寄存器分配状态：寄存器名称-是否占用
    register_status: HashMap<String, bool>,
    current_func: Option<FuncAsm>,
    // 保留寄存器的分配状态（t0, t1, t2）
    reserved_status: HashMap<String, bool>
}

impl<'p> AssGen<'p> {
    pub fn new(prog: &'p Program) -> Self {
        // t0 寄存器被保留，在立即数大于 12 位范围时用于中继。严禁在 t0 寄存器中输入任何内容，因为其可能随时被覆盖。
        // t1、t2、t3 寄存器同样被保留，主要是用于做二元运算，如果寄存器空间不足就必须留下以加载栈空间的内容
        // 与 t0 不同的是，可以通过方法来暂时申请这两个寄存器，利用其生成指令，但二元运算结束后必须释放。
        let register_info = vec![
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
        let reserved_register_info = vec![
            (String::from("t0"), false),
            (String::from("t1"), false),
            (String::from("t2"), false),
            (String::from("t3"), false)
        ];
        Self {
            prog,
            out: String::new(),
            symbol_table: HashMap::new(),
            register_status: register_info.into_iter().collect(),
            current_func: None,
            reserved_status: reserved_register_info.into_iter().collect()
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
            self.enter_function(self.prog.func(func).name());
            let f = self.prog.func(func);
            self.generate_function(f);
            self.leave_function();
        }
    }

    /// 每次翻译一个函数前，需要存储函数对象，做一些准备工作
    fn enter_function(&mut self, name: &str) {
        self.current_func = Some(FuncAsm {
            name: name.to_string(),
            prologue: Vec::new(),
            body: Vec::new(),
            epilogue: Vec::new(),
            stack_size: 0
        })
    }

    /// 每次翻译函数后，需要计算函数栈空间，将对象真正写为指令
    fn leave_function(&mut self) {
        // 清空 self.current_func
        if let Some(mut function) = self.current_func.take() {
            let aligned_stack = AssGen::remap_to_16(function.stack_size);
            // 先不管函数参数之类的玩意，因为没到这个阶段
            // 加入栈指针移动的指令
            // RISC-V 栈向下生长，所以是负的
            function.prologue.push(self.move_sp_inst_str(-aligned_stack));
            // 栈指针移动回来
            function.epilogue.push(self.move_sp_inst_str(aligned_stack));

            // 生成拼接内容
            // 函数名
            writeln!(self.out, "{}:", self.strip_symbol_prefix(&function.name)).unwrap();
            // 函数前置
            for l in function.prologue { write!(self.out, "{}", l).unwrap(); }
            // 函数内容
            for l in function.body { write!(self.out, "{}", l).unwrap(); }
            // 函数后置
            for l in function.epilogue { write!(self.out, "{}", l).unwrap(); }
            // 返回指令
            writeln!(self.out, "\tret\n").unwrap();
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
                // 在符号表中记录这次分配
                self.symbol_table.insert(*value, Symbol::Register(name.clone()));
                return Some(name.clone());
            }
        }
        None
    }

    /// 创建一块新的栈空间，并且分配空间给某个变量
    /// 返回内容为该空间在函数内的栈偏移值
    /// 如果当前不在某个函数环境下，直接崩溃。
    fn new_stack_symbol(&mut self, value: &Value) -> i32 {
        if let Some(function) = &mut self.current_func {
            // 当前的栈指针为 function.stack_size
            // 直接将当前位置开始分配给变量
            let current = function.stack_size;
            self.symbol_table.insert(*value, Symbol::Stack(current));
            function.stack_size += 4;
            current
        } else {
            panic!("错误：在非函数环境下尝试分配栈空间");
        }
    }

    /// 尝试从符号表中寻找一个值已经被分配的符号。如果失败，则分配一个新的寄存器或者栈空间并返回。
    /// 
    fn find_or_allocate_symbol(&mut self, value: &Value) -> Symbol {
        if let Some(result) = self.symbol_table.get(value) {
            result.clone()
        } else {
            match self.fresh_register(value) {
                // 优先分配寄存器
                Some(s) => Symbol::Register(s),
                // 寄存器用完了分配栈
                None => Symbol::Stack(self.new_stack_symbol(value))
            } 
        }
    }

    /// find_or_allocate_symbol 的简化版本
    /// 此函数尝试为某个 value 分配或者获得寄存器，并且在无法分配时返回栈变量
    /// check_x0：为真时，如果输入为 Integer 的 0，那么为其分配 x0 寄存器。
    fn get_symbol_for_value(&mut self, dfg: &DataFlowGraph, value: &Value, check_x0: bool) -> Symbol {
        let data = dfg.value(*value);
        if let ValueKind::Integer(i) = data.kind() {
            if i.value() == 0 && check_x0 {
                Symbol::Register(String::from("x0"))
            } else {
                self.find_or_allocate_symbol(value)
            }
        } else {
            self.find_or_allocate_symbol(value)
        }
    }

    /// 移除一个寄存器，将其设为未分配
    /// 如果输入的寄存器不存在或者没有被使用或者成功释放，返回 Ok
    /// 如果输入的寄存器被一个非立即数指令使用，返回错误。
    fn remove_register(&mut self, dfg: &DataFlowGraph, register: &str) -> Result<(), String> {
        let data: Option<&bool> = self.register_status.get(register);
        if let Some(status) = data
            && *status {
                for (key, value) in self.symbol_table.iter() {
                    if let Symbol::Register(value) = value && value == register {
                        if let ValueKind::Integer(_) = dfg.value(*key).kind() {
                            // 一会移除
                        } else {
                            return Err(format!("输入的寄存器不是不是存储常量的寄存器: {}", register));
                        }
                    }
                }
                self.symbol_table.retain(|&_, v| {
                    // 移除逻辑（其实是保留逻辑，闭包的返回值决定是否保留）
                    // 如果遍历到寄存器，那么检查是不是和输入寄存器一样，是的话就删
                    // 如果遍历到栈对象，不管，直接返回“保留”
                    if let Symbol::Register(v) = v {
                        v != register
                    } else {
                        true
                    }
                });
                // 没有问题，取消注册
                self.register_status.insert(register.to_string(), false);
            }
        Ok(())
    }

    /// 将栈空间大小对齐（增加）到最近的 16 的倍数。
    fn remap_to_16(mut a: i32) -> i32 {
        while a % 16 != 0 {
            a += 1;
        }
        a
    }

    /// 获得二元运算用的保留寄存器 t1 和 t2
    /// 此函数的使用场景为：二元运算的两个操作数都在栈空间，因此需要加载到寄存器来计算
    /// 必须在 current_func 上下文不为空时调用此函数
    /// 此函数会生成加载指令，返回寄存器的名称。
    fn get_reserved_register(&mut self, stack: i32) -> String {
        let mut empty = None;
        for (name, used) in self.reserved_status.iter_mut() {
            if !*used {
                *used = true; // 标记为占用
                empty = Some(name.clone());
                break;
            }
        }
        match empty {
            Some(s) => {
                let load_inst = self.load_inst_str(&s, stack);
                if let Some(function) = &mut self.current_func {
                    function.body.push(load_inst);
                    s
                } else {
                    panic!("在非函数环境下尝试使用栈地址")
                }
            },
            None => panic!("预留寄存器已满")
        }
    }

    fn get_reserved_register_without_load(&mut self) -> String {
        for (name, used) in self.reserved_status.iter_mut() {
            if !*used {
                *used = true; // 标记为占用
                return name.clone();
            }
        }
        panic!("预留寄存器已满");
    }

    /// 释放二元运算使用的保留寄存器 t1 和 t2
    fn remove_reserved_register(&mut self, register: &str) {
        if self.reserved_status.contains_key(register) {
            self.reserved_status.insert(register.to_string(), false);
        }
    }

    /// 生成单个函数的汇编
    fn generate_function(&mut self, f: &FunctionData) {
        let dfg = f.dfg();

        for (&_bb, bb_node) in f.layout().bbs() {
            // 如需基本块标签，可在此处输出
            for &inst in bb_node.insts().keys() {
                self.generate_instruction(dfg, inst);
            }
        }
    }

    /// 添加一条指令到当前函数中
    /// 如果当前没有函数环境，则崩溃
    fn add_inst_to_function(&mut self, s: String) {
        if let Some(function) = &mut self.current_func {
            function.body.push(s);
        } else {
            panic!("当前不是函数环境")
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
                            self.add_inst_to_function(self.return_value_inst_str(Some(i.value())));
                        },
                        ValueKind::Undef(_) => {
                            self.add_inst_to_function(self.return_value_inst_str(None));
                        },
                        _ => {
                            let value = self.symbol_table.get(&return_value);
                            match value {
                                Some(value) => {
                                    match value {
                                        Symbol::Register(register) =>  self.add_inst_to_function(self.return_register_inst_str(&register.clone())),
                                        Symbol::Stack(stack) => self.add_inst_to_function(self.return_stack_inst_str(*stack)),
                                    }
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
                let result_symbol = self.find_or_allocate_symbol(&inst);
                // 操作数表达式本身
                let left_exp = dfg.value(binary.lhs());
                let right_exp = dfg.value(binary.rhs());
                // 操作数的存储位置
                // 如果操作数是 0，省略寄存器分配
                let left_symbol = self.get_symbol_for_value(dfg, &binary.lhs(), true);
                let right_symbol = self.get_symbol_for_value(dfg, &binary.rhs(), true);

                // 给结果分配一个寄存器（如果本来就是寄存器，则不分配）
                let result_register = match &result_symbol {
                    Symbol::Register(r) => r.clone(),
                    Symbol::Stack(s) => self.get_reserved_register(*s)
                };

                // 如果操作数是栈空间，需要加载
                // 如果是寄存器，就不用了
                let left_register = match &left_symbol {
                    Symbol::Register(r) => r.clone(),
                    Symbol::Stack(s) => self.get_reserved_register(*s)
                };
                let right_register = match &right_symbol {
                    Symbol::Register(r) => r.clone(),
                    Symbol::Stack(s) => self.get_reserved_register(*s)
                };
                
                // 如果操作数表达式是立即数，那么我们就生成立即数赋值语句 li rs,?
                if let ValueKind::Integer(i) = left_exp.kind() {
                    self.add_inst_to_function(self.init_register_str(&left_register, i.value()));
                }
                if let ValueKind::Integer(i) = right_exp.kind() {
                    self.add_inst_to_function(self.init_register_str(&right_register, i.value()));
                }
                match binary.op() {
                    BinaryOp::Eq => {
                        self.add_inst_to_function(self.clear_register_str(&result_register));
                        self.add_inst_to_function(self.sub_inst_str(&result_register, &left_register, &right_register));
                        self.add_inst_to_function(self.eq0_inst_str(&result_register));
                    },
                    BinaryOp::NotEq => {
                        // 清空目标寄存器
                        self.add_inst_to_function(self.clear_register_str(&result_register));
                        self.add_inst_to_function(self.sub_inst_str(&result_register, &left_register, &right_register));
                        self.add_inst_to_function(self.neq0_inst_str(&result_register));
                    },
                    BinaryOp::Lt => {
                        self.add_inst_to_function(self.lt_inst_str(&result_register, &left_register, &right_register));
                    },
                    BinaryOp::Gt => {
                        self.add_inst_to_function(self.gt_inst_str(&result_register, &left_register, &right_register));
                    },
                    BinaryOp::Le => {
                        self.add_inst_to_function(self.le_inst_str(&result_register, &left_register, &right_register));
                    },
                    BinaryOp::Ge => {
                        self.add_inst_to_function(self.ge_inst_str(&result_register, &left_register, &right_register));
                    },
                    BinaryOp::Sub => {
                        self.add_inst_to_function(self.sub_inst_str(&result_register, &left_register, &right_register));
                    },
                    BinaryOp::Add => {
                        self.add_inst_to_function(self.add_inst_str(&result_register, &left_register, &right_register));
                    },
                    BinaryOp::Mul => {
                        self.add_inst_to_function(self.mul_inst_str(&result_register, &left_register, &right_register));
                    },
                    BinaryOp::Div => {
                        self.add_inst_to_function(self.div_inst_str(&result_register, &left_register, &right_register));
                    },
                    BinaryOp::Mod => {
                        self.add_inst_to_function(self.mod_inst_str(&result_register, &left_register, &right_register));
                    },
                    BinaryOp::And => {
                        self.add_inst_to_function(self.and_inst_str(&result_register, &left_register, &right_register));
                    },
                    BinaryOp::Or => {
                        self.add_inst_to_function(self.or_inst_str(&result_register, &left_register, &right_register));
                    }
                    _ => todo!()
                }
                // 简单寄存器优化
                // 如果两个操作数表达式是立即数而非其他指令的结果，我们在生成指令后直接释放操作数寄存器
                // 因为寄存器存放的是操作的立即数，而所有立即数都是当场分配的，之后就不可能被用，直接释放就行
                // 如果存放的是其他指令的结果，那就不能释放，因为其他指令结果不是当场分配的，万一之后的指令还引用了这一结果就完了。
                // PS: 不优化一下的话过不去 Lv3 的 TESTCASE 27（Complex binary）
                if let ValueKind::Integer(_) = left_exp.kind() {
                    self.remove_register(dfg, &left_register).unwrap();
                }
                if let ValueKind::Integer(_) = right_exp.kind() {
                    self.remove_register(dfg, &right_register).unwrap();
                }
                // 如果操作数/结果寄存器是临时寄存器，则释放
                if let Symbol::Stack(s) = result_symbol {
                    // 存放结果
                    self.add_inst_to_function(self.store_inst_str(&result_register, s));
                    self.remove_reserved_register(&result_register);
                }
                if let Symbol::Stack(_) = left_symbol {
                    self.remove_reserved_register(&left_register);
                }
                if let Symbol::Stack(_) = right_symbol {
                    self.remove_reserved_register(&right_register);
                }
            },
            ValueKind::Alloc(_) => {
                // 分配空间
                let symbol = self.get_symbol_for_value(dfg, &inst, true);
                self.symbol_table.insert(inst, symbol);
            },
            ValueKind::Load(l) => {
                let origin_symbol = l.src();
                let symbol = self.symbol_table.get(&origin_symbol).unwrap();
                // 直接复制已有符号为当前的 Symbol
                // 不用加载，需要的时候再加载。
                self.symbol_table.insert(inst, symbol.clone());
            },
            ValueKind::Store(s) => {
                
                match dfg.value(s.value()).kind() {
                    // 立即数赋值
                    ValueKind::Integer(i) => {
                        match self.symbol_table.get(&s.dest()).unwrap().clone() {
                            Symbol::Register(r) => self.add_inst_to_function(self.init_register_str(&r, i.value())),
                            Symbol::Stack(s) => {
                                let register = self.get_reserved_register_without_load();
                                self.add_inst_to_function(self.init_register_str(&register, i.value()));
                                self.add_inst_to_function(self.store_inst_str(&register, s));
                                self.remove_reserved_register(&register);
                            }
                        }
                    },
                    _ => {
                        // 应当在符号表中
                        match self.symbol_table.get(&s.value()).unwrap() {
                            Symbol::Register(r_value) => {
                                match self.symbol_table.get(&s.dest()).unwrap() {
                                    Symbol::Register(r_dest) => {
                                        self.add_inst_to_function(self.move_register_inst_str(r_dest, r_value));
                                    },
                                    Symbol::Stack(s_dest) => {
                                        self.add_inst_to_function(self.store_inst_str(r_value, *s_dest));
                                    }
                                }
                            },
                            Symbol::Stack(s_value) => {
                                match self.symbol_table.get(&s.dest()).unwrap().clone() {
                                    Symbol::Register(r_dest) => {
                                        self.add_inst_to_function(self.load_inst_str(&r_dest, *s_value));
                                    },
                                    Symbol::Stack(s_dest) => {
                                        let temp_register = self.get_reserved_register(*s_value);
                                        self.add_inst_to_function(self.store_inst_str(&temp_register, s_dest));
                                        self.remove_reserved_register(&temp_register);
                                    }
                                }
                            }
                        }
                    }
                }
            }
            _ => todo!()
        }
    }
}