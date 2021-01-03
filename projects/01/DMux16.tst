load DMux16.hdl,
output-file DMux16.out,
compare-to DMux16.cmp,
output-list in%B1.16.1 sel%B2.1.2 a%B1.16.1 b%B1.16.1;

set in 0,
set sel 0,
eval,
output;

set sel 1,
eval,
output;

set in %B0001001000110100,
set sel 0,
eval,
output;

set sel 1,
eval,
output;

