// Todo: store all function get_common_name in a hashmap, then use it to check for duplicate

/// Youdusa AST, used to build representation of reproducers
/// Only the relevant part of Solidity semantics is implemented
/// Excluded (place-holdered if needed in the future): Expressions (function calls are represented as pure statement, as we don't get
/// their returned values), Other types, Other statements
pub enum Ast {
    Function(Function), // Fn declaration, this is the root
    Statement(Statement),
}

pub struct Function {
    name: String,
    visibility: Visibility,
    arguments: Vec<Argument>,
    return_type: Type,
    children: Vec<Ast>,
}

impl Function {
    /// Parse the name and remove any trailing digit (used when checking for duplicates)
    pub fn get_common_name(&self) -> String {
        return self
            .name
            .trim_end_matches(|x: char| x.is_digit(10))
            .to_string();
    }
}

pub enum Visibility {
    Public,
    Private,
    Internal,
    External,
}

pub struct Argument {
    name: String,
    type_: Type,
}

pub enum Type {
    Uint256,
    Address,
    Bytes,
    Bytes32,
    String,
}

impl Type {
    const fn to_string(&self) -> &'static str {
        match self {
            Type::Uint256 => "uint256",
            Type::Address => "address",
            Type::Bytes => "bytes memory",
            Type::Bytes32 => "bytes32",
            Type::String => "string memory",
        }
    }
}

pub enum Statement {
    Prank(FunctionCall),
    Roll(FunctionCall),
    Warp(FunctionCall),
    ContractCall(FunctionCall),
}

pub struct FunctionCall {
    target: String,
    arguments: Vec<String>,
}
