use termtree::Tree;

use crate::ast::*;

use super::type_str;
use super::exprs::{build_expr_tree, expr_label};

pub(super) fn build_stmt_tree(stmt: &Stmt) -> Tree<String> {
    match stmt {
        Stmt::DeclNoVal { name, ty: Some(ty) } => {
            Tree::new(format!("let there be {}: {}", name, type_str(ty)))
        }

        Stmt::DeclNoVal { name, ty: None } => {
            Tree::new(format!("let there be {} (untyped)", name))
        }

        Stmt::DeclInfer { name, val } => {
            let mut tree = Tree::new(format!("let there {} =", name));
            tree.push(build_expr_tree(val));
            tree
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

        Stmt::FnCallStmt { name, type_args, args } => {
            let ta = if type_args.is_empty() { String::new() } else {
                format!("<{}>", type_args.iter().map(type_str).collect::<Vec<_>>().join(", "))
            };
            let mut tree = Tree::new(format!("hail {}{} ({})", name, ta, args.len()));
            for arg in args { tree.push(build_expr_tree(arg)); }
            tree
        }

        Stmt::MethodCallStmt { method, target, args } => {
            let mut tree = Tree::new(format!("hail {} upon ({})", method, args.len()));
            tree.push(build_expr_tree(target));
            for arg in args { tree.push(build_expr_tree(arg)); }
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
                if otherwise.is_some() { " + otherwise" } else { "" },
            ));

            for (i, (cond, body)) in branches.iter().enumerate() {
                let label = if i == 0 { "cond" } else { "otherwise so" };
                let mut branch_tree = Tree::new(label.into());
                branch_tree.push(build_expr_tree(cond));
                let mut then_tree = Tree::new("then".into());
                for stmt in body { then_tree.push(build_stmt_tree(stmt)); }
                branch_tree.push(then_tree);
                tree.push(branch_tree);
            }

            if let Some(otherwise_body) = otherwise {
                let mut otherwise_tree = Tree::new("otherwise".into());
                for stmt in otherwise_body { otherwise_tree.push(build_stmt_tree(stmt)); }
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
            for stmt in body { body_tree.push(build_stmt_tree(stmt)); }
            tree.push(body_tree);
            tree
        }

        Stmt::Confess { try_block, handlers, absolve } => {
            let mut tree = Tree::new(format!(
                "confess ({} handler{}{})",
                handlers.len(),
                if handlers.len() != 1 { "s" } else { "" },
                if absolve.is_some() { " + absolve" } else { "" },
            ));

            let mut try_tree = Tree::new("try".into());
            for stmt in try_block { try_tree.push(build_stmt_tree(stmt)); }
            tree.push(try_tree);

            for handler in handlers {
                let label = match &handler.binding {
                    Some(b) => format!("answer for {} as {}", handler.sin_type, b),
                    None    => format!("answer for {}", handler.sin_type),
                };
                let mut handler_tree = Tree::new(label);
                for stmt in &handler.body { handler_tree.push(build_stmt_tree(stmt)); }
                tree.push(handler_tree);
            }

            if let Some(absolve_block) = absolve {
                let mut absolve_tree = Tree::new("absolve".into());
                for stmt in absolve_block { absolve_tree.push(build_stmt_tree(stmt)); }
                tree.push(absolve_tree);
            }

            tree
        }

        Stmt::Discern { target, branches, otherwise } => {
            let mut tree = Tree::new(format!(
                "discern {} ({} branch{}{})",
                expr_label(target),
                branches.len(),
                if branches.len() != 1 { "es" } else { "" },
                if otherwise.is_some() { " + otherwise" } else { "" },
            ));

            for branch in branches {
                let label = if branch.bindings.is_empty() {
                    format!("as {}", branch.variant)
                } else {
                    format!("as {} bearing {}", branch.variant, branch.bindings.join(", "))
                };
                let mut branch_tree = Tree::new(label);
                for stmt in &branch.body { branch_tree.push(build_stmt_tree(stmt)); }
                tree.push(branch_tree);
            }

            if let Some(otherwise_body) = otherwise {
                let mut otherwise_tree = Tree::new("otherwise".into());
                for stmt in otherwise_body { otherwise_tree.push(build_stmt_tree(stmt)); }
                tree.push(otherwise_tree);
            }

            tree
        }

        Stmt::Transgress { sin_type, args } => {
            let mut tree = Tree::new(format!("transgress {} ({})", sin_type, args.len()));
            for arg in args { tree.push(build_expr_tree(arg)); }
            tree
        }

        Stmt::Forsake => Tree::new("forsake".into()),
        Stmt::Ascend  => Tree::new("ascend".into()),
    }
}
