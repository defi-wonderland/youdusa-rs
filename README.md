# Youdusa

This tool generate a foundry test for failling call sequences of a Medusa fuzz campaign.

Made with ♥ by Wonderland (https://defi.sucks)

## Installation
```bash
cargo install youdusa
```

## Usage
Either pipe the output of medusa fuzz (it will still log it to your console too):
```bash
medusa fuzz | youdusa
```

or using a file:
```bash
youdusa --file log.txt
```

## Example:
```markdown
(...)
⇾ [FAILED] Assertion Test: FuzzTest.prop_anyoneCanIncreaseFundInAPool(uint256,uint256)
Test for method "FuzzTest.prop_anyoneCanIncreaseFundInAPool(uint256,uint256)" resulted in an assertion failure after the following call sequence:
[Call Sequence]
1) FuzzTest.prop_alloOwnerCanAlwaysChangePercentFee(uint256)(15056796) (block=10429, time=19960, gas=12500000, gasprice=1, value=0, sender=0x0000000000000000000000000000000000050000)
2) FuzzTest.prop_anyoneCanIncreaseFundInAPool(uint256,uint256)(13441534537036768485200417184756697876915712920751763869415731560796441041418, 334437) (block=34180, time=321741, gas=12500000, gasprice=1, value=0, sender=0x0000000000000000000000000000000000070000)
[Execution Trace]
(...)
```

generates the following test function:
```solidity
function test_prop_anyoneCanIncreaseFundInAPool_1() public {
    vm.prank(0x0000000000000000000000000000000000050000);
    vm.warp(19960);
    vm.roll(10429);
    this.prop_alloOwnerCanAlwaysChangePercentFee(15056796);

    vm.prank(0x0000000000000000000000000000000000070000);
    vm.warp(321741);
    vm.roll(34180);
    this.prop_anyoneCanIncreaseFundInAPool(13441534537036768485200417184756697876915712920751763869415731560796441041418, 334437);
}
```
