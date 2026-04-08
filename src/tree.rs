use termtree::Tree;

use crate::ast::*;

pub fn print_program(program: &Program) {
    println!("{}", build_program_tree(program));
}

fn build_program_tree(program: &Program) -> Tree<String> {
    let mut root = Tree::new("Program".into());

    for testament in &program.testaments {
        root.push(build_testament_tree(testament));
    }
    for decl in &program.top_decls {
        root.push(build_top_decl_tree(decl));
    }
    for stmt in &program.stmts {
        root.push(build_stmt_tree(stmt));
    }

    root
}

fn build_testament_tree(testament: &Testament) -> Tree<String> {
    let revealing = testament.revealing.as_ref()
        .map(|items| format!(" revealing {}", items.join(", ")))
        .unwrap_or_default();
    Tree::new(format!("[testament] {}{}", testament.name, revealing))
}

fn build_top_decl_tree(decl: &TopDecl) -> Tree<String> {
    match decl {
        TopDecl::Scripture { name, fields } => {
            let mut tree = Tree::new(format!("[scripture] {}", name));
            for (field_name, field_ty) in fields {
                tree.push(Tree::new(format!("{}: {}", field_name, type_str(field_ty))));
            }
            tree
        }
        TopDecl::SinDecl { name, fields } => {
            let mut tree = Tree::new(format!("[sin] {}", name));
            for (field_name, field_ty) in fields {
                tree.push(Tree::new(format!("{}: {}", field_name, type_str(field_ty))));
            }
            tree
        }
        TopDecl::Covenant { name, variants } => {
            let mut tree = Tree::new(format!("[covenant] {}", name));
            for v in variants {
                if v.fields.is_empty() {
                    tree.push(Tree::new(v.name.clone()));
                } else {
                    let fields_str = v.fields.iter()
                        .map(|(fn_, ft)| format!("{}: {}", fn_, type_str(ft)))
                        .collect::<Vec<_>>()
                        .join(", ");
                    tree.push(Tree::new(format!("{}({})", v.name, fields_str)));
                }
            }
            tree
        }
        TopDecl::Salm { name, params, ret_type, body } => {
            let mut tree = Tree::new(format!(
                "[salm] {} ({}) -> {}",
                name,
                params_str(params),
                type_str(ret_type)
            ));
            for stmt in body {
                tree.push(build_stmt_tree(stmt));
            }
            tree
        }
        TopDecl::MethodSalm { name, target_type, params, ret_type, body } => {
            let mut tree = Tree::new(format!(
                "[method salm] {} upon {} ({}) -> {}",
                name,
                target_type,
                params_str(params),
                type_str(ret_type)
            ));
            for stmt in body {
                tree.push(build_stmt_tree(stmt));
            }
            tree
        }
    }
}

fn build_stmt_tree(stmt: &Stmt) -> Tree<String> {
    match stmt {
        Stmt::DeclNoVal { name, ty } => {
            Tree::new(format!("let there be {}: {}", name, type_str(ty)))
        }
        Stmt::DeclVal { name, ty, val } => {
            let mut tree = Tree::new(format!("let there {}: {} =", name, type_str(ty)));
            tree.push(build_expr_tree(val));
            tree
        }
        Stmt::Assign { name, val } => {
            let mut tree = Tree::new(format!("{} become", name));
            tree.push(build_expr_tree(val));
            tree
        }
        Stmt::FnCallStmt { name, args } => {
            let mut tree = Tree::new(format!("hail {} ({})", name, args.len()));
            for arg in args {
                tree.push(build_expr_tree(arg));
            }
            tree
        }
        Stmt::MethodCallStmt { method, target, args } => {
            let mut tree = Tree::new(format!("hail {} upon {} ({})", method, target, args.len()));
            for arg in args {
                tree.push(build_expr_tree(arg));
            }
            tree
        }
        Stmt::Reveal(expr) => {
            let mut tree = Tree::new("reveal".into());
            tree.push(build_expr_tree(expr));
            tree
        }
        Stmt::Conditional { branches, otherwise } => {
            let mut tree = Tree::new(format!(
                "whether ({} branch{}{})",
                branches.len(),
                if branches.len() != 1 { "es" } else { "" },
                if otherwise.is_some() { " + otherwise" } else { "" }
            ));

            for (index, (cond, body)) in branches.iter().enumerate() {
                let mut branch_tree = Tree::new(if index == 0 {
                    "cond".into()
                } else {
                    "otherwise so".into()
                });
                branch_tree.push(build_expr_tree(cond));
                let mut then_tree = Tree::new("then".into());
                for stmt in body {
                    then_tree.push(build_stmt_tree(stmt));
                }
                branch_tree.push(then_tree);
                tree.push(branch_tree);
            }

            if let Some(otherwise_body) = otherwise {
                let mut otherwise_tree = Tree::new("otherwise".into());
                for stmt in otherwise_body {
                    otherwise_tree.push(build_stmt_tree(stmt));
                }
                tree.push(otherwise_tree);
            }

            tree
        }
        Stmt::Litany { cond, body } => {
            let mut tree = Tree::new("litany for".into());
            let mut cond_tree = Tree::new("cond".into());
            cond_tree.push(build_expr_tree(cond));
            tree.push(cond_tree);

            let mut body_tree = Tree::new("body".into());
            for stmt in body {
                body_tree.push(build_stmt_tree(stmt));
            }
            tree.push(body_tree);
            tree
        }
        Stmt::Confess { try_block, handlers, absolve } => {
            let mut tree = Tree::new(format!(
                "confess ({} handler{}{})",
                handlers.len(),
                if handlers.len() != 1 { "s" } else { "" },
                if absolve.is_some() { " + absolve" } else { "" }
            ));

            let mut try_tree = Tree::new("try".into());
            for stmt in try_block {
                try_tree.push(build_stmt_tree(stmt));
            }
            tree.push(try_tree);

            for handler in handlers {
                let label = match &handler.binding {
                    Some(binding) => format!("answer for {} as {}", handler.sin_type, binding),
                    None => format!("answer for {}", handler.sin_type),
                };
                let mut handler_tree = Tree::new(label);
                for stmt in &handler.body {
                    handler_tree.push(build_stmt_tree(stmt));
                }
                tree.push(handler_tree);
            }

            if let Some(absolve_block) = absolve {
                let mut absolve_tree = Tree::new("absolve".into());
                for stmt in absolve_block {
                    absolve_tree.push(build_stmt_tree(stmt));
                }
                tree.push(absolve_tree);
            }

            tree
        }
        Stmt::Discern { target, branches, otherwise } => {
            let mut tree = Tree::new(format!(
                "discern {} ({} branch{}{})",
                target,
                branches.len(),
                if branches.len() != 1 { "es" } else { "" },
                if otherwise.is_some() { " + otherwise" } else { "" }
            ));

            for branch in branches {
                let label = if branch.bindings.is_empty() {
                    format!("as {}", branch.variant)
                } else {
                    format!("as {} bearing {}", branch.variant, branch.bindings.join(", "))
                };
                let mut branch_tree = Tree::new(label);
                for stmt in &branch.body {
                    branch_tree.push(build_stmt_tree(stmt));
                }
                tree.push(branch_tree);
            }

            if let Some(otherwise_body) = otherwise {
                let mut otherwise_tree = Tree::new("otherwise".into());
                for stmt in otherwise_body {
                    otherwise_tree.push(build_stmt_tree(stmt));
                }
                tree.push(otherwise_tree);
            }

            tree
        }
        Stmt::Transgress { sin_type, args } => {
            let mut tree = Tree::new(format!("transgress {} ({})", sin_type, args.len()));
            for arg in args {
                tree.push(build_expr_tree(arg));
            }
            tree
        }
        Stmt::Forsake => Tree::new("forsake".into()),
        Stmt::Ascend => Tree::new("ascend".into()),
    }
}

fn build_expr_tree(expr: &Expr) -> Tree<String> {
    match expr {
        Expr::Lit(lit) => {
            let label = match lit {
                Literal::Int(n) => format!("Int({})", n),
                Literal::Float(f) => format!("Float({})", f),
                Literal::Str(s) => format!("Str({:?})", s),
                Literal::Bool(true) => "blessed".into(),
                Literal::Bool(false) => "forsaken".into(),
            };
            Tree::new(label)
        }
        Expr::Var(name) => Tree::new(format!("Var({})", name)),
        Expr::Negate(expr) => {
            let mut tree = Tree::new("Negate".into());
            tree.push(build_expr_tree(expr));
            tree
        }
        Expr::BinOp { op, left, right } => {
            let op_str = match op {
                BinOp::Add => "plus",
                BinOp::Sub => "minus",
                BinOp::Mul => "times",
                BinOp::Div => "over",
                BinOp::Rem => "remainder",
                BinOp::Eq => "is",
                BinOp::Ne => "is not",
                BinOp::Gt => "greater than",
                BinOp::Lt => "lesser than",
                BinOp::Ge => "no lesser than",
                BinOp::Le => "no greater than",
            };
            let mut tree = Tree::new(format!("BinOp({})", op_str));
            tree.push(build_expr_tree(left));
            tree.push(build_expr_tree(right));
            tree
        }
        Expr::FnCall { name, args } => {
            let mut tree = Tree::new(format!("hail {} ({})", name, args.len()));
            for arg in args {
                tree.push(build_expr_tree(arg));
            }
            tree
        }
        Expr::MethodCall { method, target, args } => {
            let mut tree = Tree::new(format!("hail {} upon {} ({})", method, target, args.len()));
            for arg in args {
                tree.push(build_expr_tree(arg));
            }
            tree
        }
        Expr::Manifest { scripture, args } => {
            let mut tree = Tree::new(format!("manifest {} ({})", scripture, args.len()));
            for arg in args {
                tree.push(build_expr_tree(arg));
            }
            tree
        }
        Expr::FieldAccess { field, object } => {
            let mut tree = Tree::new(format!("FieldAccess({})", field));
            tree.push(build_expr_tree(object));
            tree
        }
        Expr::SelfFieldAccess { field } => {
            Tree::new(format!("FieldAccess({} from its)", field))
        }
    }
}

fn type_str(ty: &HolyType) -> String {
    match ty {
        HolyType::Atom => "atom".into(),
        HolyType::Fractional => "fractional".into(),
        HolyType::Word => "word".into(),
        HolyType::Dogma => "dogma".into(),
        HolyType::Void => "void".into(),
        HolyType::Custom(name) => name.clone(),
    }
}

fn params_str(params: &[(String, HolyType)]) -> String {
    if params.is_empty() {
        return "-".into();
    }

    params.iter()
        .map(|(name, ty)| format!("{}: {}", name, type_str(ty)))
        .collect::<Vec<_>>()
        .join(", ")
}
