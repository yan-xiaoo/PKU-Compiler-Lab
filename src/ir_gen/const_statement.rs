use crate::{error_report::{Label, ProblemInfo}, function_ast::{AddExp, AddOp, BType, ConstDecl, ConstDef, ConstInitVal, EqExp, EqOp, Exp, LAndExp, LOrExp, LVal, MulExp, MulOp, PrimaryExp, RelExp, RelOp, UnaryExp, UnaryOp}, ir_gen::Symbol};

use super::IrGen;

impl IrGen {
    pub(super) fn generate_const_statement(&self, const_statement: &ConstDecl) -> Result<(),()> {
        for one in &const_statement.const_def {
            self.generate_const_definition(&const_statement.b_type, one)?;
        }
        Ok(())
    }

    
    pub(super) fn generate_const_definition(&self, _: &BType, def: &ConstDef) -> Result<(),()> {
        let result = self.calculate_const_statement(&def.const_init_val)?;
        if self.new_const_symbol(def.ident.clone(), result).is_err() {
            self.problems.borrow_mut().push(ProblemInfo::error(format!("duplicate symbol '{}' found.", def.ident), 
                               vec![Label::primary("Note: duplicate symbol found here.", def.span)], None));
            Err(())
        } else {
            Ok(())
        }   
    } 

    pub(super) fn calculate_const_statement(&self, const_init_val: &ConstInitVal) -> Result<i32, ()> {
        self.calculate_expression(&const_init_val.const_exp.exp)
    }

    pub(super) fn calculate_expression(&self, exp: &Exp) -> Result<i32, ()> {
        self.calculate_l_or_expression(&exp.l_or_exp)
    }

    pub(super) fn calculate_l_or_expression(&self, l_or_exp: &LOrExp) -> Result<i32, ()> {
        match l_or_exp {
            LOrExp::LAndExp(l_and_exp) => {
                self.calculate_l_and_exp(l_and_exp)
            },
            LOrExp::CompoundLOrExp(l_or_exp, l_and_exp) => {
                let left = self.calculate_l_or_expression(l_or_exp)?;
                let right = self.calculate_l_and_exp(l_and_exp)?;
                if left != 0 || right != 0 {
                    Ok(1)
                } else {
                    Ok(0)
                }
            }
        }
    }

    pub(super) fn calculate_l_and_exp(&self, l_and_exp: &LAndExp) -> Result<i32, ()> {
        match l_and_exp {
            LAndExp::EqExp(eq_exp) => {
                self.calculate_eq_exp(eq_exp)
            },
            LAndExp::CompoundLAndExp(l_and_exp, eq_exp) => {
                let left = self.calculate_l_and_exp(l_and_exp)?;
                let right = self.calculate_eq_exp(eq_exp)?;
                if left != 0 && right != 0 {
                    Ok(1)
                } else {
                    Ok(0)
                }
            }
        }
    }

    pub(super) fn calculate_eq_exp(&self, eq_exp: &EqExp) -> Result<i32, ()> {
        match eq_exp {
            EqExp::RelExp(rel_exp) => {
                self.calculate_rel_exp(rel_exp)
            },
            EqExp::CompoundEqExp(eq_exp, rel_exp, eq_op) => {
                let left = self.calculate_eq_exp(eq_exp)?;
                let right = self.calculate_rel_exp(rel_exp)?;
                match eq_op {
                    EqOp::Eq => {
                        Ok((left == right) as i32)
                    },
                    EqOp::Ne => {
                        Ok((left != right) as i32)
                    }
                }
            }
        }
    }

    pub(super) fn calculate_rel_exp(&self, rel_exp: &RelExp) -> Result<i32, ()> {
        match rel_exp {
            RelExp::AddExp(add_exp) => {
                self.calculate_add_exp(add_exp)
            },
            RelExp::CompoundRelExp(rel_exp, add_exp, rel_op) => {
                let left = self.calculate_rel_exp(rel_exp)?;
                let right = self.calculate_add_exp(add_exp)?;
                match rel_op {
                    RelOp::Ge => {
                        Ok((left >= right) as i32)
                    },
                    RelOp::Gt => {
                        Ok((left > right) as i32)
                    },
                    RelOp::Le => {
                        Ok((left <= right) as i32)
                    },
                    RelOp::Lt => {
                        Ok((left < right) as i32)
                    }
                }
            }   
        }
    }

    pub(super) fn calculate_add_exp(&self, add_exp: &AddExp) -> Result<i32, ()> {
        match add_exp {
            AddExp::MulExp(mul_exp) => {
                self.calculate_mul_exp(mul_exp)
            },
            AddExp::CompoundAddExp(add_exp, mul_exp, add_op) => {
                let left = self.calculate_add_exp(add_exp)?;
                let right = self.calculate_mul_exp(mul_exp)?;
                match add_op {
                    AddOp::Plus => Ok(left + right),
                    AddOp::Minus => Ok(left - right)
                }
            }
        }
    }

    pub(super) fn calculate_mul_exp(&self, mul_exp: &MulExp) -> Result<i32, ()> {
        match mul_exp {
            MulExp::UnaryExp(unary_exp) => {
                self.calculate_unary_exp(unary_exp)
            },
            MulExp::CompoundMulExp(mul_exp, unary_exp, mul_op) => {
                let left = self.calculate_mul_exp(mul_exp)?;
                let right = self.calculate_unary_exp(unary_exp)?;
                match mul_op {
                    MulOp::Mul => Ok(left * right),
                    MulOp::Div => Ok(left / right),
                    MulOp::Mod => Ok(left % right)
                }
            }
        }
    }

    pub(super) fn calculate_unary_exp(&self, unary_exp: &UnaryExp) -> Result<i32, ()> {
        match unary_exp {
            UnaryExp::PrimaryExp(primary_exp) => {
                self.calculate_primary_exp(primary_exp)
            },
            UnaryExp::CompoundUnaryExp(unary_op, unary_exp) => {
                let internal = self.calculate_unary_exp(unary_exp)?;
                match unary_op {
                    UnaryOp::Plus => Ok(internal),
                    UnaryOp::Minus => Ok(-internal),
                    UnaryOp::Not => Ok((internal == 0) as i32)
                }
            }
        }
    }

    pub(super) fn calculate_primary_exp(&self, primary_exp: &PrimaryExp) -> Result<i32, ()> {
        match primary_exp {
            PrimaryExp::Number(i) => Ok(*i),
            PrimaryExp::LVal(l_val) => {
                match l_val {
                    LVal::Ident(s, span) => {
                        if let Ok(symbol) = self.get_symbol(s) {
                            match symbol {
                                Symbol::Const(const_val) => {
                                    Ok(const_val)
                                },
                                Symbol::Var(_) => {
                                     self.problems.borrow_mut().push(ProblemInfo::error(format!("variable '{}' found in const value definition", s), 
                                vec![Label::primary("Note: assignment occuried here.", *span), 
                                            Label::secondary("Note: only use literals and other const value in const value definition", *span)], None));
                                    Err(())
                                }
                            }
                        } else {
                            self.problems.borrow_mut().push(ProblemInfo::error(format!("use of undeclared identifier '{}'", s), 
                                                            vec![Label::primary("Note: error occuried here.", *span)], None));
                            Err(())
                        }
                    }
                }
            },
            PrimaryExp::Exp(exp) => {
                self.calculate_l_or_expression(&exp.l_or_exp)
            }
        }
    }
}