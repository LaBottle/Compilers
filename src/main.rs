include!("lexical_analysis.rs");
use std::collections::HashMap;
use std::error::Error;
use std::fmt::Display;
use std::mem::swap;
use std::result;
use std::sync::Mutex;
// use std::io::Result;

type Result<T> = result::Result<T, StandardError>;

#[derive(Debug)]
struct StandardError {
    what: String,
}

impl Display for StandardError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Error: \n{}", self.what)
    }
}

impl Error for StandardError {}

impl StandardError {
    pub fn new(what: String) -> Self {
        StandardError { what }
    }
}

#[derive(Clone, Debug)]
struct Quaternion {
    pub op: String,
    pub arg1: String,
    pub arg2: String,
    pub result: String,
}

#[derive(Clone, Debug)]
struct Var {
    pub name: String,
    pub ty: String,
}

impl Var {
    pub unsafe fn new(name: String, ty: String) -> Result<Self> {
        for i in params.iter().chain(vars.iter()).chain(global_vars.iter()) {
            if i.name == name {
                return Err(StandardError::new(format!("`{}` has been defined", name)));
            }
        }
        Ok(Var { name, ty })
    }
}

#[derive(Default)]
struct GenTemp {
    n: usize,
}

impl GenTemp {
    // fn new() -> Self {
    //     GenTemp { n: 0 }
    // }
    fn contains(&self, s: &str) -> bool {
        s.as_bytes()[0] == b'@'
    }
    fn gen(&mut self) -> String {
        self.n += 1;
        format!("@t{}", self.n)
    }
    fn reset(&mut self) {
        self.n = 0;
    }
}

static mut idx: usize = 0;
static mut words: Vec<Word> = Vec::new();
static mut global_vars: Vec<Var> = Vec::new();
static mut vars: Vec<Var> = Vec::new();
static mut params: Vec<Var> = Vec::new();
static mut temp_gen: GenTemp = GenTemp { n: 0 };
static mut quaternions: Vec<Quaternion> = Vec::new();
static mut proc_list: Vec<(String, Vec<Quaternion>, Vec<Var>, Vec<Var>)> = Vec::new();
fn main() {
    unsafe {
        words = lexical_analysis("resource/e5.txt");
        syntax_analysis();
    }
}

pub unsafe fn syntax_analysis() {
    if let Err(e) = program() {
        println!("{}", e)
    };
    if !global_vars.is_empty() {
        println!("global:");
        for i in global_vars.iter() {
            println!("Name: {}, Type: {}", i.name, i.ty);
        }
    }

    for proc in proc_list.iter() {
        println!("procedure {}:", proc.0);
        if !proc.2.is_empty() {
            println!("params:");
            for i in proc.2.iter() {
                println!("Name: {}, Type: {}", i.name, i.ty);
            }
        }
        if !proc.3.is_empty() {
            println!("vars:");
            for i in proc.3.iter() {
                println!("Name: {}, Type: {}", i.name, i.ty);
            }
        }
        if !proc.1.is_empty() {
            println!("quaternions:");
            for index in 0..proc.1.len() {
                let i = proc.1.get_unchecked(index);
                println!(
                    "{}: ({}, {}, {}, {})",
                    index, i.op, i.arg1, i.arg2, i.result
                );
            }
        }
    }
}

unsafe fn program() -> Result<String> {
    program_head()?;
    var_decpart()?;
    program_body()?;
    if idx != words.len() {
        return Err(StandardError::new(format!(
            "Analysis has ended in `line {}, colume {}`.
But there are statements that have not been analyzed.
Please check for syntax errors.",
            words[idx].row, words[idx].col
        )));
    }
    Ok(String::new())
}

unsafe fn program_head() -> Result<String> {
    terminator(Type::Keyword, Some("program"))?;
    terminator(Type::Identifier, None)?;

    Ok(String::new())
}
unsafe fn var_decpart() -> Result<String> {
    if let Ok(_) = terminator(Type::Keyword, Some("var")) {
        var_dec_list()?;
    }

    Ok(String::new())
}
unsafe fn var_dec_list() -> Result<String> {
    loop {
        let i = idx;
        if let Err(e) = var_id_list() {
            if i != idx {
                return Err(e);
            } else {
                break;
            }
        };
    }

    Ok(String::new())
}

unsafe fn var_id_list() -> Result<String> {
    let ty = type_name()?;
    loop {
        let name = terminator(Type::Identifier, None)?;
        vars.push(Var::new(name, ty.clone())?);
        if let Err(_) = terminator(Type::Separator, Some(",")) {
            break;
        }
    }

    // if unmatched, output error message and continue.
    if let Err(e) = terminator(Type::Separator, Some(";")) {
        println!("{}", e)
    };

    Ok(String::new())
}

unsafe fn type_name() -> Result<String> {
    let ty = multi_terminator(Type::Keyword, &["integer", "float"])?;

    Ok(ty)
}

unsafe fn program_body() -> Result<String> {
    swap(&mut global_vars, &mut vars);
    let mut i = idx;
    match proc_dec() {
        Err(e) => {
            if i != idx {
                return Err(e);
            }
        }
        Ok(_) => loop {
            i = idx;
            if let Err(e) = proc_dec() {
                if i != idx {
                    return Err(e);
                } else {
                    break;
                }
            }
        },
    }

    Ok(String::new())
}

unsafe fn proc_dec() -> Result<String> {
    terminator(Type::Keyword, Some("procedure"))?;
    let name = terminator(Type::Identifier, None)?;

    global_vars.push(Var::new(name.clone(), "procedure".to_string())?);
    proc_list.push((name, Vec::new(), Vec::new(), Vec::new()));

    terminator(Type::Separator, Some("("))?;
    param_list()?;
    terminator(Type::Separator, Some(")"))?;
    // if unmatched, output error message and continue.
    if let Err(e) = terminator(Type::Separator, Some(";")) {
        println!("{}", e)
    };
    var_decpart()?;
    proc_body()?;

    swap(&mut proc_list.last_mut().unwrap().1, &mut quaternions);
    swap(&mut proc_list.last_mut().unwrap().2, &mut params);
    swap(&mut proc_list.last_mut().unwrap().3, &mut vars);
    temp_gen.reset();

    Ok(String::new())
}

unsafe fn param_list() -> Result<String> {
    let i = idx;
    match param() {
        Err(e) => {
            if i != idx {
                return Err(e);
            }
        }
        Ok(_) => loop {
            if let Err(_) = terminator(Type::Separator, Some(";")) {
                break;
            }
            param()?;
        },
    }
    Ok(String::new())
}

unsafe fn param() -> Result<String> {
    let ty = type_name()?;
    loop {
        let name = terminator(Type::Identifier, None)?;
        params.push(Var::new(name, ty.clone())?);
        if let Err(_) = terminator(Type::Separator, Some(",")) {
            break;
        }
    }

    Ok(String::new())
}

unsafe fn proc_body() -> Result<String> {
    terminator(Type::Keyword, Some("begin"))?;
    stm_list()?;
    terminator(Type::Keyword, Some("end"))?;
    Ok(String::new())
}

unsafe fn stm_list() -> Result<String> {
    let i = idx;
    match stm() {
        Err(e) => {
            if i != idx {
                return Err(e);
            }
        }
        Ok(_) => loop {
            if let Err(_) = terminator(Type::Separator, Some(";")) {
                break;
            }
            stm()?;
        },
    }

    Ok(String::new())
}

unsafe fn stm() -> Result<String> {
    let i = idx;
    match conditional_stm() {
        Ok(v) => return Ok(v),
        Err(e) => {
            if i != idx {
                return Err(e);
            }
        }
    };

    match loop_stm() {
        Ok(v) => return Ok(v),
        Err(e) => {
            if i != idx {
                return Err(e);
            }
        }
    };
    match input_stm() {
        Ok(v) => return Ok(v),
        Err(e) => {
            if i != idx {
                return Err(e);
            }
        }
    };

    match output_stm() {
        Ok(v) => return Ok(v),
        Err(e) => {
            if i != idx {
                return Err(e);
            }
        }
    };

    match call_stm() {
        Ok(v) => return Ok(v),
        Err(e) => {
            if i < idx - 1 {
                return Err(e);
            }
            idx = i;
        }
    };

    match assignment_stm() {
        Ok(v) => return Ok(v),
        Err(e) => {
            if i < idx - 1 {
                return Err(e);
            }
            idx = i;
        }
    };

    Ok(String::new())
}

unsafe fn input_stm() -> Result<String> {
    let op = terminator(Type::Keyword, Some("read"))?;
    let result = terminator(Type::Identifier, None)?;
    check_undef(result.as_str(), &["integer", "float"])?;
    quaternions.push(Quaternion {
        op,
        arg1: "_".to_string(),
        arg2: "_".to_string(),
        result,
    });
    Ok(String::new())
}

unsafe fn output_stm() -> Result<String> {
    let op = terminator(Type::Keyword, Some("write"))?;
    let arg1 = exp()?;
    check_undef(arg1.as_str(), &["integer", "float"])?;
    quaternions.push(Quaternion {
        op,
        arg1,
        arg2: "_".to_string(),
        result: "_".to_string(),
    });
    Ok(String::new())
}

unsafe fn call_stm() -> Result<String> {
    let arg1 = terminator(Type::Identifier, None)?;
    check_undef(arg1.as_str(), &["procedure"])?;
    terminator(Type::Separator, Some("("))?;
    act_param_list()?;
    terminator(Type::Separator, Some(")"))?;
    quaternions.push(Quaternion {
        op: "call".to_string(),
        arg1,
        arg2: "_".to_string(),
        result: "_".to_string(),
    });
    Ok(String::new())
}

unsafe fn assignment_stm() -> Result<String> {
    let result = terminator(Type::Identifier, None)?;
    if let Err(e) = check_undef(result.as_str(), &["integer", "float"]) {
        idx += 1; //for output error
        return Err(e);
    }
    let op = terminator(Type::Operator, Some("="))?;
    let arg1 = exp()?;
    quaternions.push(Quaternion {
        op,
        arg1,
        arg2: "_".to_string(),
        result,
    });
    Ok(String::new())
}

unsafe fn conditional_stm() -> Result<String> {
    terminator(Type::Keyword, Some("if"))?;
    let arg1 = conditional_exp()?;

    quaternions.push(Quaternion {
        op: "jnz".to_string(),
        arg1,
        arg2: "_".to_string(),
        result: (quaternions.len() + 2).to_string(),
    });

    let a1 = quaternions.len();
    quaternions.push(Quaternion {
        op: "j".to_string(),
        arg1: "_".to_string(),
        arg2: "_".to_string(),
        result: String::new(),
    });

    terminator(Type::Keyword, Some("then"))?;
    stm_list()?;
    let a2 = quaternions.len();
    quaternions.push(Quaternion {
        op: "j".to_string(),
        arg1: "_".to_string(),
        arg2: "_".to_string(),
        result: String::new(),
    });
    quaternions[a1].result = quaternions.len().to_string();

    // if unmatched, output error message and continue.
    match terminator(Type::Keyword, Some("else")) {
        Ok(_) => {
            stm_list()?;
        }
        Err(e) => {
            println!("{}", e);
        }
    };

    quaternions[a2].result = quaternions.len().to_string();

    terminator(Type::Keyword, Some("fi"))?;

    Ok(String::new())
}

unsafe fn loop_stm() -> Result<String> {
    let a1 = quaternions.len();

    terminator(Type::Keyword, Some("while"))?;
    let arg1 = conditional_exp()?;

    let a2 = quaternions.len();
    quaternions.push(Quaternion {
        op: "jez".to_string(),
        arg1,
        arg2: "_".to_string(),
        result: String::new(),
    });

    terminator(Type::Keyword, Some("then"))?;
    stm_list()?;

    quaternions.push(Quaternion {
        op: "j".to_string(),
        arg1: "_".to_string(),
        arg2: "_".to_string(),
        result: a1.to_string(),
    });
    quaternions[a2].result = quaternions.len().to_string();

    terminator(Type::Keyword, Some("endwh"))?;

    Ok(String::new())
}

unsafe fn act_param_list() -> Result<String> {
    let i = idx;
    match exp() {
        Err(e) => {
            if i != idx {
                return Err(e);
            }
        }
        Ok(_) => loop {
            if let Err(_) = terminator(Type::Separator, Some(",")) {
                break;
            }
            exp()?;
        },
    }

    Ok(String::new())
}

unsafe fn exp() -> Result<String> {
    let mut arg1 = term()?;

    loop {
        let op;
        match multi_terminator(Type::Operator, &["+", "-"]) {
            Ok(val) => op = val,
            Err(_) => break,
        }
        let arg2 = term()?;
        let result = temp_gen.gen();
        quaternions.push(Quaternion {
            op,
            arg1: arg1.clone(),
            arg2,
            result: result.clone(),
        });
        arg1 = result;
    }

    Ok(arg1)
}

unsafe fn term() -> Result<String> {
    let mut arg1 = factor()?;

    loop {
        let op;
        match multi_terminator(Type::Operator, &["*", "/"]) {
            Ok(val) => op = val,
            Err(_) => break,
        }
        let arg2 = factor()?;
        let result = temp_gen.gen();
        quaternions.push(Quaternion {
            op,
            arg1: arg1.clone(),
            arg2,
            result: result.clone(),
        });
        arg1 = result;
    }

    Ok(arg1)
}

unsafe fn factor() -> Result<String> {
    let val = match terminator(Type::Identifier, None) {
        Ok(val) => {
            check_undef(val.as_str(), &["integer", "float"])?;
            val
        }
        Err(_) => match terminator(Type::Integer, None) {
            Ok(val) => val,
            Err(_) => match terminator(Type::FloatPoint, None) {
                Ok(val) => val,
                Err(_) => {
                    match terminator(Type::Separator, Some("(")) {
                        Ok(_) => {
                            let mut tmp = exp();
                            if tmp.is_ok() {
                                tmp = tmp.and(terminator(Type::Separator, Some(")")));
                            }
                            tmp
                        }
                        Err(e) => Err(e),
                    }
                }?,
            },
        },
    };
    // let val = terminator(Type::Identifier, None)
    //     .or(terminator(Type::Integer, None))
    //     .or(terminator(Type::FloatPoint, None))
    //     .or({
    //         let tmp = terminator(Type::Separator, Some("(".to_string())).and(exp());
    //         if tmp.is_ok() {
    //             terminator(Type::Separator, Some(")".to_string()))?;
    //         }
    //         tmp
    //     })?;
    // .and(cannot_end())
    // .and(terminator(Type::Separator, Some(")".to_string()))))?;

    Ok(val)
}

unsafe fn conditional_exp() -> Result<String> {
    let mut arg1 = relation_exp()?;

    loop {
        let op;
        match terminator(Type::Keyword, Some("or")) {
            Ok(val) => op = val,
            Err(_) => break,
        }
        let arg2 = relation_exp()?;
        let result = temp_gen.gen();
        quaternions.push(Quaternion {
            op,
            arg1: arg1.clone(),
            arg2,
            result: result.clone(),
        });
        arg1 = result;
    }

    Ok(arg1)
}

unsafe fn relation_exp() -> Result<String> {
    let mut arg1 = comp_exp()?;

    loop {
        let op;
        match terminator(Type::Keyword, Some("and")) {
            Ok(val) => op = val,
            Err(_) => break,
        }
        let arg2 = comp_exp()?;
        let result = temp_gen.gen();
        quaternions.push(Quaternion {
            op,
            arg1: arg1.clone(),
            arg2,
            result: result.clone(),
        });
        arg1 = result;
    }

    Ok(arg1)
}

unsafe fn comp_exp() -> Result<String> {
    let arg1 = exp()?;
    let op = cmp_op()?;
    let arg2 = exp()?;
    let result = temp_gen.gen();
    quaternions.push(Quaternion {
        op,
        arg1,
        arg2,
        result: result.clone(),
    });

    Ok(result)
}

unsafe fn cmp_op() -> Result<String> {
    const CMP: [&str; 6] = ["<", "<=", ">", ">=", "==", "<>"];

    let val = multi_terminator(Type::Operator, &CMP)?;

    Ok(val)
}

unsafe fn check_undef(val: &str, ty: &[&str]) -> Result<()> {
    if temp_gen.contains(val) {
        return Ok(());
    }
    for i in params.iter().chain(vars.iter()).chain(global_vars.iter()) {
        if i.name == val {
            if ty.contains(&i.ty.as_str()) {
                return Ok(());
            } else {
                return Err(StandardError::new(format!(
                    "Identifier {} exists, but expected Type `{}`, found Type `{}`",
                    val,
                    ty.join(", "),
                    i.ty
                )));
            }
        }
    }
    return Err(StandardError::new(format!(
        "Identifier {} does not exist",
        val
    )));
}

// unsafe fn check_mul_def(val: &str, ty: &[&str], ) -> Result<()> {
//
// }

unsafe fn terminator(ty: Type, val: Option<&str>) -> Result<String> {
    if idx == words.len() {
        return Err(StandardError::new(
            "End where it should not end".to_string(),
        ));
    }
    if !(words[idx].ty == ty && (val.is_none() || words[idx].val == val.unwrap())) {
        return Err(StandardError::new(format!(
            "line {}, column {}.\nexpected `{}`, found `{}`.",
            words[idx].row,
            words[idx].col,
            val.unwrap_or(format!("{}", ty).as_str()),
            words[idx].val
        )));
    }
    idx += 1;

    Ok(words[idx - 1].val.clone())
}

unsafe fn multi_terminator(ty: Type, values: &[&str]) -> Result<String> {
    if idx == words.len() {
        return Err(StandardError::new(
            "End where it should not end".to_string(),
        ));
    }
    if !(words[idx].ty == ty && values.contains(&words[idx].val.as_str())) {
        return Err(StandardError::new(format!(
            "line {}, column {}.\nexpected `{}`, found `{}`.",
            words[idx].row, words[idx].col, words[idx].ty, words[idx].val
        )));
    }
    idx += 1;

    Ok(words[idx - 1].val.clone())
}

// fn new_temp() -> String {
//     static mut N: usize = 0;
//     unsafe {
//         N += 1;
//         format!("t{}", N)
//     }
// }

// unsafe fn try_match(f: unsafe fn() -> Result<String>) -> Result<String> {
//     let i = idx;
//     match f() {
//         Ok(v) => Ok(v),
//         Err(e) => {
//             idx = i;
//             Err(e)
//         }
//     }
// }
