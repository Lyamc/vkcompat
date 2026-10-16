//! SPIR-V parser, fixed-function classifier, and interpreter.
//! The interpreter covers the straight-line draws and the phi/loop fragment
//! shader used by the pixel tests. Hardware emission refuses control flow.

use std::collections::HashMap;

pub const OP_ENTRY: u16 = 15;
pub const OP_EXEC_MODE: u16 = 16;
pub const OP_TYPE_VOID: u16 = 19;
pub const OP_TYPE_BOOL: u16 = 20;
pub const OP_TYPE_INT: u16 = 21;
pub const OP_TYPE_FLOAT: u16 = 22;
pub const OP_TYPE_VECTOR: u16 = 23;
pub const OP_TYPE_IMAGE: u16 = 25;
pub const OP_TYPE_SAMPLED_IMAGE: u16 = 27;
pub const OP_TYPE_RUNTIME_ARRAY: u16 = 29;
pub const OP_TYPE_STRUCT: u16 = 30;
pub const OP_TYPE_POINTER: u16 = 32;
pub const OP_CONSTANT: u16 = 43;
pub const OP_CONSTANT_COMPOSITE: u16 = 44;
pub const OP_FUNCTION: u16 = 54;
pub const OP_FUNCTION_END: u16 = 56;
pub const OP_VARIABLE: u16 = 59;
pub const OP_LOAD: u16 = 61;
pub const OP_STORE: u16 = 62;
pub const OP_ACCESS_CHAIN: u16 = 65;
pub const OP_DECORATE: u16 = 71;
pub const OP_MEMBER_DECORATE: u16 = 72;
pub const OP_COMPOSITE_CONSTRUCT: u16 = 80;
pub const OP_COMPOSITE_EXTRACT: u16 = 81;
pub const OP_IMAGE_SAMPLE_IMPLICIT_LOD: u16 = 87;
pub const OP_FADD: u16 = 129;
pub const OP_IADD: u16 = 128;
pub const OP_SLESSTHAN: u16 = 177;
pub const OP_PHI: u16 = 245;
pub const OP_LOOP_MERGE: u16 = 246;
pub const OP_LABEL: u16 = 248;
pub const OP_BRANCH: u16 = 249;
pub const OP_BRANCH_CONDITIONAL: u16 = 250;
pub const OP_KILL: u16 = 252;
pub const OP_RETURN: u16 = 253;

pub const DEC_BLOCK: u32 = 2;
pub const DEC_ARRAY_STRIDE: u32 = 6;
pub const DEC_BUILTIN: u32 = 11;
pub const DEC_LOCATION: u32 = 30;
pub const DEC_BINDING: u32 = 33;
pub const DEC_SET: u32 = 34;
pub const DEC_OFFSET: u32 = 35;

pub const BUILTIN_POSITION: u32 = 0;
pub const BUILTIN_VERTEX_ID: u32 = 5;
pub const BUILTIN_FRAGCOORD: u32 = 15;
pub const BUILTIN_GLOBAL_INVOCATION_ID: u32 = 28;
pub const BUILTIN_VERTEX_INDEX: u32 = 42;
pub const BUILTIN_INSTANCE_INDEX: u32 = 43;

pub const MODEL_VERTEX: u32 = 0;
pub const MODEL_FRAGMENT: u32 = 4;
pub const MODEL_GLCOMPUTE: u32 = 5;
pub const EXEC_ORIGIN_UPPER_LEFT: u32 = 7;
pub const EXEC_LOCAL_SIZE: u32 = 17;

pub const SC_UNIFORM_CONSTANT: u32 = 0;
pub const SC_INPUT: u32 = 1;
pub const SC_UNIFORM: u32 = 2;
pub const SC_OUTPUT: u32 = 3;
pub const SC_STORAGE_BUFFER: u32 = 12;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FfKind {
    None,
    Const,
    Tex,
    TexMul,
}

#[derive(Clone, Debug)]
pub struct FfInfo {
    pub kind: FfKind,
    pub color: [f32; 4],
    pub uv_loc: i32,
    pub set: u32,
    pub binding: u32,
}

impl FfInfo {
    fn none() -> Self {
        Self {
            kind: FfKind::None,
            color: [0.0; 4],
            uv_loc: 0,
            set: 0,
            binding: 0,
        }
    }
}

#[derive(Clone, Debug)]
pub enum Ty {
    Void,
    Bool,
    Int {
        signed: bool,
    },
    Float,
    Vec {
        elem: Box<Ty>,
        n: u8,
    },
    Sampled,
    RuntimeArray {
        elem: Box<Ty>,
        stride: u32,
    },
    Struct {
        members: Vec<Member>,
    },
    Pointer {
        storage: u32,
        pointee: Box<Ty>,
    },
    Other,
}

#[derive(Clone, Debug)]
pub struct Member {
    pub ty: Ty,
    pub offset: u32,
}

#[derive(Clone, Debug)]
pub struct Inst {
    pub op: u16,
    pub result: u32,
    pub operands: Vec<u32>,
}

#[derive(Clone, Debug)]
pub struct Block {
    pub id: u32,
    pub insts: Vec<Inst>,
}

#[derive(Clone, Debug)]
pub struct Func {
    pub id: u32,
    pub blocks: Vec<Block>,
}

#[derive(Clone, Debug)]
pub struct Variable {
    pub id: u32,
    pub storage: u32,
    pub ty: Ty,
    pub location: i32,
    pub binding: i32,
    pub set: i32,
    pub builtin: i32,
}

#[derive(Clone, Debug)]
pub struct Module {
    pub bound: usize,
    pub model: u32,
    pub entry: u32,
    pub entry_name: String,
    pub local_size: [u32; 3],
    pub has_control_flow: bool,
    pub types: HashMap<u32, Ty>,
    pub constants: HashMap<u32, Value>,
    pub vars: HashMap<u32, Variable>,
    pub funcs: Vec<Func>,
    /// Defining instruction for each result id inside functions.
    pub defs: HashMap<u32, Inst>,
}

#[derive(Clone, Debug)]
pub enum Value {
    Undef,
    Bool(bool),
    Scalar {
        bits: u32,
        float: bool,
        signed: bool,
    },
    Vec {
        bits: [u32; 4],
        n: u8,
        float: bool,
        signed: bool,
    },
    Sampled {
        set: u32,
        binding: u32,
    },
    Ptr {
        var: u32,
        offset: u32,
    },
}

impl Value {
    fn f32s(xs: &[f32]) -> Self {
        let mut bits = [0u32; 4];
        for (i, x) in xs.iter().enumerate().take(4) {
            bits[i] = x.to_bits();
        }
        if xs.len() == 1 {
            Value::Scalar {
                bits: bits[0],
                float: true,
                signed: true,
            }
        } else {
            Value::Vec {
                bits,
                n: xs.len() as u8,
                float: true,
                signed: true,
            }
        }
    }

    pub fn as_f32(&self) -> f32 {
        match self {
            Value::Scalar { bits, float: true, .. } => f32::from_bits(*bits),
            Value::Vec { bits, float: true, .. } => f32::from_bits(bits[0]),
            _ => 0.0,
        }
    }

    pub fn as_i32(&self) -> i32 {
        match self {
            Value::Scalar { bits, .. } => *bits as i32,
            Value::Vec { bits, .. } => bits[0] as i32,
            _ => 0,
        }
    }

    pub fn as_u32(&self) -> u32 {
        match self {
            Value::Scalar { bits, .. } => *bits,
            Value::Vec { bits, .. } => bits[0],
            _ => 0,
        }
    }

    pub fn as_vec4(&self) -> [f32; 4] {
        match self {
            Value::Vec { bits, n, float: true, .. } => {
                let mut o = [0.0, 0.0, 0.0, 1.0];
                for i in 0..*n as usize {
                    o[i] = f32::from_bits(bits[i]);
                }
                if *n < 4 && *n != 0 {
                    o[3] = if *n == 4 { o[3] } else { 1.0 };
                }
                o
            }
            Value::Scalar { bits, float: true, .. } => [f32::from_bits(*bits), 0.0, 0.0, 1.0],
            _ => [0.0, 0.0, 0.0, 1.0],
        }
    }

    fn component(&self, i: usize) -> Value {
        match self {
            Value::Vec { bits, float, signed, .. } => Value::Scalar {
                bits: bits[i],
                float: *float,
                signed: *signed,
            },
            Value::Scalar { .. } if i == 0 => self.clone(),
            _ => Value::Undef,
        }
    }

    fn to_le_bytes(&self) -> Vec<u8> {
        match self {
            Value::Vec { bits, n, .. } => {
                let mut o = Vec::new();
                for i in 0..*n as usize {
                    o.extend_from_slice(&bits[i].to_le_bytes());
                }
                o
            }
            Value::Scalar { bits, .. } => bits.to_le_bytes().to_vec(),
            _ => Vec::new(),
        }
    }
}

pub struct Io<'a> {
    pub attrs: &'a [[f32; 4]],
    pub frag_coord: [f32; 4],
    pub vertex_index: i32,
    pub instance_index: i32,
    pub global_id: [u32; 3],
    pub ubos: &'a [(u32, u32, &'a [u8])],
    pub images: &'a [ImageBind<'a>],
    pub ssbo: &'a mut [u8],
    pub ssbo_set: u32,
    pub ssbo_binding: u32,
    pub position: [f32; 4],
    pub outputs: Vec<(u32, [f32; 4])>,
}

pub struct ImageBind<'a> {
    pub set: u32,
    pub binding: u32,
    pub w: u32,
    pub h: u32,
    pub rgba: &'a [u8],
    pub nearest: bool,
    pub clamp: bool,
}

impl Module {
    pub fn func(&self) -> Option<&Func> {
        self.funcs.iter().find(|f| f.id == self.entry)
    }
}

pub fn parse(words: &[u32]) -> Result<Module, String> {
    if words.len() < 5 || words[0] != 0x0723_0203 {
        return Err("bad SPIR-V magic".into());
    }
    let bound = words[3] as usize;
    if bound > 1_000_000 {
        return Err("SPIR-V bound is too large".into());
    }
    let mut m = Module {
        bound,
        model: 0,
        entry: 0,
        entry_name: String::new(),
        local_size: [1, 1, 1],
        has_control_flow: false,
        types: HashMap::new(),
        constants: HashMap::new(),
        vars: HashMap::new(),
        funcs: Vec::new(),
        defs: HashMap::new(),
    };
    let mut locations: HashMap<u32, i32> = HashMap::new();
    let mut bindings: HashMap<u32, i32> = HashMap::new();
    let mut sets: HashMap<u32, i32> = HashMap::new();
    let mut builtins: HashMap<u32, i32> = HashMap::new();
    let mut member_offsets: HashMap<(u32, u32), u32> = HashMap::new();
    let mut array_stride: HashMap<u32, u32> = HashMap::new();

    let mut i = 5;
    let mut cur_func: Option<Func> = None;
    let mut cur_block: Option<Block> = None;
    while i < words.len() {
        let head = words[i];
        let count = (head >> 16) as usize;
        let op = (head & 0xffff) as u16;
        if count == 0 || i + count > words.len() {
            return Err("truncated SPIR-V instruction".into());
        }
        let rest = &words[i + 1..i + count];
        i += count;

        match op {
            OP_ENTRY => {
                if rest.len() >= 2 {
                    m.model = rest[0];
                    m.entry = rest[1];
                    m.entry_name = words_str(&rest[2..]);
                }
            }
            OP_EXEC_MODE => {
                if rest.len() >= 2 && rest[1] == EXEC_LOCAL_SIZE && rest.len() >= 5 {
                    m.local_size = [rest[2], rest[3], rest[4]];
                }
                let _ = EXEC_ORIGIN_UPPER_LEFT;
            }
            OP_DECORATE => {
                if rest.len() >= 2 {
                    let id = rest[0];
                    match rest[1] {
                        DEC_LOCATION if rest.len() >= 3 => {
                            locations.insert(id, rest[2] as i32);
                        }
                        DEC_BINDING if rest.len() >= 3 => {
                            bindings.insert(id, rest[2] as i32);
                        }
                        DEC_SET if rest.len() >= 3 => {
                            sets.insert(id, rest[2] as i32);
                        }
                        DEC_BUILTIN if rest.len() >= 3 => {
                            builtins.insert(id, rest[2] as i32);
                        }
                        DEC_ARRAY_STRIDE if rest.len() >= 3 => {
                            array_stride.insert(id, rest[2]);
                        }
                        DEC_BLOCK => {}
                        _ => {}
                    }
                }
            }
            OP_MEMBER_DECORATE => {
                if rest.len() >= 4 && rest[2] == DEC_OFFSET {
                    member_offsets.insert((rest[0], rest[1]), rest[3]);
                }
            }
            OP_TYPE_VOID => {
                if let Some(id) = rest.first() {
                    m.types.insert(*id, Ty::Void);
                }
            }
            OP_TYPE_BOOL => {
                if let Some(id) = rest.first() {
                    m.types.insert(*id, Ty::Bool);
                }
            }
            OP_TYPE_INT => {
                if rest.len() >= 3 {
                    m.types.insert(
                        rest[0],
                        Ty::Int {
                            signed: rest[2] != 0,
                        },
                    );
                }
            }
            OP_TYPE_FLOAT => {
                if let Some(id) = rest.first() {
                    m.types.insert(*id, Ty::Float);
                }
            }
            OP_TYPE_VECTOR => {
                if rest.len() >= 3 {
                    let elem = m.types.get(&rest[1]).cloned().unwrap_or(Ty::Float);
                    m.types.insert(
                        rest[0],
                        Ty::Vec {
                            elem: Box::new(elem),
                            n: rest[2] as u8,
                        },
                    );
                }
            }
            OP_TYPE_SAMPLED_IMAGE | OP_TYPE_IMAGE => {
                if let Some(id) = rest.first() {
                    m.types.insert(*id, Ty::Sampled);
                }
            }
            OP_TYPE_RUNTIME_ARRAY => {
                if rest.len() >= 2 {
                    let elem = m.types.get(&rest[1]).cloned().unwrap_or(Ty::Other);
                    let stride = array_stride.get(&rest[0]).copied().unwrap_or(16);
                    m.types.insert(
                        rest[0],
                        Ty::RuntimeArray {
                            elem: Box::new(elem),
                            stride,
                        },
                    );
                }
            }
            OP_TYPE_STRUCT => {
                if !rest.is_empty() {
                    let mut members = Vec::new();
                    for (mi, tid) in rest[1..].iter().enumerate() {
                        let ty = m.types.get(tid).cloned().unwrap_or(Ty::Other);
                        let offset = member_offsets.get(&(rest[0], mi as u32)).copied().unwrap_or(0);
                        members.push(Member { ty, offset });
                    }
                    m.types.insert(rest[0], Ty::Struct { members });
                }
            }
            OP_TYPE_POINTER => {
                if rest.len() >= 3 {
                    let pointee = m.types.get(&rest[2]).cloned().unwrap_or(Ty::Other);
                    m.types.insert(
                        rest[0],
                        Ty::Pointer {
                            storage: rest[1],
                            pointee: Box::new(pointee),
                        },
                    );
                }
            }
            OP_CONSTANT => {
                if rest.len() >= 3 {
                    let ty = m.types.get(&rest[0]).cloned().unwrap_or(Ty::Other);
                    let bits = rest[2];
                    let value = match ty {
                        Ty::Float => Value::Scalar {
                            bits,
                            float: true,
                            signed: true,
                        },
                        Ty::Int { signed } => Value::Scalar {
                            bits,
                            float: false,
                            signed,
                        },
                        _ => Value::Scalar {
                            bits,
                            float: false,
                            signed: false,
                        },
                    };
                    m.constants.insert(rest[1], value);
                }
            }
            OP_CONSTANT_COMPOSITE => {
                if rest.len() >= 2 {
                    let mut bits = [0u32; 4];
                    let mut n = 0u8;
                    let mut float = true;
                    let mut signed = true;
                    for cid in &rest[2..] {
                        if let Some(v) = m.constants.get(cid) {
                            match v {
                                Value::Scalar {
                                    bits: b,
                                    float: f,
                                    signed: s,
                                } => {
                                    if n < 4 {
                                        bits[n as usize] = *b;
                                    }
                                    float = *f;
                                    signed = *s;
                                    n += 1;
                                }
                                Value::Vec {
                                    bits: b,
                                    n: vn,
                                    float: f,
                                    signed: s,
                                } => {
                                    for k in 0..*vn as usize {
                                        if n < 4 {
                                            bits[n as usize] = b[k];
                                        }
                                        n += 1;
                                    }
                                    float = *f;
                                    signed = *s;
                                }
                                _ => {}
                            }
                        }
                    }
                    let id = rest[1];
                    let value = if n <= 1 {
                        Value::Scalar {
                            bits: bits[0],
                            float,
                            signed,
                        }
                    } else {
                        Value::Vec { bits, n, float, signed }
                    };
                    m.constants.insert(id, value);
                }
            }
            OP_VARIABLE => {
                if rest.len() >= 3 {
                    let id = rest[1];
                    let storage = rest[2];
                    let ty = m.types.get(&rest[0]).cloned().unwrap_or(Ty::Other);
                    m.vars.insert(
                        id,
                        Variable {
                            id,
                            storage,
                            ty,
                            location: locations.get(&id).copied().unwrap_or(-1),
                            binding: bindings.get(&id).copied().unwrap_or(-1),
                            set: sets.get(&id).copied().unwrap_or(-1),
                            builtin: builtins.get(&id).copied().unwrap_or(-1),
                        },
                    );
                }
            }
            OP_FUNCTION => {
                if let Some(f) = cur_func.take() {
                    m.funcs.push(f);
                }
                cur_func = Some(Func {
                    id: rest.get(1).copied().unwrap_or(0),
                    blocks: Vec::new(),
                });
            }
            OP_LABEL => {
                if let Some(b) = cur_block.take() {
                    if let Some(f) = cur_func.as_mut() {
                        f.blocks.push(b);
                    }
                }
                cur_block = Some(Block {
                    id: rest.first().copied().unwrap_or(0),
                    insts: Vec::new(),
                });
            }
            OP_FUNCTION_END => {
                if let Some(b) = cur_block.take() {
                    if let Some(f) = cur_func.as_mut() {
                        f.blocks.push(b);
                    }
                }
                if let Some(f) = cur_func.take() {
                    m.funcs.push(f);
                }
            }
            OP_PHI | OP_LOOP_MERGE | OP_BRANCH | OP_BRANCH_CONDITIONAL | OP_KILL => {
                m.has_control_flow = true;
                push_inst(&mut m, &mut cur_block, op, rest);
            }
            _ => {
                if cur_block.is_some() {
                    push_inst(&mut m, &mut cur_block, op, rest);
                }
            }
        }
    }
    if let Some(b) = cur_block.take() {
        if let Some(f) = cur_func.as_mut() {
            f.blocks.push(b);
        }
    }
    if let Some(f) = cur_func.take() {
        m.funcs.push(f);
    }
    // Decorations can appear before the variable. Re-apply in case the maps
    // were filled first (they are, in SPIR-V) — already applied at OpVariable.
    // A second pass covers the unusual decorate-after-variable case.
    for (id, var) in m.vars.iter_mut() {
        if let Some(v) = locations.get(id) {
            var.location = *v;
        }
        if let Some(v) = bindings.get(id) {
            var.binding = *v;
        }
        if let Some(v) = sets.get(id) {
            var.set = *v;
        }
        if let Some(v) = builtins.get(id) {
            var.builtin = *v;
        }
    }
    Ok(m)
}

fn push_inst(m: &mut Module, cur_block: &mut Option<Block>, op: u16, rest: &[u32]) {
    let (result, operands) = if has_result_and_type(op) {
        let result = rest.get(1).copied().unwrap_or(0);
        (result, rest.get(2..).unwrap_or(&[]).to_vec())
    } else if has_result_only(op) {
        let result = rest.first().copied().unwrap_or(0);
        (result, rest.get(1..).unwrap_or(&[]).to_vec())
    } else {
        (0, rest.to_vec())
    };
    let inst = Inst { op, result, operands };
    if result != 0 {
        m.defs.insert(result, inst.clone());
    }
    if let Some(b) = cur_block.as_mut() {
        b.insts.push(inst);
    }
}

fn has_result_and_type(op: u16) -> bool {
    matches!(
        op,
        OP_LOAD
            | OP_ACCESS_CHAIN
            | OP_COMPOSITE_CONSTRUCT
            | OP_COMPOSITE_EXTRACT
            | OP_IMAGE_SAMPLE_IMPLICIT_LOD
            | OP_FADD
            | OP_IADD
            | OP_SLESSTHAN
            | OP_PHI
    )
}

fn has_result_only(op: u16) -> bool {
    matches!(op, OP_LABEL)
}

fn words_str(words: &[u32]) -> String {
    let mut bytes = Vec::new();
    for w in words {
        bytes.extend_from_slice(&w.to_le_bytes());
    }
    let end = bytes.iter().position(|b| *b == 0).unwrap_or(bytes.len());
    String::from_utf8_lossy(&bytes[..end]).into_owned()
}

pub fn fixed_function(m: &Module) -> FfInfo {
    if m.has_control_flow || m.model != MODEL_FRAGMENT {
        return FfInfo::none();
    }
    let Some(func) = m.func() else {
        return FfInfo::none();
    };
    if func.blocks.len() != 1 {
        return FfInfo::none();
    }
    let ops: Vec<&Inst> = func.blocks[0]
        .insts
        .iter()
        .filter(|i| i.op != OP_RETURN && i.op != OP_LABEL)
        .collect();
    if ops.len() == 1 && ops[0].op == OP_STORE {
        let val = ops[0].operands.get(1).copied().unwrap_or(0);
        if let Some(Value::Vec { bits, n: 4, float: true, .. }) = m.constants.get(&val) {
            return FfInfo {
                kind: FfKind::Const,
                color: [
                    f32::from_bits(bits[0]),
                    f32::from_bits(bits[1]),
                    f32::from_bits(bits[2]),
                    f32::from_bits(bits[3]),
                ],
                uv_loc: 0,
                set: 0,
                binding: 0,
            };
        }
    }
    // load varying, load sampled image, sample, store
    if ops.len() == 4
        && ops[0].op == OP_LOAD
        && ops[1].op == OP_LOAD
        && ops[2].op == OP_IMAGE_SAMPLE_IMPLICIT_LOD
        && ops[3].op == OP_STORE
    {
        let var_id = ops[0].operands.first().copied().unwrap_or(0);
        let img_id = ops[1].operands.first().copied().unwrap_or(0);
        let uv_loc = m.vars.get(&var_id).map(|v| v.location).unwrap_or(0);
        let (set, binding) = m
            .vars
            .get(&img_id)
            .map(|v| (v.set.max(0) as u32, v.binding.max(0) as u32))
            .unwrap_or((0, 0));
        return FfInfo {
            kind: FfKind::Tex,
            color: [1.0; 4],
            uv_loc,
            set,
            binding,
        };
    }
    FfInfo::none()
}

pub fn execute(m: &Module, io: &mut Io) -> Result<(), String> {
    let func = m.func().ok_or("missing entry")?;
    if func.blocks.is_empty() {
        return Err("empty function".into());
    }
    let mut vals: Vec<Value> = vec![Value::Undef; m.bound.max(1)];
    for (id, v) in &m.constants {
        if (*id as usize) < vals.len() {
            vals[*id as usize] = v.clone();
        }
    }
    for (id, _) in &m.vars {
        if (*id as usize) < vals.len() {
            vals[*id as usize] = Value::Ptr { var: *id, offset: 0 };
        }
    }
    let mut block_id = func.blocks[0].id;
    let mut pred: u32 = 0;
    for _step in 0..200_000 {
        let block = func
            .blocks
            .iter()
            .find(|b| b.id == block_id)
            .ok_or("missing block")?;
        let mut phi_write = Vec::new();
        for inst in &block.insts {
            if inst.op != OP_PHI {
                continue;
            }
            let mut k = 0;
            while k + 1 < inst.operands.len() {
                let src = inst.operands[k];
                let parent = inst.operands[k + 1];
                if parent == pred {
                    let v = vals.get(src as usize).cloned().unwrap_or(Value::Undef);
                    phi_write.push((inst.result, v));
                    break;
                }
                k += 2;
            }
        }
        for (id, v) in phi_write {
            if (id as usize) < vals.len() {
                vals[id as usize] = v;
            }
        }
        let mut next: Option<u32> = None;
        for inst in &block.insts {
            match inst.op {
                OP_PHI | OP_LOOP_MERGE | OP_LABEL => {}
                OP_BRANCH => {
                    next = inst.operands.first().copied();
                    break;
                }
                OP_BRANCH_CONDITIONAL => {
                    let cond = vals.get(inst.operands[0] as usize);
                    let take = matches!(cond, Some(Value::Bool(true)));
                    let idx = if take { 1 } else { 2 };
                    next = inst.operands.get(idx).copied();
                    break;
                }
                OP_RETURN => return Ok(()),
                OP_KILL => return Ok(()),
                OP_STORE => {
                    let ptr = vals.get(inst.operands[0] as usize).cloned().unwrap_or(Value::Undef);
                    let val = vals.get(inst.operands[1] as usize).cloned().unwrap_or(Value::Undef);
                    do_store(m, io, &ptr, &val)?;
                }
                OP_LOAD => {
                    let ptr = vals.get(inst.operands[0] as usize).cloned().unwrap_or(Value::Undef);
                    let v = do_load(m, io, &ptr)?;
                    put(&mut vals, inst.result, v);
                }
                OP_ACCESS_CHAIN => {
                    let base = inst.operands.first().copied().unwrap_or(0);
                    let base_ptr = vals.get(base as usize).cloned().unwrap_or(Value::Ptr { var: base, offset: 0 });
                    let (var, mut offset) = match base_ptr {
                        Value::Ptr { var, offset } => (var, offset),
                        _ => (base, 0),
                    };
                    let var_info = m.vars.get(&var);
                    let mut ty = var_info.map(|v| pointee(&v.ty)).cloned();
                    for idx_id in inst.operands.iter().skip(1) {
                        let idx = vals.get(*idx_id as usize).map(|v| v.as_u32()).unwrap_or(0);
                        ty = match ty {
                            Some(Ty::Struct { members }) => {
                                let member = members.get(idx as usize);
                                if let Some(member) = member {
                                    offset += member.offset;
                                    Some(member.ty.clone())
                                } else {
                                    None
                                }
                            }
                            Some(Ty::RuntimeArray { elem, stride }) => {
                                offset += idx * stride;
                                Some(*elem)
                            }
                            other => other,
                        };
                    }
                    put(&mut vals, inst.result, Value::Ptr { var, offset });
                }
                OP_COMPOSITE_CONSTRUCT => {
                    let mut bits = [0u32; 4];
                    let mut n = 0u8;
                    let mut float = true;
                    let mut signed = true;
                    for cid in &inst.operands {
                        let v = vals.get(*cid as usize).cloned().unwrap_or(Value::Undef);
                        match v {
                            Value::Scalar {
                                bits: b,
                                float: f,
                                signed: s,
                            } => {
                                if n < 4 {
                                    bits[n as usize] = b;
                                }
                                float = f;
                                signed = s;
                                n += 1;
                            }
                            Value::Vec {
                                bits: b,
                                n: vn,
                                float: f,
                                signed: s,
                            } => {
                                for k in 0..vn as usize {
                                    if n < 4 {
                                        bits[n as usize] = b[k];
                                    }
                                    n += 1;
                                }
                                float = f;
                                signed = s;
                            }
                            _ => {}
                        }
                    }
                    let value = if n <= 1 {
                        Value::Scalar {
                            bits: bits[0],
                            float,
                            signed,
                        }
                    } else {
                        Value::Vec { bits, n, float, signed }
                    };
                    put(&mut vals, inst.result, value);
                }
                OP_COMPOSITE_EXTRACT => {
                    let src = vals.get(inst.operands[0] as usize).cloned().unwrap_or(Value::Undef);
                    let idx = inst.operands.get(1).copied().unwrap_or(0) as usize;
                    put(&mut vals, inst.result, src.component(idx));
                }
                OP_FADD => {
                    let a = vals.get(inst.operands[0] as usize).map(|v| v.as_f32()).unwrap_or(0.0);
                    let b = vals.get(inst.operands[1] as usize).map(|v| v.as_f32()).unwrap_or(0.0);
                    put(&mut vals, inst.result, Value::f32s(&[a + b]));
                }
                OP_IADD => {
                    let a = vals.get(inst.operands[0] as usize).map(|v| v.as_i32()).unwrap_or(0);
                    let b = vals.get(inst.operands[1] as usize).map(|v| v.as_i32()).unwrap_or(0);
                    put(
                        &mut vals,
                        inst.result,
                        Value::Scalar {
                            bits: (a.wrapping_add(b)) as u32,
                            float: false,
                            signed: true,
                        },
                    );
                }
                OP_SLESSTHAN => {
                    let a = vals.get(inst.operands[0] as usize).map(|v| v.as_i32()).unwrap_or(0);
                    let b = vals.get(inst.operands[1] as usize).map(|v| v.as_i32()).unwrap_or(0);
                    put(&mut vals, inst.result, Value::Bool(a < b));
                }
                OP_IMAGE_SAMPLE_IMPLICIT_LOD => {
                    let img = vals.get(inst.operands[0] as usize).cloned().unwrap_or(Value::Undef);
                    let coord = vals.get(inst.operands[1] as usize).cloned().unwrap_or(Value::Undef);
                    let (set, binding) = match img {
                        Value::Sampled { set, binding } => (set, binding),
                        _ => (0, 0),
                    };
                    let c = coord.as_vec4();
                    let color = sample_image(io, set, binding, c[0], c[1]);
                    put(&mut vals, inst.result, Value::f32s(&color));
                }
                other => {
                    return Err(format!("unsupported opcode {other}"));
                }
            }
        }
        if let Some(n) = next {
            pred = block_id;
            block_id = n;
        } else {
            return Ok(());
        }
    }
    Err("SPIR-V step cap".into())
}

fn put(vals: &mut [Value], id: u32, v: Value) {
    if (id as usize) < vals.len() {
        vals[id as usize] = v;
    }
}

fn pointee(ty: &Ty) -> &Ty {
    match ty {
        Ty::Pointer { pointee, .. } => pointee,
        other => other,
    }
}

fn do_load(m: &Module, io: &Io, ptr: &Value) -> Result<Value, String> {
    let Value::Ptr { var, offset } = ptr else {
        return Ok(Value::Undef);
    };
    let Some(var) = m.vars.get(var) else {
        return Ok(Value::Undef);
    };
    match var.storage {
        SC_INPUT => {
            if var.builtin == BUILTIN_GLOBAL_INVOCATION_ID as i32 {
                return Ok(Value::Vec {
                    bits: [io.global_id[0], io.global_id[1], io.global_id[2], 0],
                    n: 3,
                    float: false,
                    signed: false,
                });
            }
            if var.builtin == BUILTIN_FRAGCOORD as i32 {
                return Ok(Value::f32s(&io.frag_coord));
            }
            if var.builtin == BUILTIN_VERTEX_INDEX as i32 || var.builtin == BUILTIN_VERTEX_ID as i32 {
                return Ok(Value::Scalar {
                    bits: io.vertex_index as u32,
                    float: false,
                    signed: true,
                });
            }
            if var.builtin == BUILTIN_INSTANCE_INDEX as i32 {
                return Ok(Value::Scalar {
                    bits: io.instance_index as u32,
                    float: false,
                    signed: true,
                });
            }
            let loc = var.location.max(0) as usize;
            let attr = io.attrs.get(loc).copied().unwrap_or([0.0, 0.0, 0.0, 1.0]);
            let n = match pointee(&var.ty) {
                Ty::Vec { n, .. } => *n,
                _ => 4,
            };
            let mut bits = [0u32; 4];
            for i in 0..n as usize {
                bits[i] = attr[i].to_bits();
            }
            if n == 1 {
                Ok(Value::Scalar {
                    bits: bits[0],
                    float: true,
                    signed: true,
                })
            } else {
                Ok(Value::Vec {
                    bits,
                    n,
                    float: true,
                    signed: true,
                })
            }
        }
        SC_UNIFORM => {
            let set = var.set.max(0) as u32;
            let binding = var.binding.max(0) as u32;
            let bytes = io
                .ubos
                .iter()
                .find(|u| u.0 == set && u.1 == binding)
                .map(|u| u.2)
                .unwrap_or(&[]);
            let off = *offset as usize;
            let mut bits = [0u32; 4];
            for i in 0..4 {
                let s = off + i * 4;
                if s + 4 <= bytes.len() {
                    bits[i] = u32::from_le_bytes(bytes[s..s + 4].try_into().unwrap());
                }
            }
            Ok(Value::Vec {
                bits,
                n: 4,
                float: true,
                signed: true,
            })
        }
        SC_UNIFORM_CONSTANT => Ok(Value::Sampled {
            set: var.set.max(0) as u32,
            binding: var.binding.max(0) as u32,
        }),
        _ => Ok(Value::Undef),
    }
}

fn do_store(m: &Module, io: &mut Io, ptr: &Value, val: &Value) -> Result<(), String> {
    let Value::Ptr { var, offset } = ptr else {
        return Ok(());
    };
    let Some(var) = m.vars.get(var).cloned() else {
        return Ok(());
    };
    match var.storage {
        SC_OUTPUT => {
            if var.builtin == BUILTIN_POSITION as i32 {
                io.position = val.as_vec4();
            } else {
                let loc = var.location.max(0) as u32;
                io.outputs.push((loc, val.as_vec4()));
            }
        }
        SC_STORAGE_BUFFER => {
            let set = var.set.max(0) as u32;
            let binding = var.binding.max(0) as u32;
            if set == io.ssbo_set && binding == io.ssbo_binding {
                let bytes = val.to_le_bytes();
                let off = *offset as usize;
                if off + bytes.len() <= io.ssbo.len() {
                    io.ssbo[off..off + bytes.len()].copy_from_slice(&bytes);
                }
            }
        }
        _ => {}
    }
    Ok(())
}

fn sample_image(io: &Io, set: u32, binding: u32, u: f32, v: f32) -> [f32; 4] {
    let Some(img) = io.images.iter().find(|i| i.set == set && i.binding == binding) else {
        return [0.0, 0.0, 0.0, 1.0];
    };
    if img.w == 0 || img.h == 0 || img.rgba.len() < 4 {
        return [0.0, 0.0, 0.0, 1.0];
    }
    let (u, v) = if img.clamp {
        (u.clamp(0.0, 0.999999), v.clamp(0.0, 0.999999))
    } else {
        (u.rem_euclid(1.0), v.rem_euclid(1.0))
    };
    let x = ((u * img.w as f32).floor() as i32).clamp(0, img.w as i32 - 1) as u32;
    let y = ((v * img.h as f32).floor() as i32).clamp(0, img.h as i32 - 1) as u32;
    let i = ((y * img.w + x) * 4) as usize;
    if i + 4 > img.rgba.len() {
        return [0.0, 0.0, 0.0, 1.0];
    }
    let p = &img.rgba[i..i + 4];
    [
        p[0] as f32 / 255.0,
        p[1] as f32 / 255.0,
        p[2] as f32 / 255.0,
        p[3] as f32 / 255.0,
    ]
}

pub fn input_locations(m: &Module) -> Vec<(u32, u8)> {
    let mut out = Vec::new();
    for v in m.vars.values() {
        if v.storage == SC_INPUT && v.builtin < 0 && v.location >= 0 {
            let n = match pointee(&v.ty) {
                Ty::Vec { n, .. } => *n,
                Ty::Float => 1,
                _ => 4,
            };
            out.push((v.location as u32, n));
        }
    }
    out.sort_by_key(|p| p.0);
    out
}

pub fn output_locations(m: &Module) -> Vec<u32> {
    let mut out = Vec::new();
    for v in m.vars.values() {
        if v.storage == SC_OUTPUT && v.builtin < 0 && v.location >= 0 {
            out.push(v.location as u32);
        }
    }
    out.sort();
    out.dedup();
    out
}
