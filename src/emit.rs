//! Straight-line SPIR-V to HLSL or GLSL. Control flow is rejected so the
//! CPU interpreter runs those shaders. Fixed-function OpenGL does not get a
//! shader from here.

use crate::spirv::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Dialect {
    Hlsl5,
    Hlsl3,
    Glsl330,
    Glsl120,
    Fixed,
}

#[derive(Clone, Debug)]
pub struct Emitted {
    pub vs: String,
    pub fs: String,
    pub cs: String,
    /// HLSL register / GLSL block binding. -1 if this stage does not use a UBO.
    pub ubo_slot: i32,
    pub ubo_in_vs: bool,
    pub ubo_in_fs: bool,
    pub tex_slot: i32,
    pub ssbo_slot: i32,
}

pub fn emit_graphics(vs: &Module, fs: &Module, dialect: Dialect) -> Result<Emitted, String> {
    if dialect == Dialect::Fixed {
        return Err("fixed function".into());
    }
    if vs.has_control_flow || fs.has_control_flow {
        return Err("control flow".into());
    }
    let vs_src = emit_vs(vs, dialect)?;
    let fs_src = emit_fs(fs, dialect)?;
    let ubo_vs = uses_ubo(vs);
    let ubo_fs = uses_ubo(fs);
    let tex = uses_tex(fs);
    Ok(Emitted {
        vs: vs_src,
        fs: fs_src,
        cs: String::new(),
        ubo_slot: if ubo_vs || ubo_fs { ubo_register(dialect) } else { -1 },
        ubo_in_vs: ubo_vs,
        ubo_in_fs: ubo_fs,
        tex_slot: if tex { 0 } else { -1 },
        ssbo_slot: -1,
    })
}

pub fn emit_compute(cs: &Module, dialect: Dialect) -> Result<Emitted, String> {
    if dialect != Dialect::Hlsl5 {
        return Err("compute is software".into());
    }
    if cs.has_control_flow {
        return Err("control flow".into());
    }
    let src = emit_cs(cs)?;
    Ok(Emitted {
        vs: String::new(),
        fs: String::new(),
        cs: src,
        ubo_slot: -1,
        ubo_in_vs: false,
        ubo_in_fs: false,
        tex_slot: -1,
        ssbo_slot: 0,
    })
}

fn ubo_register(dialect: Dialect) -> i32 {
    if dialect == Dialect::Hlsl3 { 10 } else { 1 }
}

fn uses_ubo(m: &Module) -> bool {
    m.vars.values().any(|v| v.storage == SC_UNIFORM)
}

fn uses_tex(m: &Module) -> bool {
    m.vars.values().any(|v| v.storage == SC_UNIFORM_CONSTANT)
}

fn glsl(dialect: Dialect) -> bool {
    matches!(dialect, Dialect::Glsl330 | Dialect::Glsl120)
}

fn flip_y(dialect: Dialect) -> bool {
    glsl(dialect)
}

fn fmt_f(bits: u32) -> String {
    let f = f32::from_bits(bits);
    format!("{f:.8}")
}

fn ctor(n: u8, dialect: Dialect) -> &'static str {
    match (glsl(dialect), n) {
        (false, 2) => "float2",
        (false, 3) => "float3",
        (false, 4) => "float4",
        (true, 2) => "vec2",
        (true, 3) => "vec3",
        (true, 4) => "vec4",
        _ => "float",
    }
}

fn expr(m: &Module, id: u32, dialect: Dialect) -> Result<String, String> {
    if let Some(v) = m.constants.get(&id) {
        return Ok(match v {
            Value::Scalar { bits, float: true, .. } => fmt_f(*bits),
            Value::Scalar { bits, signed: true, .. } => format!("{}", *bits as i32),
            Value::Scalar { bits, .. } => format!("{bits}u"),
            Value::Vec { bits, n, float: true, .. } => {
                let args: Vec<String> = (0..*n as usize).map(|i| fmt_f(bits[i])).collect();
                format!("{}({})", ctor(*n, dialect), args.join(", "))
            }
            Value::Vec { bits, n, .. } => {
                let args: Vec<String> = (0..*n as usize).map(|i| format!("{}", bits[i])).collect();
                format!("{}({})", ctor(*n, dialect), args.join(", "))
            }
            _ => return Err("bad constant".into()),
        });
    }
    let Some(inst) = m.defs.get(&id) else {
        return Err(format!("no definition for id {id}"));
    };
    match inst.op {
        OP_LOAD => load_expr(m, inst, dialect),
        OP_ACCESS_CHAIN => chain_expr(m, inst, dialect),
        OP_COMPOSITE_CONSTRUCT => {
            let mut args = Vec::new();
            let mut n = 0u8;
            for cid in &inst.operands {
                let e = expr(m, *cid, dialect)?;
                n += comp_count(m, *cid);
                args.push(e);
            }
            if n <= 1 {
                Ok(args.into_iter().next().unwrap_or_else(|| "0.0".into()))
            } else {
                Ok(format!("{}({})", ctor(n.min(4), dialect), args.join(", ")))
            }
        }
        OP_COMPOSITE_EXTRACT => {
            let base = expr(m, inst.operands[0], dialect)?;
            let idx = inst.operands.get(1).copied().unwrap_or(0) as usize;
            let sw = ["x", "y", "z", "w"];
            Ok(format!("({base}).{}", sw.get(idx).copied().unwrap_or("x")))
        }
        OP_FADD => {
            let a = expr(m, inst.operands[0], dialect)?;
            let b = expr(m, inst.operands[1], dialect)?;
            Ok(format!("(({a}) + ({b}))"))
        }
        OP_IMAGE_SAMPLE_IMPLICIT_LOD => {
            let var = sampled_var(m, inst.operands[0])?;
            let slot = var.binding.max(0);
            let coord = expr(m, inst.operands[1], dialect)?;
            Ok(sample_call(slot, &coord, dialect))
        }
        other => Err(format!("cannot emit opcode {other}")),
    }
}

fn comp_count(m: &Module, id: u32) -> u8 {
    if let Some(Value::Vec { n, .. }) = m.constants.get(&id) {
        return *n;
    }
    if let Some(Value::Scalar { .. }) = m.constants.get(&id) {
        return 1;
    }
    if let Some(inst) = m.defs.get(&id) {
        if inst.op == OP_COMPOSITE_CONSTRUCT {
            return inst.operands.iter().map(|c| comp_count(m, *c)).sum();
        }
        if inst.op == OP_LOAD {
            if let Some(var) = ptr_var(m, inst.operands.first().copied().unwrap_or(0)) {
                let ty = match &var.ty {
                    Ty::Pointer { pointee, .. } => pointee.as_ref(),
                    other => other,
                };
                if let Ty::Vec { n, .. } = ty {
                    return *n;
                }
            }
        }
    }
    1
}

fn sampled_var(m: &Module, load_id: u32) -> Result<&Variable, String> {
    let load = m.defs.get(&load_id).ok_or("sample image is not a load")?;
    let var_id = load.operands.first().copied().unwrap_or(0);
    m.vars.get(&var_id).ok_or_else(|| "sample variable missing".into())
}

fn ptr_var(m: &Module, id: u32) -> Option<&Variable> {
    if let Some(v) = m.vars.get(&id) {
        return Some(v);
    }
    let inst = m.defs.get(&id)?;
    if inst.op == OP_ACCESS_CHAIN {
        return m.vars.get(&inst.operands.first().copied().unwrap_or(0));
    }
    None
}

fn load_expr(m: &Module, inst: &Inst, dialect: Dialect) -> Result<String, String> {
    let pid = inst.operands.first().copied().unwrap_or(0);
    if let Some(def) = m.defs.get(&pid) {
        if def.op == OP_ACCESS_CHAIN {
            return chain_expr(m, def, dialect);
        }
    }
    let var = m.vars.get(&pid).ok_or("load of unknown pointer")?;
    if var.builtin == BUILTIN_GLOBAL_INVOCATION_ID as i32 {
        return Ok("gid".into());
    }
    if var.builtin >= 0 {
        return Err(format!("builtin {} is not emitted", var.builtin));
    }
    let loc = var.location.max(0);
    Ok(match (var.storage, dialect) {
        (SC_INPUT, Dialect::Hlsl5) if m.model == MODEL_VERTEX => format!("input.a{loc}"),
        (SC_INPUT, Dialect::Hlsl5) => format!("pin.vin{loc}"),
        (SC_INPUT, Dialect::Hlsl3) if m.model == MODEL_VERTEX => format!("a{loc}"),
        (SC_INPUT, Dialect::Hlsl3) => format!("vin{loc}"),
        (SC_INPUT, _) => {
            if m.model == MODEL_VERTEX {
                format!("a{loc}")
            } else {
                format!("vin_t{loc}")
            }
        }
        _ => return Err("unsupported load".into()),
    })
}

fn chain_expr(m: &Module, inst: &Inst, dialect: Dialect) -> Result<String, String> {
    let var = m
        .vars
        .get(&inst.operands.first().copied().unwrap_or(0))
        .ok_or("access chain base")?;
    if var.storage == SC_UNIFORM {
        let ni = 1 + var.binding.max(0);
        let idx = if inst.operands.len() > 1 {
            expr(m, inst.operands[1], dialect)?
        } else {
            "0".into()
        };
        return Ok(format!("U{ni}[{idx}]"));
    }
    Err("access chain is not a uniform".into())
}

fn sample_call(slot: i32, coord: &str, dialect: Dialect) -> String {
    match dialect {
        Dialect::Hlsl5 => format!("tex{slot}.Sample(samp{slot}, ({coord}).xy)"),
        Dialect::Hlsl3 => format!("tex2D(tex{slot}, ({coord}).xy)"),
        Dialect::Glsl330 if flip_y(dialect) => {
            format!("texture(tex{slot}, vec2(({coord}).x, 1.0 - ({coord}).y))")
        }
        Dialect::Glsl120 => {
            format!("texture2D(tex{slot}, vec2(({coord}).x, 1.0 - ({coord}).y))")
        }
        Dialect::Glsl330 => format!("texture(tex{slot}, ({coord}).xy)"),
        Dialect::Fixed => String::new(),
    }
}

fn attr_ty(n: u8, dialect: Dialect) -> String {
    match (glsl(dialect), n) {
        (true, 1) => "float".into(),
        (true, 2) => "vec2".into(),
        (true, 3) => "vec3".into(),
        (true, _) => "vec4".into(),
        (false, 1) => "float".into(),
        (false, 2) => "float2".into(),
        (false, 3) => "float3".into(),
        (false, _) => "float4".into(),
    }
}

fn entry_block(m: &Module) -> Result<&Block, String> {
    let f = m.func().ok_or("no entry function")?;
    f.blocks.first().ok_or_else(|| "no block".into())
}

fn emit_vs(m: &Module, dialect: Dialect) -> Result<String, String> {
    let inputs = input_locations(m);
    let outputs = output_locations(m);
    let block = entry_block(m)?;
    let mut body = String::new();
    for inst in &block.insts {
        if inst.op != OP_STORE {
            continue;
        }
        let ptr = inst.operands[0];
        let val = expr(m, inst.operands[1], dialect)?;
        let var = ptr_var(m, ptr).ok_or("store pointer")?;
        if var.builtin == BUILTIN_POSITION as i32 {
            body.push_str(&pos_store(&val, dialect));
        } else if var.location >= 0 {
            let loc = var.location;
            match dialect {
                Dialect::Hlsl5 | Dialect::Hlsl3 => {
                    body.push_str(&format!("  o.vin{loc} = {val};\n"));
                }
                _ => body.push_str(&format!("  vin_t{loc} = {val};\n")),
            }
        }
    }
    Ok(match dialect {
        Dialect::Hlsl5 => {
            let mut s = String::from("struct VSIn {\n");
            for (loc, n) in &inputs {
                s.push_str(&format!("  {} a{loc} : TEXCOORD{loc};\n", attr_ty(*n, dialect)));
            }
            if inputs.is_empty() {
                s.push_str("  float4 a0 : TEXCOORD0;\n");
            }
            s.push_str("};\nstruct VSOut {\n  float4 pos : SV_Position;\n");
            for loc in &outputs {
                s.push_str(&format!("  float4 vin{loc} : TEXCOORD{loc};\n"));
            }
            s.push_str("};\nVSOut main(VSIn input) {\n  VSOut o;\n");
            s.push_str("  o.pos = float4(0,0,0,1);\n");
            for loc in &outputs {
                s.push_str(&format!("  o.vin{loc} = float4(0,0,0,1);\n"));
            }
            s.push_str(&body);
            s.push_str("  return o;\n}\n");
            s
        }
        Dialect::Hlsl3 => {
            let mut s = String::from("struct VSOut {\n  float4 pos : POSITION;\n");
            for loc in &outputs {
                s.push_str(&format!("  float4 vin{loc} : TEXCOORD{loc};\n"));
            }
            s.push_str("};\nVSOut main(");
            if inputs.is_empty() {
                s.push_str("float4 a0 : POSITION0");
            } else {
                for (i, (loc, n)) in inputs.iter().enumerate() {
                    if i > 0 {
                        s.push_str(", ");
                    }
                    let sem = if *loc == 0 {
                        format!("POSITION{loc}")
                    } else {
                        format!("TEXCOORD{loc}")
                    };
                    s.push_str(&format!("{} a{loc} : {sem}", attr_ty(*n, dialect)));
                }
            }
            s.push_str(") {\n  VSOut o;\n  o.pos = float4(0,0,0,1);\n");
            for loc in &outputs {
                s.push_str(&format!("  o.vin{loc} = float4(0,0,0,1);\n"));
            }
            s.push_str(&body);
            s.push_str("  return o;\n}\n");
            s
        }
        Dialect::Glsl330 => {
            let mut s = String::from("#version 330\n");
            for (loc, n) in &inputs {
                s.push_str(&format!("layout(location = {loc}) in {} a{loc};\n", attr_ty(*n, dialect)));
            }
            for loc in &outputs {
                s.push_str(&format!("out vec4 vin_t{loc};\n"));
            }
            s.push_str("void main() {\n  gl_Position = vec4(0.0, 0.0, 0.0, 1.0);\n");
            s.push_str(&body);
            s.push_str("}\n");
            s
        }
        Dialect::Glsl120 => {
            let mut s = String::from("#version 120\n");
            for (loc, n) in &inputs {
                s.push_str(&format!("attribute {} a{loc};\n", attr_ty(*n, dialect)));
            }
            for loc in &outputs {
                s.push_str(&format!("varying vec4 vin_t{loc};\n"));
            }
            s.push_str("void main() {\n  gl_Position = vec4(0.0, 0.0, 0.0, 1.0);\n");
            s.push_str(&body);
            s.push_str("}\n");
            s
        }
        Dialect::Fixed => String::new(),
    })
}

fn pos_store(val: &str, dialect: Dialect) -> String {
    match dialect {
        Dialect::Hlsl5 | Dialect::Hlsl3 => {
            format!("  {{\n    float4 _p = {val};\n    _p.y = -_p.y;\n    o.pos = _p;\n  }}\n")
        }
        _ => format!(
            "  {{\n    vec4 _p = {val};\n    _p.y = -_p.y;\n    _p.z = _p.z * 2.0 - _p.w;\n    gl_Position = _p;\n  }}\n"
        ),
    }
}

fn emit_fs(m: &Module, dialect: Dialect) -> Result<String, String> {
    let inputs = input_locations(m);
    let block = entry_block(m)?;
    let mut color = String::from(if glsl(dialect) { "vec4(0.0, 0.0, 0.0, 1.0)" } else { "float4(0,0,0,1)" });
    for inst in &block.insts {
        if inst.op == OP_STORE {
            let var = ptr_var(m, inst.operands[0]);
            if var.map(|v| v.storage == SC_OUTPUT).unwrap_or(false) {
                color = expr(m, inst.operands[1], dialect)?;
            }
        }
    }
    let ubo = uses_ubo(m);
    let tex = uses_tex(m);
    let ubo_decl = if ubo {
        match dialect {
            Dialect::Hlsl5 => "cbuffer C1 : register(b1) { float4 U1[1]; };\n".to_string(),
            Dialect::Hlsl3 => "float4 U1[1] : register(c10);\n".to_string(),
            Dialect::Glsl330 => "layout(std140) uniform C1 { vec4 U1[1]; };\n".to_string(),
            Dialect::Glsl120 => "uniform vec4 U1[1];\n".to_string(),
            Dialect::Fixed => String::new(),
        }
    } else {
        String::new()
    };
    let tex_decl = if tex {
        match dialect {
            Dialect::Hlsl5 => "Texture2D tex0 : register(t0);\nSamplerState samp0 : register(s0);\n".to_string(),
            Dialect::Hlsl3 => "sampler2D tex0 : register(s0);\n".to_string(),
            Dialect::Glsl330 | Dialect::Glsl120 => "uniform sampler2D tex0;\n".to_string(),
            Dialect::Fixed => String::new(),
        }
    } else {
        String::new()
    };
    Ok(match dialect {
        Dialect::Hlsl5 => {
            let mut s = String::new();
            s.push_str(&ubo_decl);
            s.push_str(&tex_decl);
            s.push_str("struct PSIn {\n  float4 pos : SV_Position;\n");
            for (loc, _) in &inputs {
                s.push_str(&format!("  float4 vin{loc} : TEXCOORD{loc};\n"));
            }
            s.push_str("};\nfloat4 main(PSIn pin) : SV_Target {\n");
            s.push_str(&format!("  return {color};\n}}\n"));
            s
        }
        Dialect::Hlsl3 => {
            let mut s = String::new();
            s.push_str(&ubo_decl);
            s.push_str(&tex_decl);
            s.push_str("float4 main(");
            if inputs.is_empty() {
                s.push_str("");
            } else {
                for (i, (loc, _)) in inputs.iter().enumerate() {
                    if i > 0 {
                        s.push_str(", ");
                    }
                    s.push_str(&format!("float4 vin{loc} : TEXCOORD{loc}"));
                }
            }
            s.push_str(") : COLOR0 {\n");
            s.push_str(&format!("  return {color};\n}}\n"));
            s
        }
        Dialect::Glsl330 => {
            let mut s = String::from("#version 330\n");
            s.push_str(&ubo_decl);
            s.push_str(&tex_decl);
            for (loc, _) in &inputs {
                s.push_str(&format!("in vec4 vin_t{loc};\n"));
            }
            s.push_str("out vec4 color0_out;\nvoid main() {\n");
            s.push_str(&format!("  color0_out = {color};\n}}\n"));
            s
        }
        Dialect::Glsl120 => {
            let mut s = String::from("#version 120\n");
            s.push_str(&ubo_decl);
            s.push_str(&tex_decl);
            for (loc, _) in &inputs {
                s.push_str(&format!("varying vec4 vin_t{loc};\n"));
            }
            s.push_str("void main() {\n");
            s.push_str(&format!("  gl_FragColor = {color};\n}}\n"));
            s
        }
        Dialect::Fixed => String::new(),
    })
}

fn emit_cs(m: &Module) -> Result<String, String> {
    let block = entry_block(m)?;
    let mut body = String::new();
    for inst in &block.insts {
        if inst.op != OP_STORE {
            continue;
        }
        let ptr_id = inst.operands[0];
        let chain = m.defs.get(&ptr_id).ok_or("compute store pointer")?;
        if chain.op != OP_ACCESS_CHAIN {
            return Err("compute store is not an access chain".into());
        }
        let var = m
            .vars
            .get(&chain.operands[0])
            .ok_or("compute ssbo")?;
        if var.storage != SC_STORAGE_BUFFER {
            return Err("compute store target is not a storage buffer".into());
        }
        let index = chain.operands.get(2).copied().ok_or("ssbo index")?;
        let idx = expr(m, index, Dialect::Hlsl5)?;
        let val = expr(m, inst.operands[1], Dialect::Hlsl5)?;
        body.push_str(&format!("  ssbo0[{idx}].v = {val};\n"));
    }
    let [lx, ly, lz] = m.local_size;
    Ok(format!(
        "struct Elem {{ float4 v; }};\nRWStructuredBuffer<Elem> ssbo0 : register(u0);\n[numthreads({lx}, {ly}, {lz})]\nvoid main(uint3 gid : SV_DispatchThreadID) {{\n{body}}}\n"
    ))
}
