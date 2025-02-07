// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

import {FuzzTest} from './FuzzTest.t.sol';
import {vm} from './Setup.t.sol';

contract {{ contract_name }} is FuzzTest {
{{ reproducers }}
}