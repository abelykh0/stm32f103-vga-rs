#define	HSIZE_CHARS  36

.syntax unified

@ Arguments:
@  r0  uint8_t*  font
@  r1  uint8_t*  characters
@  r2  uint32_t* attributes
@  r3  uint8_t*  dest
.section	.text.vga_draw,"ax",%progbits
@.section .ramcode,ax,%progbits
.balign 4
.global vga_draw
.thumb_func
vga_draw:
	@ parameters
	fnt   .req r0
	chr   .req r1
	col   .req r2
	odr   .req r3

	push {r4, r5, r6, r7, r8}      @ 6

	@ prepare
	ldrh r5, [chr], #2             @ 2* r5 = {character}
	ldr r6, [col], #4              @ 1* r6 = {character attribute}
	ldrb r5, [fnt, r5, LSL #3]     @ 1* r5 = {character pixels}
	lsr r4, r5, #2                 @ 1  r4 = {character pixels} >> 2
	and r4, r4, #60                @ 1  r4 &= 0B00111100

	.rept HSIZE_CHARS

		@ first 4 pixels, each pixel one byte (r7)

 		ldr r7, [r6, r4]           @ 2* r7 = {4 bytes for 1st part of character}
		strb r7, [odr]             @ 2

		ror r7, #8                 @ 1
 		and r4, r5, #15            @ 1  r4 = {character pixels} & 0B00001111
		strb r7, [odr]             @ 2

		ror r7, #8                 @ 1
 		ldr r8, [r6, r4, LSL #2]   @ 1* r8 = {4 bytes for 2nd part of character}
		strb r7, [odr]             @ 2

		ror r7, #8                 @ 1
 		ldr r6, [col], #4          @ 1* r6 = {character attribute}
		strb r7, [odr]             @ 2

		@ next 4 pixels, each pixel one byte  (r8)

		nop                        @ 1
		ldrh r5, [chr], #2         @ 1* r5 = {character}
		strb r8, [odr]             @ 2

		ror r8, #8                 @ 1
		ldrb r5, [fnt, r5, LSL #3] @ 1* r5 = {character pixels}
		strb r8, [odr]             @ 2

		ror r8, #8                 @ 1
 		lsr r4, r5, #2             @ 1  r4 = {character pixels} >> 2
		strb r8, [odr]             @ 2

		ror r8, #8                 @ 1
 		and r4, r4, #60            @ 1  r4 &= 0B00111100
		strb r8, [odr]             @ 2

	.endr

	@ 0 at the end
	mov r4, #0                     @ 1
	nop                            @ 1
	strb r4, [odr]                 @ 2

	pop	{r4, r5, r6, r7, r8}       @ 6
	bx lr
