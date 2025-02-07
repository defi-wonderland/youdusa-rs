// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

import {FuzzTest} from './FuzzTest.t.sol';
import {vm} from './Setup.t.sol';

contract ForgeReproducer1 is FuzzTest {
    function test_prop_anyoneCanIncreaseFundInAPool() public {
        vm.roll(10429);
        vm.warp(19960);
        vm.prank(0x0000000000000000000000000000000000050000);
        this.prop_alloOwnerCanAlwaysChangePercentFee(15056796);

        vm.roll(34180);
        vm.warp(321741);
        vm.prank(0x0000000000000000000000000000000000070000);
        this.prop_anyoneCanIncreaseFundInAPool(13441534537036768485200417184756697876915712920751763869415731560796441041418, 334437,'');

    }

    function test_prop_anyoneCanIncreaseFundInAPool2() public {
        vm.roll(10429);
        vm.warp(19960);
        vm.prank(0x0000000000000000000000000000000000050000);
        this.prop_alloOwnerCanAlwaysChangePercentFee(15056796);

        vm.roll(34180);
        vm.warp(321741);
        vm.prank(0x0000000000000000000000000000000000070000);
        this.prop_anyoneCanIncreaseFundInAPool(23, 334437, (1, 2),'');

    }


}