## Compiler
A Simple Grammar Analysis and Quad Generation made by Rust
## Rules
Program→ ProgramHead VarDecpart ProgramBody 
ProgramHead→ 'program' ID

VarDecpart→ ε
        | 'var' VarDecList
VarDecList→ VarIdList {VarIdList}
VarIdList→ TypeName ID {',' ID} ';'
TypeName→'integer'
        | 'float'

ProgramBody→ε
| ProcDec {ProcDec}
ProcDec→ 'procedure' ID '(' ParamList ')' ';' VarDecpart ProcBody

ParamList→ ε
        | Param {';' Param}
Param→ TypeName ID {',' ID} 

ProcBody→ 'begin' StmList 'end'

StmList→ ε
     | Stm {';' Stm}

Stm→ConditionalStm
     | LoopStm
     | InputStm
     | OutputStm
     | CallStm
| AssignmentStm
InputStm→'read' ID
OutputStm→'write' Exp
CallStm→ ID '(' ActParamList ')'
AssignmentStm→ ID '=' Exp
ConditionalStm→'if' ConditionalExp 'then' StmList 'else' StmList 'fi'
LoopStm→'while' ConditionalExp 'do' StmList 'endwh'

ActParamList→ ε
          | Exp {',' Exp}

Exp→ Term {'+'|'-' Term}
Term→ Factor {'*'|'/' Factor}
Factor→ ID | INTC | DECI | '(' Exp ')'

ConditionalExp→RelationExp {'or' RelationExp}  
RelationExp→ CompExp {'and' CompExp}
CompExp→ Exp CmpOp Exp
CmpOp→'<' | '<=' | '>' | '>=| '==' | '<>'          
