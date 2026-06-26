use std::fs;
use std::path::{Path, PathBuf};

use oxc::allocator::Allocator;
use oxc::ast::ast::{
    Argument, Expression, ImportDeclaration, StaticMemberExpression, StringLiteral,
    VariableDeclaration,
};
use oxc::parser::{ParseOptions, Parser};
use oxc::span::SourceType;

use oxc_resolver::{
    EnforceExtension, ResolveError, ResolveOptions, Resolver, TsconfigDiscovery, TsconfigOptions,
    TsconfigReferences,
};

struct ImportResolver {
    root: PathBuf,
    resolver: Resolver,
}

impl ImportResolver {
    fn new<P: AsRef<Path>>(path: P) -> Self {
        let path = path.as_ref();
        let root = PathBuf::from(path).canonicalize().unwrap();
        let mut ts_conf = root.clone();
        ts_conf.push("tsconfig.json");

        let options = ResolveOptions {
            // XXX: do any of these matter when we have ts-config.json?
            // alias_fields: vec![vec!["browser".into()]],
            // alias: vec![("asdf".into(), vec![AliasValue::from("./test.js")])],
            extensions: vec![".js".into(), ".ts".into(), ".jsx".into(), ".tsx".into()],
            // extensions: vec![".js".into(), ".ts".into()],
            extension_alias: vec![
                (".js".into(), vec![".ts".into(), ".js".into()]),
                (".jsx".into(), vec![".tsx".into(), ".jsx".into()]),
            ],
            // // ESM
            // condition_names: vec!["node".into(), "import".into()],
            // CJS
            // condition_names: vec!["node".into(), "require".into()],
            builtin_modules: true,
            enforce_extension: EnforceExtension::Disabled,
            tsconfig: Some(TsconfigDiscovery::Manual(TsconfigOptions {
                config_file: ts_conf,
                references: TsconfigReferences::Auto,
            })),
            ..ResolveOptions::default()
        };

        let resolver = Resolver::new(options);
        Self { root, resolver }
    }

    fn format_relative(file: &Path, import: &str) -> String {
        if import.starts_with('.') {
            let file = PathBuf::from(file);
            let mut root = file.parent().unwrap().to_path_buf();
            root.push(import);
            root.to_string_lossy().to_string()
        } else {
            import.to_string()
        }
    }

    // Resolve import paths, ignoring buildin modules
    fn resolve_import_path(&self, import: &str) -> Option<String> {
        match self.resolver.resolve(&self.root, import) {
            Ok(resolved) => Some(resolved.full_path().to_string_lossy().to_string()),
            Err(ResolveError::Builtin {
                resolved: _,
                is_runtime_module: _,
            }) => None,
            Err(err) => panic!("{err:#?}"),
        }
    }
}

struct ImportParser {}

impl ImportParser {
    fn handle_import_decl<'a>(decl: &'a ImportDeclaration) -> &'a StringLiteral<'a> {
        // only sources now, rest later
        &decl.source
    }

    fn handle_var_decl_static_import<'a>(
        import: &'a StaticMemberExpression,
    ) -> Vec<&'a StringLiteral<'a>> {
        let mut stat = import;
        while let Expression::StaticMemberExpression(inner) = &stat.object {
            stat = inner;
        }
        let expr = stat.object.without_parentheses();
        if let Expression::AwaitExpression(wait) = expr
            && let Expression::ImportExpression(import) = &wait.argument
        {
            match &import.source {
                Expression::StringLiteral(arg) => {
                    return vec![arg];
                }
                Expression::ConditionalExpression(cond) => {
                    if let (Expression::StringLiteral(first), Expression::StringLiteral(second)) =
                        (&cond.consequent, &cond.alternate)
                    {
                        return vec![first, second];
                    }
                }
                imp => todo!("Implement import exporession argument\n{imp:#?}"),
            }
        }

        vec![]
    }

    /// Checking for `const foo = require("bar");
    fn handle_var_decl<'a>(decl: &'a VariableDeclaration) -> Vec<&'a StringLiteral<'a>> {
        if let Some(decl) = decl.declarations.first() {
            if let Some(Expression::CallExpression(init_call)) = &decl.init {
                if let Expression::StaticMemberExpression(ident) = &init_call.callee {
                    return Self::handle_var_decl_static_import(ident);
                }
                let is_require = if let Expression::Identifier(ident) = &init_call.callee {
                    ident.name == "require"
                } else {
                    false
                };

                if is_require
                    && let Some(Argument::StringLiteral(arg)) = init_call.arguments.first()
                {
                    return vec![arg];
                }
            }

            // const esbuild = (await import(useWasm ? 'esbuild-wasm' : 'esbuild')).default;
            // and the like
            if let Some(Expression::StaticMemberExpression(stat_mem)) = &decl.init {
                return Self::handle_var_decl_static_import(stat_mem);
            }
        }

        vec![]
    }

    fn parse<P: AsRef<Path>>(path: P) -> Vec<String> {
        let mut imports: Vec<String> = Vec::new();

        let path = path.as_ref();
        // ignore css and scss files
        if path.to_string_lossy().ends_with("css") {
            return imports;
        }

        let source_text = fs::read_to_string(path).unwrap();
        let source_type = SourceType::from_path(path).unwrap();

        // Parse the source code
        let allocator = Allocator::default();
        let ret = Parser::new(&allocator, &source_text, source_type)
            .with_options(ParseOptions {
                parse_regular_expression: true,
                ..ParseOptions::default()
            })
            .parse();

        for stmt in &ret.program.body {
            match stmt {
                oxc::ast::ast::Statement::ImportDeclaration(import) => {
                    imports.push(Self::handle_import_decl(import).to_string());
                }
                oxc::ast::ast::Statement::VariableDeclaration(var) => {
                    imports.extend(
                        Self::handle_var_decl(var)
                            .iter()
                            .map(|import| import.to_string()),
                    );
                }
                oxc::ast::ast::Statement::TSImportEqualsDeclaration(_) => {
                    todo!("Handle TSImportEqualsDelaration")
                }
                oxc::ast::ast::Statement::ExportAllDeclaration(export) => {
                    imports.push(export.source.to_string());
                }
                oxc::ast::ast::Statement::ExportNamedDeclaration(export) => {
                    if let Some(source) = &export.source {
                        imports.push(source.to_string());
                    }
                }
                // s => todo!("Unhandeled stmt: {s:#?}"),
                _ => {}
            }
        }

        imports
    }
}

pub struct ImportAnalyzer {
    root: PathBuf,
    resolver: ImportResolver,
    dependency: String,
}

impl ImportAnalyzer {
    pub fn new<P: AsRef<Path>>(root: P, dependency: &str) -> Self {
        let root = root.as_ref();
        let root = root.canonicalize().unwrap();
        Self {
            dependency: dependency.into(),
            resolver: ImportResolver::new(&root),
            root,
        }
    }

    pub fn analyze_imports<P: AsRef<Path>>(&self, file: P, found: &mut Vec<String>) {
        let file = file.as_ref();
        let imports = ImportParser::parse(file);

        for import in &imports {
            // internal node things are safe to ignore
            if import.starts_with("node:") {
                continue;
            }

            if self.dependency == *import || import.starts_with(&format!("{}/", self.dependency)) {
                found.push(file.to_string_lossy().to_string());
            }

            let full_import = ImportResolver::format_relative(file, import);
            let resolv = self.resolver.resolve_import_path(&full_import);
            if let Some(resolv) = resolv {
                self.analyze_imports(resolv, found);
            }
        }
    }

    pub fn analyze<P: AsRef<Path>>(&self, file: P) -> Vec<String> {
        let mut found_imports = Vec::new();
        let mut full_path = self.root.clone();
        full_path.push(file);
        self.analyze_imports(full_path, &mut found_imports);
        found_imports
    }
}
