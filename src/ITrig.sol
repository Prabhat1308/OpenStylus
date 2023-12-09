 pragma solidity ^0.8.0;

interface ITrig {
    function sin(uint256 _angle) external pure returns (int256);
    function cos(uint256 _angle) external pure returns (int256);
   
} 