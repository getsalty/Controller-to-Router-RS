use std::collections::HashMap;

use crate::{
    Block, BlockDetails, BlockType, Data, HttpType, LinqSyntax, QueryType, ReturnData, Table,
    Variable, WhereClause,
};

pub fn analyze_lines(mut data: Data) -> Data {
    data.blocks = get_blocks(&data);
    data = get_class_name(data);
    data = set_block_types(data);

    data = attach_block_details(&data);
    data = get_additional_select_blocks(&data);
    data = attach_select_block_details(&data);

    // println!("blocks: {:?}", data.blocks);

    data
}

fn get_block_boundry(line: &String) -> Option<char> {
    if !line.contains("{") && !line.contains("}") {
        return None;
    }

    let first_char = line.clone().replace(" ", "").chars().nth(0).unwrap();

    if first_char == '{' || first_char == '}' {
        return Some(first_char);
    }

    None
}

fn get_correct_index(blocks: &Vec<Block>) -> usize {
    let mut index = blocks.len() - 1;

    while blocks[index].end.is_some() {
        index -= 1;
    }

    index
}

fn get_blocks(data: &Data) -> Option<Vec<Block>> {
    let mut blocks = Vec::new();

    let mut open_count = 0;

    for (index, line) in data.lines.iter().enumerate() {
        let boundry = get_block_boundry(line);

        match boundry {
            Some('{') => {
                open_count += 1;
                blocks.push(Block {
                    start: index as u32,
                    end: None,
                    block_type: BlockType::Unknown,
                    details: None,
                });
            }
            Some('}') => {
                open_count -= 1;
                let correct_index = get_correct_index(&blocks);
                blocks[correct_index].end = Some(index as u32);
            }
            _ => (),
        }
    }

    assert!(open_count == 0, "openCount is not 0");

    Some(blocks)
}

fn find_class_block_index(data: &Data) -> Option<usize> {
    if let Some(blocks) = &data.blocks {
        for (index, block) in blocks.iter().enumerate() {
            let previous_line = &data.lines[block.start as usize - 1];
            let no_spaces = previous_line.replace(" ", "");
            let keywords = vec!["class", "public", "private", "protected"];

            let mut current_keyword: Option<&str> = None;

            for keyword in keywords {
                if no_spaces[0..keyword.len()] == *keyword {
                    current_keyword = Some(keyword);
                    break;
                }
            }

            if current_keyword == Some("class") {
                return Some(index);
            }

            if current_keyword.is_some() {
                let keyword_len = current_keyword.unwrap().len();

                let is_class = no_spaces[keyword_len..keyword_len + 5] == *"class";
                if is_class {
                    return Some(index);
                }
            }
        }
    }

    None
}

fn get_class_name(mut data: Data) -> Data {
    let class_block_index = find_class_block_index(&data);

    if class_block_index.is_none() || data.blocks.is_none() {
        return data;
    }

    let class_block_index = class_block_index.unwrap();

    if let Some(mut blocks) = data.blocks {
        blocks[class_block_index].block_type = BlockType::Class;
        let no_spaces = data.lines[blocks[class_block_index].start as usize - 1].replace(" ", "");
        let start = no_spaces.find("class").unwrap() + 5;
        let end = no_spaces.find(":").unwrap();

        data.class_name = Some(no_spaces[start..end].to_string());

        return Data {
            lines: data.lines,
            blocks: Some(blocks),
            class_name: data.class_name,
        };
    }

    data
}

fn get_blocktype_from_string(str: &str) -> BlockType {
    match str {
        "namespace" => BlockType::Namespace,
        "class" => BlockType::Class,
        "method" => BlockType::Method,
        "constructor" => BlockType::Constructor,
        "context" => BlockType::Context,
        "variable" => BlockType::Variable,
        "if" => BlockType::If,
        "select" => BlockType::Select,
        _ => BlockType::Unknown,
    }
}

fn determine_block_type(
    block: &Block,
    lines: &Vec<String>,
    class_name: &Option<String>,
) -> BlockType {
    let previous_line = &lines[block.start as usize - 1];
    let no_spaces = previous_line.replace(" ", "");

    let keywords = vec![
        "namespace",
        "class",
        "public",
        "private",
        "protected",
        "using",
        "var",
        "select",
        "if",
        "else",
    ];

    let mut current_keyword = "unknown";

    for keyword in keywords {
        if no_spaces.len() < keyword.len() {
            continue;
        }

        if no_spaces[0..keyword.len()] == *keyword {
            current_keyword = keyword;
            break;
        }
    }

    if current_keyword == "public" || current_keyword == "private" || current_keyword == "protected"
    {
        let keyword_len = current_keyword.len();

        if no_spaces[keyword_len..(keyword_len + 5)] == "class".to_string() {
            return BlockType::Class;
        }

        if class_name.is_some() {
            let class_name = class_name.clone().unwrap();
            let class_name_with_open_parens = format!("{}(", class_name);

            if no_spaces.contains(&class_name_with_open_parens) {
                return BlockType::Constructor;
            }
        }

        return BlockType::Method;
    }

    if current_keyword == "using" && no_spaces.contains("CreateContext") {
        return BlockType::Context;
    }

    if current_keyword == "var" {
        return BlockType::Variable;
    }

    if current_keyword == "else" {
        return BlockType::If;
    }

    get_blocktype_from_string(current_keyword)
}

fn set_block_types(mut data: Data) -> Data {
    if let Some(blocks) = &mut data.blocks {
        for block in blocks {
            let block_type = determine_block_type(&block, &data.lines, &data.class_name);
            block.block_type = block_type;
        }
    }

    data
}

fn get_additional_select_blocks(data: &Data) -> Data {
    let data = data.clone();
    let data_blocks = data.blocks.unwrap();

    let mut result = vec![];
    for block in &data_blocks {
        if block.block_type != BlockType::Context {
            continue;
        }

        let mut current_block_start: Option<u32> = None;
        for i in block.start..block.end.unwrap() {
            let line = &data.lines[i as usize];

            if line.contains(&format!(
                " {}.",
                match block.clone().details.unwrap() {
                    BlockDetails::ContextBlock { variable } => variable,
                    _ => panic!("Expected context block details"),
                }
            )) {
                current_block_start = Some(i);
            }

            if current_block_start.is_some() && line.contains("{") {
                current_block_start = None;
            }

            if current_block_start.is_some()
                && (line.contains(".Add")
                    || line.contains(".SaveChanges")
                    || line.contains(".Remove"))
            {
                current_block_start = None;
            }

            if current_block_start.is_some() && line.contains(";") {
                result.push(Block {
                    start: current_block_start.unwrap(),
                    end: Some(i),
                    block_type: BlockType::Select,
                    details: None,
                });
                current_block_start = None;
            }
        }
    }

    Data {
        lines: data.lines,
        blocks: Some([data_blocks, result].concat()),
        class_name: data.class_name,
    }
}

fn get_httptype_from_string(str: &str) -> HttpType {
    match str {
        "HttpGet" => HttpType::HttpGet,
        "HttpPost" => HttpType::HttpPost,
        "HttpPut" => HttpType::HttpPut,
        "HttpDelete" => HttpType::HttpDelete,
        _ => panic!("Unknown http method"),
    }
}

fn determine_http_method(data: &Data, block: &Block) -> Option<HttpType> {
    let line = &data.lines[(block.start - 2) as usize];
    let no_spaces = line.replace(" ", "");

    if no_spaces.len() == 0 || !(no_spaces.as_bytes()[0] as char == '[') {
        return None;
    }

    let http_methods = vec!["HttpGet", "HttpPost", "HttpPut", "HttpDelete"];

    let mut current_http_method = None;
    for http_method in http_methods {
        if no_spaces[1..http_method.len() + 1] == http_method.to_string() {
            current_http_method = Some(http_method);
            break;
        }
    }

    if current_http_method.is_none() {
        return determine_http_method(
            data,
            &Block {
                start: block.start - 1,
                end: block.end,
                block_type: BlockType::Method,
                details: Some(BlockDetails::MethodBlock {
                    name: "".to_owned(),
                    http_method: None,
                    variables: vec![],
                    uses_context: false,
                }),
            },
        );
    }

    Some(get_httptype_from_string(current_http_method.unwrap()))
}

fn attach_block_details(data: &Data) -> Data {
    let new_data = data.clone();
    let mut data_blocks = new_data.blocks.unwrap();

    for index in 0..data_blocks.len() {
        let mut block = data_blocks[index].clone();
        if block.block_type == BlockType::Context {
            let prev_line = &new_data.lines[block.start as usize - 1];
            let variable = prev_line.trim_start()["using (var ".len()..]
                .split("=")
                .collect::<Vec<&str>>()[0]
                .trim()
                .to_string();

            block.details = Some(BlockDetails::ContextBlock { variable });
            data_blocks[index] = block;
        } else if block.block_type == BlockType::Method {
            let prev_line = &new_data.lines[block.start as usize - 1]
                .split(" ")
                .collect::<Vec<&str>>();
            let name_index = prev_line.iter().position(|word| word.contains("("));

            if name_index.is_none() {
                continue;
            }

            let name_index = name_index.unwrap();

            let end = prev_line[name_index].find("(").unwrap();
            let function_name = prev_line[name_index][0..end].to_string();

            let mut variables = vec![];

            let mut previous_type = prev_line[name_index][end + 1..].to_string();
            let mut next_index = name_index + 1;

            while next_index < prev_line.len() {
                let has_comma = prev_line[next_index].contains(",");
                let has_end = prev_line[next_index].contains(")");

                if !has_comma && !has_end {
                    previous_type = prev_line[next_index].to_string();
                } else {
                    variables.push(Variable {
                        name: prev_line[next_index].replace(",", "").replace(")", ""),
                        variable_type: previous_type.clone(),
                    });
                }

                next_index += 1;
            }

            block.details = Some(BlockDetails::MethodBlock {
                name: function_name,
                variables,
                http_method: determine_http_method(&data, &block),
                uses_context: data_blocks.iter().any(|b| {
                    b.start > block.start
                        && b.end.unwrap() < block.end.unwrap()
                        && b.block_type == BlockType::Context
                }),
            });

            data_blocks[index] = block;
        } else if block.block_type == BlockType::Variable {
            let parts = new_data.lines[block.start as usize - 1]
                .trim_start()
                .split(" ")
                .collect::<Vec<&str>>();

            let name = parts[1].to_string();
            let data_type = if parts[2..].contains(&"new") {
                let correct_data_type =
                    parts[2..].iter().position(|a| *a == "new").unwrap() + 1 + 2;
                parts[correct_data_type].replace("()", "")
            } else {
                parts[0].to_string()
            };

            block.details = Some(BlockDetails::VariableBlock { name, data_type });
            data_blocks[index] = block;
        } else if block.block_type == BlockType::If {
            let prev_line = &new_data.lines[block.start as usize - 1].trim_start();

            let first_four_chars = if prev_line.len() >= 4 {
                &prev_line[0..4]
            } else {
                ""
            };

            let has_else = first_four_chars == "else";

            block.details = Some(BlockDetails::IfBlock {
                clause: if has_else && !prev_line.contains("if") {
                    "".to_string()
                } else {
                    let splits = prev_line.trim_start().split("(").collect::<Vec<&str>>();
                    splits[1][0..(splits[1].len() - 1)].to_string()
                },
                is_else: has_else,
            });
            data_blocks[index] = block;
        }
    }

    Data {
        lines: new_data.lines,
        blocks: Some(data_blocks),
        class_name: new_data.class_name,
    }
}

fn sort_and_save_frequency(
    tables: &Vec<Table>,
    return_data: &Vec<ReturnData>,
) -> (Vec<Table>, Vec<ReturnData>) {
    let mut tables = tables.clone();
    let mut map: HashMap<String, i32> = HashMap::new();

    for data in return_data.clone() {
        let old_value = map.get(&data.table).unwrap_or(&0);
        map.insert(data.table, *old_value + 1);
    }

    tables.iter_mut().for_each(|table| {
        table.return_frequency = map.get(&table.name).unwrap_or(&0).to_owned();
    });

    let mut return_data = return_data.clone();
    return_data.sort_by(|a, b| {
        map.get(&b.table)
            .unwrap_or(&0)
            .cmp(map.get(&a.table).unwrap_or(&0))
    });

    (tables, return_data)
}

fn attach_select_block_details(data: &Data) -> Data {
    let final_data = data.clone();
    let new_data = data.clone();
    let mut data_blocks = new_data.blocks.unwrap();

    for index in 0..data_blocks.len() {
        let block = data_blocks[index].clone();
        if block.block_type != BlockType::Select {
            continue;
        }

        let mut block_syntax = LinqSyntax::Query;
        let mut where_clauses = vec![];
        let mut tables = vec![];
        let mut has_return = false;
        let mut query_type = QueryType::Many;
        let mut return_data = vec![];

        for index in block.start..=block.end.unwrap() {
            let line = &data.lines[index as usize];

            if index == block.start {
                if line.contains(".Where") {
                    block_syntax = LinqSyntax::Lambda;
                }

                if block_syntax == LinqSyntax::Query {
                    let keywords = ["from", "join", "where", "select", "&&"];

                    let mut current_index = (index - 1) as usize;
                    while keywords
                        .iter()
                        .any(|word| data.lines[current_index].contains(word))
                    {
                        let line = &data.lines[current_index].trim_start();
                        if line.contains("where") || line.contains("&&") {
                            let parts = line.clone().split(" ").collect::<Vec<&str>>();

                            let properties = parts
                                .iter()
                                .filter(|o| o.contains("."))
                                .collect::<Vec<&&str>>();

                            let mut local_shortcut = vec![];
                            let mut local_property = vec![];
                            for property in properties {
                                let parts = property.split(".").collect::<Vec<&str>>();

                                let shortcut = parts[0].to_string();
                                let property = parts[1].to_string();

                                local_shortcut.push(shortcut);
                                local_property.push(property);
                            }

                            if local_shortcut.len() > 0 {
                                where_clauses.push(WhereClause {
                                    shortcut: local_shortcut,
                                    property: local_property,
                                    value: data.lines[current_index].trim_start().to_string(),
                                    lambda_varible: None,
                                });
                            }

                            current_index -= 1;
                            continue;
                        }

                        if data.lines[current_index].contains("join") {
                            let keyword_index = data.lines[current_index].find("join").unwrap();
                            let keyword_string =
                                data.lines[current_index][keyword_index..].to_string();

                            let parts = keyword_string.split(" ").collect::<Vec<&str>>();

                            let shortcut = parts[1].to_string();
                            let table_name = parts[3].to_string().replace("cx.", "");

                            tables.push(Table {
                                name: table_name,
                                shortcut,
                                joined_tables: vec![],
                                return_frequency: 0,
                            });

                            current_index -= 1;
                            continue;
                        }

                        if data.lines[current_index].contains("from") {
                            let keyword_index = data.lines[current_index].find("from").unwrap();
                            let keyword_string =
                                data.lines[current_index][keyword_index..].to_string();
                            let parts = keyword_string.split(" ").collect::<Vec<&str>>();

                            let shortcut = parts[1].to_string();
                            let table_name = parts[3].to_string().replace("cx.", "");

                            tables.push(Table {
                                name: table_name,
                                shortcut,
                                joined_tables: vec![],
                                return_frequency: 0,
                            });

                            if data.lines[current_index].contains("return ") {
                                has_return = true;
                            }

                            current_index -= 1;
                            continue;
                        }

                        current_index -= 1;
                    }
                }

                if block_syntax == LinqSyntax::Lambda {
                    let start_sub = line.find(".Where(").unwrap() + 7;
                    let end_sub = line.find(")").unwrap();

                    let split_values = line[start_sub..end_sub].split("=>").collect::<Vec<&str>>();

                    let lambda_varible = Some(split_values[0].trim().to_string());
                    let value = split_values[1].trim().to_string();

                    let properties = value.split(" ").filter(|o| o.contains("."));

                    let mut shortcut = vec![];
                    let mut property = vec![];
                    for prop in properties {
                        let parts = prop.split(".").collect::<Vec<&str>>();
                        shortcut.push(parts[0].to_string());
                        property.push(parts[1].to_string());
                    }

                    if shortcut.len() > 0 {
                        where_clauses.push(WhereClause {
                            shortcut,
                            property,
                            value,
                            lambda_varible,
                        });
                    }

                    let words = line.split(" ").filter(|o| o.contains("cx."));

                    for word in words {
                        let parts = word.split(".").collect::<Vec<&str>>();

                        tables.push(Table {
                            name: parts[1].to_string(),
                            shortcut: parts[0].to_string(),
                            joined_tables: vec![],
                            return_frequency: 0,
                        });
                    }

                    if line.contains("return ") {
                        has_return = true;
                    }
                }

                if block.start != block.end.unwrap() {
                    continue;
                }
            }

            if index == block.end.unwrap() {
                let final_line = if line.contains(")") {
                    line
                } else {
                    &data.lines[index as usize + 1]
                };

                if final_line.contains(".SingleOrDefault") {
                    query_type = QueryType::Unique;
                }

                if final_line.contains(".FirstOrDefault") {
                    query_type = QueryType::First;
                }

                continue;
            }

            let trimmed_line = line.trim_start().replace(",", "");
            let parts = trimmed_line.split(" ").collect::<Vec<&str>>();

            let property = parts[0].to_string();
            let value = parts[2..].join(" ");

            let mut shortcut = None;
            let mut value = value;

            if value.contains(".") {
                let parts = value.split(".").collect::<Vec<&str>>();
                shortcut = Some(parts[0].to_string());
                value = parts[1].to_string();
            }

            let mut table = "".to_string();

            if shortcut.is_some() {
                let shortcut = shortcut.unwrap();
                let correct_table = tables.iter().find(|o| o.shortcut == shortcut);

                if let Some(correct_table) = correct_table {
                    table = correct_table.name.clone();
                }
            }

            return_data.push(ReturnData {
                property,
                table,
                value,
            });
        }

        let (tables, return_data) = sort_and_save_frequency(&tables, &return_data);

        data_blocks[index].details = Some(BlockDetails::SelectBlock {
            query_type,
            return_data,
            tables,
            where_clauses,
            syntax: block_syntax,
            has_return,
        });
    }

    Data {
        lines: final_data.lines,
        blocks: Some(data_blocks),
        class_name: final_data.class_name,
    }
}

#[cfg(test)]
mod tests {
    use std::vec;

    use super::analyze_lines;
    use crate::{Block, BlockDetails, BlockType, Data, HttpType, LinqSyntax, QueryType};

    #[test]
    fn analyze_data_input_1() {
        let input = std::fs::read_to_string("./tests/mocks/input.cs")
            .expect("Something went wrong reading the file");

        let lines = input.lines().collect::<Vec<&str>>();

        let mut data = Data {
            lines: lines.iter().map(|x| x.to_string()).collect::<Vec<String>>(),
            class_name: None,
            blocks: None,
        };

        data = analyze_lines(data);

        // parent data
        let parent_data = data.clone();
        assert!(parent_data.class_name.is_some());
        assert_eq!(parent_data.class_name.unwrap(), "TestController");
        assert!(parent_data.blocks.is_some());
        assert_eq!(parent_data.blocks.unwrap().len(), 33);

        // method blocks
        let method_blocks = data.clone().blocks.unwrap();
        let methods = method_blocks
            .iter()
            .filter(|block| block.block_type == BlockType::Method)
            .collect::<Vec<&Block>>();

        assert_eq!(methods.len(), 8);

        let non_http_methods = methods
            .iter()
            .filter(|block| match &block.details {
                Some(BlockDetails::MethodBlock {
                    http_method,
                    name: _,
                    variables: _,
                    uses_context: _,
                }) => http_method.is_none(),
                _ => false,
            })
            .collect::<Vec<&&Block>>();

        assert_eq!(non_http_methods.len(), 2);

        assert_eq!(non_http_methods[0].start, 154);
        assert!(non_http_methods[0].end.is_some());
        assert_eq!(non_http_methods[0].end.unwrap(), 165);
        assert!(non_http_methods[0].details.is_some());
        let details = non_http_methods[0].details.clone().unwrap();

        if let BlockDetails::MethodBlock {
            name,
            http_method,
            variables,
            uses_context,
        } = details
        {
            assert_eq!(name, "UpdateUserTask");
            assert_eq!(uses_context, true);
            assert_eq!(variables.len(), 1);
            assert_eq!(http_method, None);
            assert_eq!(variables[0].name, "userTaskDetails");
            assert_eq!(variables[0].variable_type, "UserTaskToAdd");
        } else {
            assert!(false);
        }

        assert_eq!(non_http_methods[1].start, 168);
        assert!(non_http_methods[1].end.is_some());
        assert_eq!(non_http_methods[1].end.unwrap(), 183);
        assert!(non_http_methods[1].details.is_some());
        let details = non_http_methods[1].details.clone().unwrap();

        if let BlockDetails::MethodBlock {
            name,
            http_method,
            variables,
            uses_context,
        } = details
        {
            assert_eq!(name, "AddUserTask");
            assert_eq!(uses_context, true);
            assert_eq!(variables.len(), 1);
            assert_eq!(http_method, None);
            assert_eq!(variables[0].name, "userTaskDetails");
            assert_eq!(variables[0].variable_type, "UserTaskToAdd");
        } else {
            assert!(false);
        }

        // http method blocks
        let http_methods = methods
            .iter()
            .filter(|block| match &block.details {
                Some(BlockDetails::MethodBlock {
                    http_method,
                    name: _,
                    variables: _,
                    uses_context: _,
                }) => http_method.is_some(),
                _ => false,
            })
            .collect::<Vec<&&Block>>();

        assert_eq!(http_methods.len(), 6);

        let get_http_methods = http_methods
            .iter()
            .filter(|block| match &block.details {
                Some(BlockDetails::MethodBlock {
                    http_method,
                    name: _,
                    variables: _,
                    uses_context: _,
                }) => http_method.clone().unwrap() == HttpType::HttpGet,
                _ => false,
            })
            .collect::<Vec<&&&Block>>();

        assert_eq!(get_http_methods[0].start, 58);
        assert!(get_http_methods[0].end.is_some());
        assert_eq!(get_http_methods[0].end.unwrap(), 78);
        assert!(get_http_methods[0].details.is_some());
        let details = get_http_methods[0].details.clone().unwrap();

        if let BlockDetails::MethodBlock {
            name,
            http_method,
            variables,
            uses_context,
        } = details
        {
            assert_eq!(name, "getClientTodoTasks");
            assert_eq!(uses_context, true);
            assert_eq!(variables.len(), 1);
            assert_eq!(http_method, Some(HttpType::HttpGet));
            assert_eq!(variables[0].name, "userOid");
            assert_eq!(variables[0].variable_type, "Guid");
        } else {
            assert!(false);
        }

        assert_eq!(get_http_methods[1].start, 82);
        assert!(get_http_methods[1].end.is_some());
        assert_eq!(get_http_methods[1].end.unwrap(), 100);
        assert!(get_http_methods[1].details.is_some());
        let details = get_http_methods[1].details.clone().unwrap();

        if let BlockDetails::MethodBlock {
            name,
            http_method,
            variables,
            uses_context,
        } = details
        {
            assert_eq!(name, "GetUserTaskDetails");
            assert_eq!(uses_context, true);
            assert_eq!(variables.len(), 1);
            assert_eq!(http_method, Some(HttpType::HttpGet));
            assert_eq!(variables[0].name, "userTaskOid");
            assert_eq!(variables[0].variable_type, "Guid");
        } else {
            assert!(false);
        }

        let post_http_methods = http_methods
            .iter()
            .filter(|block| match &block.details {
                Some(BlockDetails::MethodBlock {
                    http_method,
                    name: _,
                    variables: _,
                    uses_context: _,
                }) => http_method.clone().unwrap() == HttpType::HttpPost,
                _ => false,
            })
            .collect::<Vec<&&&Block>>();

        assert_eq!(post_http_methods[0].start, 25);
        assert!(post_http_methods[0].end.is_some());
        assert_eq!(post_http_methods[0].end.unwrap(), 54);
        assert!(post_http_methods[0].details.is_some());
        let details = post_http_methods[0].details.clone().unwrap();

        if let BlockDetails::MethodBlock {
            name,
            http_method,
            variables,
            uses_context,
        } = details
        {
            assert_eq!(name, "AddAdmin");
            assert_eq!(uses_context, true);
            assert_eq!(variables.len(), 1);
            assert_eq!(http_method, Some(HttpType::HttpPost));
            assert_eq!(variables[0].name, "adminName");
            assert_eq!(variables[0].variable_type, "AdminName");
        } else {
            assert!(false);
        }

        assert_eq!(post_http_methods[1].start, 119);
        assert!(post_http_methods[1].end.is_some());
        assert_eq!(post_http_methods[1].end.unwrap(), 137);
        assert!(post_http_methods[1].details.is_some());
        let details = post_http_methods[1].details.clone().unwrap();

        if let BlockDetails::MethodBlock {
            name,
            http_method,
            variables,
            uses_context,
        } = details
        {
            assert_eq!(name, "AddUpdateUserTask");
            assert_eq!(uses_context, false);
            assert_eq!(variables.len(), 1);
            assert_eq!(http_method, Some(HttpType::HttpPost));
            assert_eq!(variables[0].name, "userTaskDetails");
            assert_eq!(variables[0].variable_type, "UserTaskToAdd");
        } else {
            assert!(false);
        }

        let put_http_methods = http_methods
            .iter()
            .filter(|block| match &block.details {
                Some(BlockDetails::MethodBlock {
                    http_method,
                    name: _,
                    variables: _,
                    uses_context: _,
                }) => http_method.clone().unwrap() == HttpType::HttpPut,
                _ => false,
            })
            .collect::<Vec<&&&Block>>();

        assert_eq!(put_http_methods[0].start, 104);
        assert!(put_http_methods[0].end.is_some());
        assert_eq!(put_http_methods[0].end.unwrap(), 115);
        assert!(put_http_methods[0].details.is_some());
        let details = put_http_methods[0].details.clone().unwrap();

        if let BlockDetails::MethodBlock {
            name,
            http_method,
            variables,
            uses_context,
        } = details
        {
            assert_eq!(name, "CompleteTask");
            assert_eq!(uses_context, true);
            assert_eq!(variables.len(), 1);
            assert_eq!(http_method, Some(HttpType::HttpPut));
            assert_eq!(variables[0].name, "userTaskOid");
            assert_eq!(variables[0].variable_type, "Guid");
        } else {
            assert!(false);
        }

        let delete_http_methods = http_methods
            .iter()
            .filter(|block| match &block.details {
                Some(BlockDetails::MethodBlock {
                    http_method,
                    name: _,
                    variables: _,
                    uses_context: _,
                }) => http_method.clone().unwrap() == HttpType::HttpDelete,
                _ => false,
            })
            .collect::<Vec<&&&Block>>();

        assert_eq!(delete_http_methods[0].start, 141);
        assert!(delete_http_methods[0].end.is_some());
        assert_eq!(delete_http_methods[0].end.unwrap(), 151);
        assert!(delete_http_methods[0].details.is_some());
        let details = delete_http_methods[0].details.clone().unwrap();

        if let BlockDetails::MethodBlock {
            name,
            http_method,
            variables,
            uses_context,
        } = details
        {
            assert_eq!(name, "DeleteUserTask");
            assert_eq!(uses_context, true);
            assert_eq!(variables.len(), 1);
            assert_eq!(http_method, Some(HttpType::HttpDelete));
            assert_eq!(variables[0].name, "userTaskOid");
            assert_eq!(variables[0].variable_type, "Guid");
        } else {
            assert!(false);
        }

        // Variables
        let variable_blocks = data.clone().blocks.unwrap();

        let variables = variable_blocks
            .iter()
            .filter(|block| block.block_type == BlockType::Variable)
            .collect::<Vec<&Block>>();

        assert_eq!(variables.len(), 3);

        assert_eq!(variables[0].start, 34);
        assert!(variables[0].end.is_some());
        assert_eq!(variables[0].end.unwrap(), 41);
        assert!(variables[0].details.is_some());
        let details = variables[0].details.clone().unwrap();

        if let BlockDetails::VariableBlock { name, data_type } = details {
            assert_eq!(name, "user");
            assert_eq!(data_type, "User");
        } else {
            assert!(false);
        }

        assert_eq!(variables[1].start, 45);
        assert!(variables[1].end.is_some());
        assert_eq!(variables[1].end.unwrap(), 49);
        assert!(variables[1].details.is_some());
        let details = variables[1].details.clone().unwrap();

        if let BlockDetails::VariableBlock { name, data_type } = details {
            assert_eq!(name, "admin");
            assert_eq!(data_type, "Admin");
        } else {
            assert!(false);
        }

        assert_eq!(variables[2].start, 172);
        assert!(variables[2].end.is_some());
        assert_eq!(variables[2].end.unwrap(), 179);
        assert!(variables[2].details.is_some());
        let details = variables[2].details.clone().unwrap();

        if let BlockDetails::VariableBlock { name, data_type } = details {
            assert_eq!(name, "userTask");
            assert_eq!(data_type, "UserTask");
        } else {
            assert!(false);
        }

        // Select Block Details
        let select_blocks = data.clone().blocks.unwrap();
        let selects: Vec<&Block> = select_blocks
            .iter()
            .filter(|block| block.block_type == BlockType::Select)
            .collect::<Vec<&Block>>();

        assert_eq!(selects.len(), 5);

        let current_select = selects[0];
        assert_eq!(current_select.start, 66);
        assert!(current_select.end.is_some());
        assert_eq!(current_select.end.unwrap(), 75);
        assert!(current_select.details.is_some());
        let details = current_select.details.clone().unwrap();

        if let BlockDetails::SelectBlock {
            query_type,
            tables,
            where_clauses,
            return_data,
            syntax,
            has_return,
        } = details
        {
            assert_eq!(query_type, QueryType::Many);
            assert_eq!(syntax, LinqSyntax::Query);
            assert_eq!(has_return, true);

            assert_eq!(tables.len(), 3);

            assert_eq!(tables[0].name, "TaskStatuses");
            assert_eq!(tables[0].shortcut, "uts");
            assert_eq!(tables[0].joined_tables, vec![]);
            assert_eq!(tables[0].return_frequency, 1);

            assert_eq!(tables[1].name, "Users");
            assert_eq!(tables[1].shortcut, "u");
            assert_eq!(tables[1].joined_tables, vec![]);
            assert_eq!(tables[1].return_frequency, 1);

            assert_eq!(tables[2].name, "UserTasks");
            assert_eq!(tables[2].shortcut, "ut");
            assert_eq!(tables[2].joined_tables, vec![]);
            assert_eq!(tables[2].return_frequency, 6);

            assert_eq!(where_clauses.len(), 1);

            assert_eq!(where_clauses[0].property, vec!["UserOid"]);
            assert_eq!(where_clauses[0].shortcut, vec!["ut"]);
            assert_eq!(where_clauses[0].value, "where ut.UserOid == userOid");

            assert_eq!(return_data.len(), 8);

            assert_eq!(return_data[0].property, "UserTaskOid");
            assert_eq!(return_data[0].table, "UserTasks");
            assert_eq!(return_data[0].value, "UserTaskOid");

            assert_eq!(return_data[1].property, "Name");
            assert_eq!(return_data[1].table, "UserTasks");
            assert_eq!(return_data[1].value, "Name");

            assert_eq!(return_data[2].property, "CompleteDate");
            assert_eq!(return_data[2].table, "UserTasks");
            assert_eq!(return_data[2].value, "CompleteDate");

            assert_eq!(return_data[3].property, "TaskStatusId");
            assert_eq!(return_data[3].table, "UserTasks");
            assert_eq!(return_data[3].value, "TaskStatusId");

            assert_eq!(return_data[4].property, "StartDate");
            assert_eq!(return_data[4].table, "UserTasks");
            assert_eq!(return_data[4].value, "StartDate");

            assert_eq!(return_data[5].property, "OrderNumber");
            assert_eq!(return_data[5].table, "UserTasks");
            assert_eq!(return_data[5].value, "OrderNumber");

            assert_eq!(return_data[6].property, "UserOid");
            assert_eq!(return_data[6].table, "Users");
            assert_eq!(return_data[6].value, "UserOid");

            assert_eq!(return_data[7].property, "TaskStatus");
            assert_eq!(return_data[7].table, "TaskStatuses");
            assert_eq!(return_data[7].value, "Name");
        } else {
            assert!(false);
        }

        let current_select = selects[1];
        assert_eq!(current_select.start, 89);
        assert!(current_select.end.is_some());
        assert_eq!(current_select.end.unwrap(), 98);
        assert!(current_select.details.is_some());
        let details = current_select.details.clone().unwrap();

        if let BlockDetails::SelectBlock {
            query_type,
            tables,
            where_clauses,
            return_data,
            syntax,
            has_return,
        } = details
        {
            assert_eq!(query_type, QueryType::Unique);
            assert_eq!(syntax, LinqSyntax::Query);
            assert_eq!(has_return, true);

            assert_eq!(tables.len(), 2);

            assert_eq!(tables[0].name, "TaskStatuses");
            assert_eq!(tables[0].shortcut, "uts");
            assert_eq!(tables[0].joined_tables, vec![]);
            assert_eq!(tables[0].return_frequency, 1);

            assert_eq!(tables[1].name, "UserTasks");
            assert_eq!(tables[1].shortcut, "ut");
            assert_eq!(tables[1].joined_tables, vec![]);
            assert_eq!(tables[1].return_frequency, 7);

            assert_eq!(where_clauses.len(), 1);

            assert_eq!(where_clauses[0].property, vec!["UserTaskOid"]);
            assert_eq!(where_clauses[0].shortcut, vec!["ut"]);
            assert_eq!(
                where_clauses[0].value,
                "where ut.UserTaskOid == userTaskOid"
            );

            assert_eq!(return_data.len(), 8);

            assert_eq!(return_data[0].property, "UserOid");
            assert_eq!(return_data[0].table, "UserTasks");
            assert_eq!(return_data[0].value, "UserOid");

            assert_eq!(return_data[1].property, "UserTaskOid");
            assert_eq!(return_data[1].table, "UserTasks");
            assert_eq!(return_data[1].value, "UserTaskOid");

            assert_eq!(return_data[2].property, "Name");
            assert_eq!(return_data[2].table, "UserTasks");
            assert_eq!(return_data[2].value, "Name");

            assert_eq!(return_data[3].property, "CompleteDate");
            assert_eq!(return_data[3].table, "UserTasks");
            assert_eq!(return_data[3].value, "CompleteDate");

            assert_eq!(return_data[4].property, "TaskStatusId");
            assert_eq!(return_data[4].table, "UserTasks");
            assert_eq!(return_data[4].value, "TaskStatusId");

            assert_eq!(return_data[5].property, "StartDate");
            assert_eq!(return_data[5].table, "UserTasks");
            assert_eq!(return_data[5].value, "StartDate");

            assert_eq!(return_data[6].property, "OrderNumber");
            assert_eq!(return_data[6].table, "UserTasks");
            assert_eq!(return_data[6].value, "OrderNumber");

            assert_eq!(return_data[7].property, "TaskStatus");
            assert_eq!(return_data[7].table, "TaskStatuses");
            assert_eq!(return_data[7].value, "Name");
        } else {
            assert!(false);
        }

        let current_select = selects[2];
        assert_eq!(current_select.start, 107);
        assert!(current_select.end.is_some());
        assert_eq!(current_select.end.unwrap(), 107);
        assert!(current_select.details.is_some());
        let details = current_select.details.clone().unwrap();

        if let BlockDetails::SelectBlock {
            query_type,
            tables,
            where_clauses,
            return_data,
            syntax,
            has_return,
        } = details
        {
            assert_eq!(query_type, QueryType::Unique);
            assert_eq!(syntax, LinqSyntax::Lambda);
            assert_eq!(has_return, false);

            assert_eq!(tables.len(), 1);

            assert_eq!(tables[0].name, "UserTasks");
            assert_eq!(tables[0].shortcut, "cx");
            assert_eq!(tables[0].joined_tables, vec![]);
            assert_eq!(tables[0].return_frequency, 0);

            assert_eq!(where_clauses.len(), 1);

            assert_eq!(where_clauses[0].property, vec!["UserTaskOid"]);
            assert_eq!(where_clauses[0].shortcut, vec!["t"]);
            assert_eq!(where_clauses[0].value, "t.UserTaskOid == userTaskOid");
            assert_eq!(where_clauses[0].lambda_varible, Some("t".to_string()));

            assert_eq!(return_data.len(), 0);
        } else {
            assert!(false);
        }

        let current_select = selects[3];
        assert_eq!(current_select.start, 144);
        assert!(current_select.end.is_some());
        assert_eq!(current_select.end.unwrap(), 144);
        assert!(current_select.details.is_some());
        let details = current_select.details.clone().unwrap();

        if let BlockDetails::SelectBlock {
            query_type,
            tables,
            where_clauses,
            return_data,
            syntax,
            has_return,
        } = details
        {
            assert_eq!(query_type, QueryType::Unique);
            assert_eq!(syntax, LinqSyntax::Lambda);
            assert_eq!(has_return, false);

            assert_eq!(tables.len(), 1);

            assert_eq!(tables[0].name, "UserTasks");
            assert_eq!(tables[0].shortcut, "cx");
            assert_eq!(tables[0].joined_tables, vec![]);
            assert_eq!(tables[0].return_frequency, 0);

            assert_eq!(where_clauses.len(), 1);

            assert_eq!(where_clauses[0].property, vec!["UserTaskOid"]);
            assert_eq!(where_clauses[0].shortcut, vec!["t"]);
            assert_eq!(where_clauses[0].value, "t.UserTaskOid == userTaskOid");
            assert_eq!(where_clauses[0].lambda_varible, Some("t".to_string()));

            assert_eq!(return_data.len(), 0);
        } else {
            assert!(false);
        }

        let current_select = selects[4];
        assert_eq!(current_select.start, 157);
        assert!(current_select.end.is_some());
        assert_eq!(current_select.end.unwrap(), 157);
        assert!(current_select.details.is_some());
        let details = current_select.details.clone().unwrap();

        if let BlockDetails::SelectBlock {
            query_type,
            tables,
            where_clauses,
            return_data,
            syntax,
            has_return,
        } = details
        {
            assert_eq!(query_type, QueryType::Unique);
            assert_eq!(syntax, LinqSyntax::Lambda);
            assert_eq!(has_return, false);

            assert_eq!(tables.len(), 1);

            assert_eq!(tables[0].name, "UserTasks");
            assert_eq!(tables[0].shortcut, "cx");
            assert_eq!(tables[0].joined_tables, vec![]);
            assert_eq!(tables[0].return_frequency, 0);

            assert_eq!(where_clauses.len(), 1);

            assert_eq!(
                where_clauses[0].property,
                vec!["UserTaskOid", "UserTaskOid"]
            );
            assert_eq!(where_clauses[0].shortcut, vec!["t", "userTaskDetails"]);
            assert_eq!(
                where_clauses[0].value,
                "t.UserTaskOid == userTaskDetails.UserTaskOid"
            );
            assert_eq!(where_clauses[0].lambda_varible, Some("t".to_string()));

            assert_eq!(return_data.len(), 0);
        } else {
            assert!(false);
        }

        // If Blocks
        let if_blocks = data.clone().blocks.unwrap();

        let ifs = if_blocks
            .iter()
            .filter(|block| block.block_type == BlockType::If)
            .collect::<Vec<&Block>>();

        assert_eq!(ifs.len(), 5);

        let current_if = ifs[0];
        assert_eq!(current_if.start, 109);
        assert!(current_if.end.is_some());
        assert_eq!(current_if.end.unwrap(), 113);
        assert!(current_if.details.is_some());
        let details = current_if.details.clone().unwrap();

        if let BlockDetails::IfBlock { clause, is_else } = details {
            assert_eq!(clause, "userTask != null");
            assert_eq!(is_else, false);
        } else {
            assert!(false);
        }

        let current_if = ifs[1];
        assert_eq!(current_if.start, 123);
        assert!(current_if.end.is_some());
        assert_eq!(current_if.end.unwrap(), 125);
        assert!(current_if.details.is_some());
        let details = current_if.details.clone().unwrap();

        if let BlockDetails::IfBlock { clause, is_else } = details {
            assert_eq!(clause, "userTaskDetails.UserTaskOid == Guid.Empty");
            assert_eq!(is_else, false);
        } else {
            assert!(false);
        }

        let current_if = ifs[2];
        assert_eq!(current_if.start, 127);
        assert!(current_if.end.is_some());
        assert_eq!(current_if.end.unwrap(), 129);
        assert!(current_if.details.is_some());
        let details = current_if.details.clone().unwrap();

        if let BlockDetails::IfBlock { clause, is_else } = details {
            assert_eq!(clause, "");
            assert_eq!(is_else, true);
        } else {
            assert!(false);
        }

        let current_if = ifs[3];
        assert_eq!(current_if.start, 146);
        assert!(current_if.end.is_some());
        assert_eq!(current_if.end.unwrap(), 149);
        assert!(current_if.details.is_some());
        let details = current_if.details.clone().unwrap();

        if let BlockDetails::IfBlock { clause, is_else } = details {
            assert_eq!(clause, "userTask != null");
            assert_eq!(is_else, false);
        } else {
            assert!(false);
        }

        let current_if = ifs[4];
        assert_eq!(current_if.start, 159);
        assert!(current_if.end.is_some());
        assert_eq!(current_if.end.unwrap(), 163);
        assert!(current_if.details.is_some());
        let details = current_if.details.clone().unwrap();

        if let BlockDetails::IfBlock { clause, is_else } = details {
            assert_eq!(clause, "userTask != null");
            assert_eq!(is_else, false);
        } else {
            assert!(false);
        }
    }

    #[test]
    fn analyze_data_input_2() {
        let input = std::fs::read_to_string("./tests/mocks/input2.cs")
            .expect("Something went wrong reading the file");

        let lines = input.lines().collect::<Vec<&str>>();

        let mut data = Data {
            lines: lines.iter().map(|x| x.to_string()).collect::<Vec<String>>(),
            class_name: None,
            blocks: None,
        };

        data = analyze_lines(data);

        assert!(data.class_name.is_some());
        assert_eq!(data.class_name.unwrap(), "Test2Controller");
        assert!(data.blocks.is_some());
        assert_eq!(data.blocks.unwrap().len(), 38);
    }
}
