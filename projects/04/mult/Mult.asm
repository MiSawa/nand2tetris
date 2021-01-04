// This file is part of www.nand2tetris.org
// and the book "The Elements of Computing Systems"
// by Nisan and Schocken, MIT Press.
// File name: projects/04/Mult.asm

// Multiplies R0 and R1 and stores the result in R2.
// (R0, R1, R2 refer to RAM[0], RAM[1], and RAM[2], respectively.)

// Put your code here.

// reset R2 zero
@R2
M=0

// while R0 > 0
(LOOP)
// GOTO END IF R0 <= 0
@R0
D=M
@END
D;JLE

// R0 -= 1
@R0
M=M-1

// R2 += R1
@R1
D=M
@R2
M=M+D

// GOTO LOOP
@LOOP
0;JMP

(END)
