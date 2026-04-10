use termtree::Tree;

use crate::ast::*;

use super::{params_str, type_str};
use super::stmts::build_stmt_tree;

pub(super) fn build_testament_tree(testament: &Testament) -> Tree<String> {
    let label = if testament.path.is_empty() {
        testament.name.clone()
    } else {
        format!("{} from {}", testament.name, testament.path.join(" from "))
    };
    let revealing = testament.revealing.as_ref()
        .map(|items| format!(" revealing {}", items.join(", ")))
        .unwrap_or_default();
    Tree::new(format!("[testament] {}{}", label, revealing))
}

pub(super) fn build_top_decl_tree(decl: &TopDecl) -> Tree<String> {
    match decl {
        TopDecl::Scripture { name, type_params, fields } => {
            let tp = fmt_type_params(type_params);
            let mut tree = Tree::new(format!("[scripture] {}{}", name, tp));
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

        TopDecl::Covenant { name, type_params, variants } => {
            let tp = fmt_type_params(type_params);
            let mut tree = Tree::new(format!("[covenant] {}{}", name, tp));
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

        TopDecl::Salm { name, type_params, params, ret_type, body } => {
            let tp = fmt_type_params(type_params);
            let mut tree = Tree::new(format!(
                "[salm] {}{} ({}) -> {}",
                name, tp, params_str(params), type_str(ret_type)
            ));
            for stmt in body {
                tree.push(build_stmt_tree(stmt));
            }
            tree
        }

        TopDecl::MethodSalm { name, type_params, target_type, params, ret_type, body } => {
            let tp = fmt_type_params(type_params);
            let mut tree = Tree::new(format!(
                "[method salm] {}{} upon {} ({}) -> {}",
                name, tp, target_type, params_str(params), type_str(ret_type)
            ));
            for stmt in body {
                tree.push(build_stmt_tree(stmt));
            }
            tree
        }
    }
}

fn fmt_type_params(type_params: &[String]) -> String {
    if type_params.is_empty() {
        String::new()
    } else {
        format!(" of {}", type_params.join(", "))
    }
}
