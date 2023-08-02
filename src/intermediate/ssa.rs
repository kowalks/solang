use std::sync::Arc;

use crate::codegen::cfg;
use crate::codegen::cfg::{ASTFunction, BasicBlock, ControlFlowGraph, Expression};
use crate::intermediate::{Expr, Inst, Operand, StaticSingleAssignment, ThreeAddressesBlock};

//TODO: create ssa Type
// enum Type {
//     Integer(u16), // width
//     // Address is just Array
//     // but array is Pointer(Box<Array>)
//     Bytes(u8), // it can also represented by int (but endianness is opposite, but we should not be worried about it)
//     UInt(u16),  // width
//     Bool,
//     // Rational,
//
//     Array{ty: Box<Type>, dim: Vec<usize>},  // fixed-length
//     Pointer(Box<Type>),
//     Struct(StructDecl),   // definition of struct ***
//     StorageReference(),
// }

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
                    Expression::NumberLiteral { ref value, .. } => value.clone(),
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
                    Expression::NumberLiteral { ref value, .. } => value.clone(),
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
                let Some(Inst::Set { res: right_operand, .. }) = right_inst.last() else {
                    panic!("Error!");
                };
                let Some(Inst::Set { res: left_operand, .. }) = left_inst.last() else {
                    panic!("Error!");
                };
                let last_inst = Inst::Set {
                    res: Operand::new_temp_var(ty.clone()),
                    expr: Expr::Add { loc: loc.clone(), ty: ty.clone(), left: left_operand.clone(), right: right_operand.clone() },
                    loc: loc.clone()
                };
                instr.append(&mut left_inst);
                instr.append(&mut right_inst);
                instr.push(last_inst);
            },
            Expression::NumberLiteral { loc, ty, value } => {
                let literal = Operand::Literal { ty: ty.clone(), value: value.clone() };
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
