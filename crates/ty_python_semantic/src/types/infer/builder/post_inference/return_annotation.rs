use ruff_db::files::File;
use ruff_diagnostics::{Edit, Fix};
use ruff_python_ast::{self as ast, helpers::is_dunder, name::UnqualifiedName};
use ruff_text_size::Ranged;
use ty_module_resolver::file_to_module;
use ty_python_core::{scope::NodeWithScopeKind, semantic_index};

use crate::Db;
use crate::dunder_all::dunder_all_names;
use crate::lint::LintId;
use crate::types::context::InferContext;
use crate::types::diagnostic::{
    MISSING_RETURN_TYPE_CLASS_METHOD, MISSING_RETURN_TYPE_PRIVATE_FUNCTION,
    MISSING_RETURN_TYPE_PUBLIC_FUNCTION, MISSING_RETURN_TYPE_SPECIAL_METHOD,
    MISSING_RETURN_TYPE_STATIC_METHOD,
};
use crate::types::function::{FunctionType, InferredReturnTypeFor};

#[salsa::tracked]
pub(in crate::types) fn rules_enabled(db: &dyn Db, file: File) -> bool {
    let selection = db.rule_selection(file);
    db.should_check_file(file)
        && [
            &MISSING_RETURN_TYPE_PUBLIC_FUNCTION,
            &MISSING_RETURN_TYPE_PRIVATE_FUNCTION,
            &MISSING_RETURN_TYPE_SPECIAL_METHOD,
            &MISSING_RETURN_TYPE_STATIC_METHOD,
            &MISSING_RETURN_TYPE_CLASS_METHOD,
        ]
        .into_iter()
        .any(|rule| selection.get(LintId::of(rule)).is_some())
}

pub(super) fn check_missing_return_annotation<'db>(
    context: &InferContext<'db, '_>,
    function_type: FunctionType<'db>,
) {
    let db = context.db();
    let literal = function_type.literal(db);
    let last_definition = literal.last_definition;
    if last_definition.has_explicit_return_annotation(db)
        || literal.has_separate_implementation(db)
        || !rules_enabled(db, context.file())
    {
        return;
    }

    let function = last_definition.node(db, context.file(), context.module());
    let is_method = last_definition.body_scope(db).is_method_scope(db);
    let (lint, kind) = if is_method && last_definition.is_classmethod(db) {
        (&MISSING_RETURN_TYPE_CLASS_METHOD, "class method")
    } else if is_method && last_definition.is_staticmethod(db) {
        (&MISSING_RETURN_TYPE_STATIC_METHOD, "static method")
    } else if is_method && is_dunder(&function.name) {
        (&MISSING_RETURN_TYPE_SPECIAL_METHOD, "special method")
    } else if is_private(context, function, is_method) {
        (&MISSING_RETURN_TYPE_PRIVATE_FUNCTION, "private function")
    } else {
        (&MISSING_RETURN_TYPE_PUBLIC_FUNCTION, "public function")
    };

    let name = &function.name;
    let Some(builder) = context.report_lint(lint, name) else {
        return;
    };
    let mut diagnostic = builder.into_diagnostic(format_args!(
        "Missing return type annotation for {kind} `{name}`"
    ));
    if let Some((annotation, imports)) =
        suggested_annotation(context, function, function_type, is_method)
    {
        diagnostic.help(format_args!("Add return type annotation `{annotation}`"));
        diagnostic.set_fix(Fix::unsafe_edits(
            Edit::insertion(format!(" -> {annotation}"), function.parameters.end()),
            imports,
        ));
    }
}

fn suggested_annotation<'db>(
    context: &InferContext<'db, '_>,
    function: &ast::StmtFunctionDef,
    function_type: FunctionType<'db>,
    is_method: bool,
) -> Option<(String, Vec<Edit>)> {
    if is_method && function.name.as_str() == "__init__" {
        return Some((String::from("None"), Vec::new()));
    }
    let db = context.db();
    let overload = function_type.literal(db).last_definition;
    if function.is_async
        || overload.is_overload(db)
        || context.file().is_stub(db)
        || function_type.has_trivial_body(db)
    {
        return None;
    }
    let env = context.program_environment();
    let ty = function_type.inferred_return_type(db, InferredReturnTypeFor::Annotation);
    if ty.has_dynamic(db, env) {
        return None;
    }
    let details = ty.promote(db, env).display(db, env).to_string_parts();
    let index = semantic_index(db, context.program_file());
    let annotation_scope = index.parent_scope_id(overload.body_scope(db).file_scope_id(db))?;
    context
        .importer()
        .type_annotation(&details, annotation_scope, function.parameters.end())
}

fn is_private(
    context: &InferContext<'_, '_>,
    function: &ast::StmtFunctionDef,
    is_method: bool,
) -> bool {
    let db = context.db();
    if function.name.starts_with('_') {
        return true;
    }
    if is_method && function.decorator_list.iter()
        .filter_map(|decorator| UnqualifiedName::from_expr(&decorator.expression))
        .any(|name| matches!(name.segments(), [property, "setter" | "deleter"] if *property == function.name.as_str()))
    {
        return true;
    }

    let file = context.program_file();
    let index = semantic_index(db, file);
    let mut exported_name = function.name.as_str();
    for (_, scope) in index.ancestor_scopes(context.scope().file_scope_id(db)) {
        match scope.node() {
            NodeWithScopeKind::Function(_) => return true,
            NodeWithScopeKind::Class(class) => {
                exported_name = class.node(context.module()).name.as_str();
                if exported_name.starts_with('_') {
                    return true;
                }
            }
            _ => {}
        }
    }

    file_to_module(db, file.resolver_file(db)).map_or_else(
        || {
            context
                .file()
                .path(db)
                .as_system_path()
                .and_then(|p| p.file_stem())
                .is_some_and(|stem| stem.starts_with('_'))
        },
        |module| module.name(db).is_private(),
    ) || dunder_all_names(db, file).is_some_and(|names| !names.contains(exported_name))
}
