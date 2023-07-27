use std::sync::Arc;
use solang_parser::pt;
use crate::codegen::cfg;
use crate::codegen::cfg::{ASTFunction, BasicBlock, ControlFlowGraph, Expression};
use crate::sema::ast::{Parameter, Type};

use num_bigint::{BigInt};

#[derive(Clone)]
pub enum Operand {
    Variable {
        ty: Type,
        res: usize,
    },
    Literal {
        ty: Type,
        value: BigInt,
    },
    TempVar {
        ty: Type,
        res: usize,
    }
}

pub enum Inst {
    Set {
        loc: pt::Loc,
        res: Operand,
        expr: Expr,
    },
    Call {
        res: Vec<usize>,
        call: cfg::InternalCallTy,
        args: Vec<Expr>,
    },
    Return { value: Vec<Expr> },
    Branch { block: usize },
}

#[derive(Clone)]
pub enum Expr {
    Add {
        loc: pt::Loc,
        ty: Type,
        left: Operand,
        right: Operand,
    },
    Operand {
        oper: Operand,
    }
}

pub struct ThreeAddressesBlock {
    pub name: String,
    pub insts: Vec<Inst>,
}

pub struct StaticSingleAssignment {
    pub name: String,
    pub function_no: ASTFunction,
    pub params: Arc<Vec<Parameter>>,
    pub returns: Arc<Vec<Parameter>>,
    pub blocks: Vec<ThreeAddressesBlock>,
}

impl Operand {
    fn new_temp_var(ty: Type) -> Self {
        Operand::TempVar {res: 0, ty}
    }
}

impl Inst {
    pub fn from_expression(expression: &Expression) -> Vec<Self> {
        let mut instr = vec![];

        match expression {
            Expression::Add {
                loc,
                ty,
                overflowing,
                left,
                right
            } if matches!(**left, Expression::NumberLiteral{ .. }) => {
                let mut right_inst = Inst::from_expression(right);
                let value = match **left {
                    Expression::NumberLiteral{ ref value, .. } => value.clone(),
                    _ => panic!("Error!"),
                };

                match right_inst.last() {
                    Some(Inst::Set { res: Operand::TempVar { res, ty }, expr, loc })
                    | Some(Inst::Set { res: Operand::Variable { res, ty }, expr, loc }) => {
                        let left_operand = Operand::Literal { ty: ty.clone(), value };
                        let expr = Expr::Add {
                            loc: loc.clone(),
                            ty: ty.clone(),
                            left: left_operand,
                            right: Operand::Variable { res: res.clone(), ty: ty.clone() },
                        };
                        let last_inst = Inst::Set {
                            res: Operand::new_temp_var(ty.clone()),
                            expr,
                            loc: loc.clone()
                        };
                        instr.append(&mut right_inst);
                        instr.push(last_inst);
                    },
                    _ => instr.append(&mut right_inst),
                }
            },
            Expression::Add {
                loc,
                ty,
                overflowing,
                left,
                right
            } if matches!(**right, Expression::NumberLiteral{ .. }) => {
                let mut left_inst = Inst::from_expression(left);
                let value = match **right {
                    Expression::NumberLiteral{ ref value, .. } => value.clone(),
                    _ => panic!("Error!"),
                };

                match left_inst.last() {
                    Some(Inst::Set { res: Operand::TempVar { res, ty }, expr, loc })
                    | Some(Inst::Set { res: Operand::Variable { res, ty }, expr, loc }) => {
                        // assert_eq!(last_inst, Inst::Set);
                        // assert!(matches!(res, Operand::TempVar {..}));
                        let right_operand = Operand::Literal { ty: ty.clone(), value };
                        let expr = Expr::Add {
                            loc: loc.clone(),
                            ty: ty.clone(),
                            left: Operand::Variable { res: res.clone(), ty: ty.clone() },
                            right: right_operand,
                        };
                        let last_inst = Inst::Set {
                            res: Operand::new_temp_var(ty.clone()),
                            expr,
                            loc: loc.clone()
                        };
                        instr.append(&mut left_inst);
                        instr.push(last_inst);
                    },
                    _ => instr.append(&mut left_inst),
                }
            },
            Expression::Add { left, right, loc, ty, .. } => {
                let mut right_inst = Inst::from_expression(right);
                let mut left_inst = Inst::from_expression(left);
                let Some(Inst::Set{res: right_operand, .. }) = right_inst.last() else {
                    panic!("Error!");
                };
                let Some(Inst::Set{res: left_operand, .. }) = left_inst.last() else {
                    panic!("Error!");
                };
                let last_inst = Inst::Set {
                    res: Operand::new_temp_var(ty.clone()),
                    expr: Expr::Add { loc: loc.clone(), ty: ty.clone(), left: left_operand.clone(), right: right_operand.clone()},
                    loc: loc.clone()
                };
                instr.append(&mut left_inst);
                instr.append(&mut right_inst);
                instr.push(last_inst);
            },
            Expression::NumberLiteral { loc, ty, value } => {
                let literal = Operand::Literal {ty: ty.clone(), value: value.clone()};
                let new_inst = Inst::Set {
                    res: Operand::new_temp_var(ty.clone()),
                    loc: loc.clone(),
                    expr: Expr::Operand { oper: literal },
                };
                instr.push(new_inst);
            }
            _ => {},
        }

        instr
    }
}

impl ThreeAddressesBlock {
    pub fn edges(&self) -> Vec<usize> {
        let mut edges = Vec::new();

        for inst in self.insts.iter() {
            match inst {
                Inst::Branch { block } => {
                    edges.push(*block);
                }
                _ => (),
            }
        }

        edges
    }

    pub fn from_basic(block: &BasicBlock) -> Self {
        let mut new_block = ThreeAddressesBlock {
            name: block.name.clone(),
            insts: Vec::new(),
        };

        for instr in block.instr.iter() {
            match instr {
                cfg::Instr::Set {
                    loc,
                    res,
                    expr
                } => {
                    let mut insts = Inst::from_expression(expr);
                    if let Some(Inst::Set {loc, res: Operand::TempVar { ty, ..} , expr}) = insts.pop() {
                        // insts.last_mut() =
                        let new_inst = Inst::Set {
                            loc: loc.clone(),
                            res: Operand::Variable { res: res.clone(), ty: ty.clone() },
                            expr: expr.clone(),
                        };
                        insts.push(new_inst);
                        new_block.insts.append(&mut insts);
                    }
                }
                _ => (),
            }
        }

        new_block
    }
}

impl StaticSingleAssignment {
    pub fn new(name: String, function_no: ASTFunction) -> Self {
        StaticSingleAssignment {
            name,
            function_no,
            params: Arc::new(Vec::new()),
            returns: Arc::new(Vec::new()),
            blocks: Vec::new(),
        }
    }

    pub fn from_cfg(cfg: &ControlFlowGraph) -> Self {
        let mut ssa = StaticSingleAssignment {
            name: cfg.name.clone(),
            function_no: cfg.function_no,
            params: Arc::new(Vec::new()),
            returns: Arc::new(Vec::new()),
            blocks: Vec::new(),
        };

        for block in cfg.blocks.iter() {
            let three_addresses = ThreeAddressesBlock::from_basic(block);
            ssa.blocks.push(three_addresses);
        }

        ssa
    }
}
