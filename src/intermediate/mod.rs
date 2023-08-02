use std::sync::Arc;
use num_bigint::BigInt;
use solang_parser::pt;
use crate::codegen::cfg;
use crate::codegen::cfg::ASTFunction;
use crate::sema::ast::{Parameter, Type};

pub mod ssa;


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
    //TODO: remove TempVar
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