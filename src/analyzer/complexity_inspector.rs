use syn::{File, Item};

use syn::ItemFn;

#[allow(dead_code)] //max_nesting_depth, return_count, param_count not used yet.
#[derive(Debug)]
pub struct FunctionComplexity {
    pub name: String,
    pub lines_of_code: usize,
    pub cyclomatic_complexity: usize,
    pub max_nesting_depth: usize,
    pub return_count: usize,
    pub param_count: usize,
}

use syn::{Expr, visit::Visit};

struct ComplexityVisitor {
    cyclomatic_complexity: usize,
    max_depth: usize,
    current_depth: usize,
    return_count: usize,
}

impl<'ast> Visit<'ast> for ComplexityVisitor {
    fn visit_expr(&mut self, node: &'ast Expr) {
        match node {
            Expr::If(_) | Expr::Match(_) | Expr::While(_) | Expr::ForLoop(_) | Expr::Loop(_) => {
                self.cyclomatic_complexity += 1;
                self.current_depth += 1;
                self.max_depth = self.max_depth.max(self.current_depth);
                syn::visit::visit_expr(self, node);
                self.current_depth -= 1;
                return;
            }
            Expr::Closure(_) => {
                // Skip closure internals for simplicity
                return;
            }
            Expr::Return(_) => {
                self.return_count += 1;
            }
            _ => {}
        }

        syn::visit::visit_expr(self, node);
    }
}

pub fn analyze_function(func: &ItemFn) -> FunctionComplexity {
    let loc = func.block.stmts.len(); // Rough LOC as number of statements

    let mut visitor = ComplexityVisitor {
        cyclomatic_complexity: 1, // baseline
        max_depth: 0,
        current_depth: 0,
        return_count: 0,
    };
    visitor.visit_block(&func.block);

    let param_count = func.sig.inputs.len();

    FunctionComplexity {
        name: func.sig.ident.to_string(),
        lines_of_code: loc,
        cyclomatic_complexity: visitor.cyclomatic_complexity,
        max_nesting_depth: visitor.max_depth,
        return_count: visitor.return_count,
        param_count,
    }
}

pub fn analyze_file(file: &File) -> Vec<FunctionComplexity> {
    file.items
        .iter()
        .filter_map(|item| {
            if let Item::Fn(func) = item {
                Some(analyze_function(func))
            } else {
                None
            }
        })
        .collect()
}
