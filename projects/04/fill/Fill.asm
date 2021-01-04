// This file is part of www.nand2tetris.org
// and the book "The Elements of Computing Systems"
// by Nisan and Schocken, MIT Press.
// File name: projects/04/Fill.asm

// Runs an infinite loop that listens to the keyboard input.
// When a key is pressed (any key), the program blackens the screen,
// i.e. writes "black" in every pixel;
// the screen should remain fully black as long as the key is pressed. 
// When no key is pressed, the program clears the screen, i.e. writes
// "white" in every pixel;
// the screen should remain fully clear as long as no key is pressed.

// while true
(LOOP)

// @R0 = @SCREEN + 8K (counter)
@SCREEN
D=A
@8191
D=D+A
@R0
M=D

// GOTO @BLACK if *@KBD != 0
// GOTO @WHITE if *@KBD == 0
@KBD
D=M
@WHITE
D;JEQ

(BLACK)
// GOTO @ENDFILL IF @R0 < @SCREEN
@R0
D=M
@SCREEN
D=D-A
@ENDFILL
D;JLT

// *@R0 = -1
@R0
A=M
M=-1

// --@R0
@R0
M=M-1
// GOTO @BLACK
@BLACK
0;JMP

(WHITE)
// GOTO @ENDFILL IF @R0 < @SCREEN
@R0
D=M
@SCREEN
D=D-A
@ENDFILL
D;JLT

// *@R0 = -1
@R0
A=M
M=0

// --@R0
@R0
M=M-1
// GOTO @WHITE
@WHITE
0;JMP

(ENDFILL)
@LOOP
0;JMP

