use std::sync::Arc;
use solang_parser::pt;
use crate::codegen::cfg;
use crate::codegen::cfg::{ASTFunction, ControlFlowGraph};
use crate::intermediate::ssa;
use crate::sema::ast::{Parameter, Type};


pub enum Operand {
    Variable {
        ty: Type
    },
    Literal,
}

pub enum Inst {
    Set {
        res: Operand,
        expr: Expr,
    },
    Call {
        res: Vec<Operand>,
        call: cfg::InternalCallTy,
        args: Vec<Expr>,
    },
    Return { value: Vec<Expr> },
    Branch { block: usize }
}

pub enum Expr {
    Add {
        loc: pt::Loc,
        ty: ssa::Type,
        left: Operand,
        right: Operand
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

impl ThreeAddressesBlock {
    pub fn edges(&self) -> Vec<usize> {
        let mut edges = Vec::new();

        for inst in self.insts.iter() {
            match inst {
                Inst::Branch { block } => {
                    edges.push(*block);
                },
                _ => (),
            }
        }

        edges
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
        let ssa = StaticSingleAssignment {
            name: cfg.name.clone(),
            function_no: cfg.function_no,
            params: Arc::new(Vec::new()),
            returns: Arc::new(Vec::new()),
            blocks: Vec::new(),
        };

        ssa
    }
}
