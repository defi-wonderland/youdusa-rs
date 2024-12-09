// Todo: store all function get_common_name in a hashmap, then use it to check for duplicate

/// Youdusa AST, used to build representation of reproducers
/// Only the relevant part of Solidity semantics is implemented
/// Excluded (place-holdered if needed in the future): Expressions (function calls are represented as pure statement, as we don't get
/// their returned values), Other types, Other statements
#[derive(Debug, PartialEq)]
pub enum Ast {
    FunctionDeclaration(FunctionDeclaration), // Fn declaration, this is the root
    Statement(Statement),
}

#[derive(Debug, PartialEq)]
pub struct FunctionDeclaration {
    name: String,
    visibility: Visibility,
    arguments: Vec<Argument>,
    return_type: Type,
    children: Vec<Ast>,
}

impl FunctionDeclaration {
    // todo: builder?
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            visibility: Visibility::Public,
            arguments: Vec::new(),
            return_type: Type::None,
            children: Vec::new(),
        }
    }

    /// Parse the name and remove any trailing digit (used when checking for duplicates)
    pub fn get_common_name(&self) -> String {
        return self
            .name
            .trim_end_matches(|x: char| x.is_digit(10))
            .to_string();
    }

    pub fn add_child(&mut self, child: Ast) {
        self.children.push(child);
    }
}

#[derive(Debug, PartialEq)]
pub enum Visibility {
    Public,
    Private,
    Internal,
    External,
}

#[derive(Debug, PartialEq)]
pub struct Argument {
    name: String,
    type_: Type,
}

#[derive(Debug, PartialEq)]
pub enum Type {
    None,
    Uint256,
    Address,
    Bytes,
    Bytes32,
    String,
}

impl Type {
    const fn to_string(&self) -> &'static str {
        match self {
            Type::None => "",
            Type::Uint256 => "uint256",
            Type::Address => "address",
            Type::Bytes => "bytes memory",
            Type::Bytes32 => "bytes32",
            Type::String => "string memory",
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Statement {
    Prank(FunctionCall),
    Roll(FunctionCall),
    Warp(FunctionCall),
    ContractCall(FunctionCall),
}

impl Statement {
    pub fn new_prank(pranked_address: &str) -> Self {
        Self::Prank(FunctionCall {
            external_contract: Some("vm".to_string()),
            function_name: "prank".to_string(),
            value: None,
            arguments: vec![pranked_address.to_string()],
        })
    }

    pub fn new_roll(block_to_roll: i32) -> Self {
        Self::Roll(FunctionCall {
            external_contract: Some("vm".to_string()),
            function_name: "roll".to_string(),
            value: None,
            arguments: vec![block_to_roll.to_string()],
        })
    }

    pub fn new_warp(timestamp_to_warp_to: i32) -> Self {
        Self::Warp(FunctionCall {
            external_contract: Some("vm".to_string()),
            function_name: "warp".to_string(),
            value: None,
            arguments: vec![timestamp_to_warp_to.to_string()],
        })
    }

    pub fn new_contract_call(
        external_contract: Option<String>,
        function_name: String,
        value: Option<i32>,
        arguments: Vec<String>,
    ) -> Self {
        Self::ContractCall(FunctionCall {
            external_contract,
            function_name,
            value,
            arguments,
        })
    }
}

#[derive(Debug, PartialEq)]
pub struct FunctionCall {
    external_contract: Option<String>,
    function_name: String,
    value: Option<i32>,
    arguments: Vec<String>,
}
