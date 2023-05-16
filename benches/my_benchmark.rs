use criterion::{criterion_group, criterion_main, Criterion};

// #[path = "../src/analyze.rs"]
// mod analyze;

fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("analyze");

    // let input = std::fs::read_to_string("./tests/mocks/input2.cs")
    //     .expect("Something went wrong reading the file");

    // let test = input.lines().collect::<Vec<&str>>();

    // let data = Data {
    //     lines: test.iter().map(|x| x.to_string()).collect::<Vec<String>>(),
    //     class_name: None,
    //     blocks: None,
    // };

    // group.bench_function("analyze_lines", |b| {
    //     b.iter(|| {
    //         crate::analyze::analyze_lines(data.clone());
    //     })
    // });

    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);

#[derive(PartialEq, Clone, Debug)]
pub enum BlockType {
    Namespace,
    Class,
    Constructor,
    Method,
    Context,
    Variable,
    Select,
    If,
    Unknown,
}

#[derive(Clone, Debug)]
pub struct Block {
    pub start: u32,
    pub end: Option<u32>,
    pub block_type: BlockType,
    pub details: Option<BlockDetails>,
}

#[derive(Clone)]
pub struct Data {
    pub lines: Vec<String>,
    pub class_name: Option<String>,
    pub blocks: Option<Vec<Block>>,
}

#[derive(PartialEq, Clone, Debug)]
pub enum BlockDetails {
    ContextBlock {
        variable: String,
    },
    MethodBlock {
        name: String,
        http_method: Option<HttpType>,
        variables: Vec<Variable>,
        uses_context: bool,
    },
    VariableBlock {
        name: String,
        data_type: String,
    },
    SelectBlock {
        query_type: QueryType,
        tables: Vec<Table>,
        where_clauses: Vec<WhereClause>,
        return_data: Vec<ReturnData>,
        syntax: LinqSyntax,
        has_return: bool,
    },
    IfBlock {
        clause: String,
        is_else: bool,
    },
}

#[derive(PartialEq, Clone, Debug)]
pub enum HttpType {
    HttpGet,
    HttpPost,
    HttpPut,
    HttpDelete,
}

#[derive(PartialEq, Clone, Debug)]
pub struct Variable {
    pub name: String,
    pub variable_type: String,
}

#[derive(PartialEq, Clone, Debug)]
pub enum QueryType {
    Many,
    First,
    Unique,
}

#[derive(PartialEq, Clone, Debug)]
pub enum LinqSyntax {
    Lambda,
    Query,
    Both,
}

#[derive(PartialEq, Clone, Debug)]
pub struct ReturnData {
    pub table: String,
    pub property: String,
    pub value: String,
}

#[derive(PartialEq, Clone, Debug)]
pub struct Table {
    pub name: String,
    pub shortcut: String,
    pub joined_tables: Vec<Table>,
    pub return_frequency: i32,
}

#[derive(PartialEq, Clone, Debug)]
pub struct WhereClause {
    pub shortcut: Vec<String>,
    pub property: Vec<String>,
    pub value: String,
    pub lambda_varible: Option<String>,
}
