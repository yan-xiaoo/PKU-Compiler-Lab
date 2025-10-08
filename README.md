# 北京大学编译原理实践

这是个人自学[北京大学编译原理实践课程](https://pku-minic.github.io/online-doc/#/)中编写的代码。

此项目实现了一个从 Sysy 语言到 Koopa IR 或 RISC-V 的编译器。项目采用 Rust 实现。

主要结构：

> 大部分文件位于 src 文件夹下

1. main.rs： 处理命令行参数，调用内部编译接口
2. sysy.lalrpop, function_ast.rs：定义前端处理过程，实现词法分析和语法分析。
3. ir_gen.rs：从语法分析返回的 AST（对象树）生成 Koopa IR
4. assembly.rs：从 Koopa IR 结构生成汇编代码。