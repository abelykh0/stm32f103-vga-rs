#define HSIZE_CHARS 44

.syntax unified

@ Arguments:
@  r0  uint8_t*  pix
@  r1  uint32_t* attr_base - address of attribute definitions
@  r2  uint8_t*  attr - attributes
@  r3  uint8_t*  odr
.section .text.vga_draw_impl,"ax",%progbits
.balign 4
.global vga_draw_impl
.thumb_func
vga_draw_impl:
	@ parameters
	pix         .req r0
	attr_base   .req r1
	attr        .req r2
	odr         .req r3

	@ variables
	pixels      .req r4
	attr_offset .req r5
	tmp1        .req r6
	tmp2        .req r7

	push {r4, r5, r6, r7}

	@ prepare
	ldrb pixels, [pix], #1
	ldrb attr_offset, [attr], #1
	ror pixels, #4
	and tmp1, pixels, #15
	add tmp1, tmp1, attr_offset, LSL #4
	ldr tmp1, [attr_base, tmp1, LSL #2]

	.rept HSIZE_CHARS
		ror pixels, #28                             @ 1
		and tmp2, pixels, #15                       @ 1
		strb tmp1, [odr]                            @ 2

		ror tmp1, #8                                @ 1
		add tmp2, tmp2, attr_offset, LSL #4         @ 1
		strb tmp1, [odr]                            @ 2

		ror tmp1, #8                                @ 1
		ldr tmp2, [attr_base, tmp2, LSL #2]         @ 1*
		strb tmp1, [odr]                            @ 2

		ror tmp1, #8                                @ 1
		ldrb pixels, [pix], #1                      @ 1*
		strb tmp1, [odr]                            @ 2

		ror pixels, #4                              @ 1
		and tmp1, pixels, #15                       @ 1
		strb tmp2, [odr]                            @ 2

		ror tmp2, #8                                @ 1
		add tmp1, tmp1, attr_offset, LSL #4         @ 1
		strb tmp2, [odr]                            @ 2

		ror tmp2, #8                                @ 1
		ldr tmp1, [attr_base, tmp1, LSL #2]         @ 1*
		strb tmp2, [odr]                            @ 2

		ror tmp2, #8                                @ 1
		ldrb attr_offset, [attr], #1                @ 1*
		strb tmp2, [odr]                            @ 2
	.endr

	@ 0 at the end
	mov r4, #0                     @ 1
	nop                            @ 1
	strb r4, [odr]                 @ 2

	pop	{r4, r5, r6, r7}
	bx lr
