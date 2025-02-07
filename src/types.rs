use primitive_types::U256;

#[derive(Debug)]
pub struct CheatsData {
    pub block_to_roll: i32,
    pub timestamp_to_warp_to: i32,
    pub caller_to_prank: String,
    pub value: U256,
}
