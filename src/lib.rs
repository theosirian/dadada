
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::iter::IntoIterator;
use std::cmp::PartialEq;
use pulldown_cmark::{Parser, html};

// `Block` stores code sections, consisting of comments and associated code.
// We initialise a new block with empty `Vec` which will later be joined.
pub struct Block {
    comment: Vec<String>,
    code: Vec<String>
}

impl Block {
    pub fn new() -> Block {
        return Block {
            comment: Vec::new(),
            code: Vec::new()
        }
    }

    pub fn new_file(title: &str, filename: &str) -> Block {
        return Block {
            comment: vec![format!("## `{:}`", title)],
            code: vec![format!("// File: {:}", filename)]
        }
    }
}

#[derive(PartialEq)]
enum CommentType {
    Simple,
    Bang,
    Doc,
    ANY
}

// We divide the source code into code/comment blocks.
// A `Vec` of `Block`s is returned for further processing.
pub fn extract(path: String) -> Vec<Block> {
    let file = File::open(path).expect("Unable to open the file");
    let mut process_as_code = false;
    let mut current_comment_type : CommentType = CommentType::ANY;
    let mut blocks: Vec<Block> = Vec::new();
    let mut current_block = Block::new();

    for line in BufReader::new(file).lines() {

        let line_str = line.unwrap().to_string();
        let stripped = line_str.trim();

        if stripped.starts_with("//") {
            if process_as_code {
                blocks.push(current_block);
                current_block = Block::new();
            }
            process_as_code = false;
        } else {
            process_as_code = true;
            current_comment_type = CommentType::ANY;
        }

        if process_as_code {
            current_block.code.push(line_str.to_string());
        } else {
            let (strip_pos, com_type) = {
                if stripped.starts_with("///") {
                    (3,  CommentType::Doc)
                } else if stripped.starts_with("//!") {
                    (3,  CommentType::Bang)
                } else if stripped.starts_with("// !") {
                    (4,  CommentType::Bang)
                } else {
                    (2,  CommentType::Simple)
                }
            };
            
            let line = stripped.split_at(strip_pos).1;
            if current_comment_type != CommentType::ANY &&
                    com_type != current_comment_type {
                // different type of comment, means we assume a new block
                blocks.push(current_block);
                current_block = Block::new();
            }
            current_comment_type = com_type;
            current_block.comment.push(line.trim().to_string());
        }
    }
    blocks.push(current_block);
    return blocks;
}

// Build a full HTML document from a vector of blocks.
// This function also inlines the CSS.
pub fn build_html<I: IntoIterator<Item=Block>>(blocks: I) -> String {
    let css = include_str!("style.css").to_string();
    let mut html_output = String::new();

    for (i, block) in blocks.into_iter().enumerate() {
        html_output.push_str(&format!(include_str!("block_before.html"), index=i));
        html::push_html(&mut html_output, Parser::new(&block.comment.join("\n")));
        html_output.push_str(&format!(include_str!("block_after.html"), code=block.code.join("\n").replace("<", "&lt;")));
    }

    return format!(include_str!("template.html"),
                       title="dada",
                       css=css,
                       blocks=html_output);
}