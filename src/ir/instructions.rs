use std::fmt::Display;

use crate::{parser::simple_type::Signedness, semantics::semantics::SymbolID};

pub(super) struct Program(pub Vec<Module>);

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub(super) struct VRegisterID(pub u64);

impl Display for VRegisterID {
    fn fmt(&self, display: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(display, "{}", self.0)
    }
}

impl VRegisterID {
    pub fn next(&mut self) -> VRegisterID {
        // let temp = *self;
        self.0 += 1;
        *self
    }

    pub fn prev(&self) -> VRegisterID {
        VRegisterID(self.0 - 1)
    }
}

#[derive(Clone, Copy, Default, Debug)]
pub(super) enum IRType {
    I1,
    I8,
    I16,
    #[default]
    I32,
    I64,
    Float,
    Double,
    X86Fp80,
    Ptr,
    Void,
}

impl IRType {
    pub(super) fn num_bits(&self) -> u8 {
        match self {
            Self::I1 => 1,
            Self::I8 => 8,
            Self::I16 => 16,
            Self::I32 => 32,
            Self::I64 => 64,
            Self::Float => 32,
            Self::Double => 64,
            Self::X86Fp80 => 128,
            Self::Ptr => 64,
            Self::Void => 0,
        }
    }

    fn alignment(&self) -> u8 {
        self.num_bits().div_ceil(8)
    }

    pub(super) fn is_floating(&self) -> bool {
        match self {
            Self::Float | Self::Double | Self::X86Fp80 => true,
            _ => false,
        }
    }
}

impl Display for IRType {
    fn fmt(&self, display: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let output = match self {
            Self::I1 => "i1",
            Self::I8 => "i8",
            Self::I16 => "i16",
            Self::I32 => "i32",
            Self::I64 => "i64",
            Self::Float => "float",
            Self::Double => "double",
            Self::X86Fp80 => "x86_fp80",
            Self::Ptr => "ptr",
            Self::Void => "void",
        };

        write!(display, "{output}")
    }
}

#[derive(Default, Clone, Copy, Debug)]
pub(super) struct SignedIRType {
    pub ir_type: IRType,
    pub signedness: Signedness,
}

pub(super) enum Module {
    FunctionDef {
        return_type: IRType,
        name: String,
        instructions: Option<Vec<Instruction>>,
        params: Vec<SignedIRType>,
    },
}

#[derive(Clone)]
pub(super) enum RValue {
    VRegister(VRegisterID),
    Literal(i64),
    Void,
}

impl Display for RValue {
    fn fmt(&self, display: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let output = match self {
            Self::VRegister(id) => format!("%{}", id.0),
            Self::Literal(value) => value.to_string(),
            Self::Void => String::new(),
        };

        write!(display, "{output}")
    }
}

#[derive(Clone)]
pub(super) enum BinaryKind {
    Add,
    Sub,
    Mul,
    Udiv,
    Sdiv,
    Fadd,
    Fsub,
    Fmul,
    Fdiv,
    And,
    Or,
    Xor,
    Shl,
    Lshr,
    Ashr,
}

impl Display for BinaryKind {
    fn fmt(&self, display: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let output = match self {
            Self::Add => "add",
            Self::Sub => "sub",
            Self::Mul => "mul",
            Self::Udiv => "udiv",
            Self::Sdiv => "sdiv",
            Self::Fadd => "fadd",
            Self::Fsub => "fsub",
            Self::Fmul => "fmul",
            Self::Fdiv => "fdiv",
            Self::And => "and",
            Self::Or => "or",
            Self::Xor => "xor",
            Self::Shl => "shl",
            Self::Lshr => "lshr",
            Self::Ashr => "ashr",
        };

        write!(display, "{output}")
    }
}

#[derive(Clone)]
pub(super) enum CastKind {
    Zext,
    Sext,
    Trunc,
    Sitofp,
    Uitofp,
    Fptosi,
    Fptoui,
    Fpext,
    Fptrunc,
}

impl Display for CastKind {
    fn fmt(&self, display: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let output = match self {
            Self::Zext => "zext",
            Self::Sext => "sext",
            Self::Trunc => "trunc",
            Self::Sitofp => "sitofp",
            Self::Uitofp => "uitofp",
            Self::Fptosi => "fptosi",
            Self::Fptoui => "fptoui",
            Self::Fpext => "fpext",
            Self::Fptrunc => "fptrunc",
        };

        write!(display, "{output}")
    }
}

#[derive(Clone)]
pub(super) enum Instruction {
    Return {
        size: IRType,
        value: RValue,
    },
    Alloca {
        size: IRType,
        v_register: VRegisterID,
    },
    Store {
        size: IRType,
        value: RValue,
        store_ptr: VRegisterID,
    },
    Load {
        size: IRType,
        value_ptr: VRegisterID,
        v_register: VRegisterID,
    },
    BinaryOp {
        kind: BinaryKind,
        size: IRType,
        left: RValue,
        right: RValue,
        v_register: VRegisterID,
    },
    Cast {
        kind: CastKind,
        from_size: IRType,
        to_size: IRType,
        value: RValue,
        v_register: VRegisterID,
    },
    Call {
        func_name: String,
        args: Vec<(IRType, RValue)>,
        return_value: IRType,
        v_register: Option<VRegisterID>,
    },
}

impl Instruction {
    pub(super) fn as_llvm_ir(&self) -> String {
        match self {
            Self::Return { size, value } => match size {
                IRType::Void => format!("  ret {size}\n"),
                _ => format!("  ret {size} {value}\n"),
            },

            Self::Alloca { size, v_register } => {
                format!(
                    "  %{v_register} = alloca {size}, align {}\n",
                    size.alignment()
                )
            }

            Self::Store {
                size,
                value,
                store_ptr,
            } => format!(
                "  store {size} {value}, ptr %{store_ptr}, align {}\n",
                size.alignment()
            ),

            Self::Load {
                size,
                value_ptr: value,
                v_register,
            } => format!(
                "  %{v_register} = load {size}, ptr %{value}, align {}\n",
                size.alignment()
            ),

            Self::BinaryOp {
                kind,
                size,
                left,
                right,
                v_register,
            } => format!("  %{v_register} = {kind} {size} {left}, {right}\n"),

            Self::Cast {
                kind,
                v_register,
                from_size,
                to_size,
                value,
            } => format!("  %{v_register} = {kind} {from_size} {value} to {to_size} \n"),

            Self::Call {
                func_name,
                args,
                return_value,
                v_register,
            } => {
                let mut output = format!("  ");

                if let Some(v_register) = v_register {
                    output.push_str(&format!("%{v_register} = "));
                }

                output.push_str(&format!("call {return_value} @{func_name}("));

                for (i, (ir_type, r_value)) in args.iter().enumerate() {
                    output.push_str(&format!("{ir_type} {r_value}"));

                    if i != args.len() - 1 {
                        output.push_str(", ");
                    }
                }

                output.push_str(")\n");
                output
            }
        }
    }
}

#[derive(Default, Copy, Clone)]
pub(super) struct AllocationInfo {
    pub allocated_id: VRegisterID,
}

#[derive(Default)]
pub(super) struct VRegisterIdentifierLookup {
    lookup: Vec<AllocationInfo>,
}

impl VRegisterIdentifierLookup {
    pub fn add_identifier(&mut self, symbol_id: SymbolID, allocation: AllocationInfo) {
        if self.lookup.len() <= symbol_id.0 as usize {
            self.lookup
                .resize(symbol_id.0 as usize + 1, AllocationInfo::default());
        }

        self.lookup[symbol_id.0 as usize] = allocation;
    }

    pub fn lookup_v_register(&mut self, symbol_id: SymbolID) -> AllocationInfo {
        self.lookup[symbol_id.0 as usize]
    }
}

#[derive(Default)]
pub(super) struct VRegisterTypeLookup {
    lookup: Vec<SignedIRType>,
}

impl VRegisterTypeLookup {
    pub fn add_type(&mut self, v_register: VRegisterID, signed_ir_type: SignedIRType) {
        if self.lookup.len() <= v_register.0 as usize {
            self.lookup
                .resize(v_register.0 as usize + 1, SignedIRType::default());
        }

        self.lookup[v_register.0 as usize] = signed_ir_type;
    }

    pub fn lookup_type(&mut self, v_register: VRegisterID) -> &SignedIRType {
        &self.lookup[v_register.0 as usize]
    }
}
