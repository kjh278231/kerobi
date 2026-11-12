0000000000131710 <PRIVATE000000000009e376>:
  131710:	push   %rbx
  131711:	push   %rbp
  131712:	push   %rsi
  131713:	mov    %rdi,%rbp
  131716:	mov    %rcx,%rbx
  131719:	mov    0x1d0(%rbp),%rcx
  131720:	movq   $0x0,0x200(%rbp)
  13172b:	call   12de60 <PRIVATE0000000000221173>
  131730:	test   %rbx,%rbx
  131733:	je     13174d <PRIVATE000000000009e376+0x3d>
  131735:	movsd  0x200(%rbp),%xmm0
  13173d:	mulsd  0x1f8(%rbp),%xmm0
  131745:	addsd  (%rbx),%xmm0
  131749:	movsd  %xmm0,(%rbx)
  13174d:	pop    %rcx
  13174e:	pop    %rbp
  13174f:	pop    %rbx
  131750:	ret    
000000000012de60 <PRIVATE0000000000221173>:
  12de60:	push   %r12
  12de62:	push   %r13
  12de64:	push   %r14
  12de66:	push   %r15
  12de68:	push   %rbx
  12de69:	push   %rbp
  12de6a:	sub    $0x118,%rsp
  12de71:	mov    %rdi,%rbx
  12de74:	mov    %rdx,%r12
  12de77:	mov    %rcx,%r15
  12de7a:	mov    %rsi,%r13
  12de7d:	mov    0x198(%rbx),%r9
  12de84:	lea    0x20(%rbx),%rdx
  12de88:	movsd  0xe8(%rdx),%xmm1
  12de90:	mov    %r9,0x50(%rsp)
  12de95:	mov    0x10(%rdx),%r9
  12de99:	mov    %r9,0xf8(%rsp)
  12dea1:	mov    0x1b0(%rbx),%r10
  12dea8:	mov    0x1a8(%rbx),%r11
  12deaf:	movsd  0x10(%rbx),%xmm0
  12deb4:	mov    0xe4(%rdx),%r9d
  12debb:	mov    0x8(%rbx),%ebp
  12debe:	mov    0x180(%rbx),%eax
  12dec4:	mov    0x1b8(%rbx),%r14
  12decb:	mov    %r10,0x108(%rsp)
  12ded3:	mov    %r11,0x8(%rsp)
  12ded8:	movsd  %xmm0,0x100(%rsp)
  12dee1:	mov    %r9d,0x58(%rsp)
  12dee6:	addsd  0xf0(%rdx),%xmm1
  12deee:	cvttsd2si %xmm1,%esi
  12def2:	test   %r15,%r15
  12def5:	je     12df24 <PRIVATE0000000000221173+0xc4>
  12def7:	mov    0x8(%r13),%rdi
  12defb:	mov    0xb0(%rbx),%r9
  12df02:	mov    0xb8(%rbx),%r11
  12df09:	movslq (%rdi),%rcx
  12df0c:	movl   $0x0,(%r15)
  12df13:	mov    (%r9,%rcx,8),%r10
  12df17:	cmp    (%r11,%rcx,8),%r10
  12df1b:	jne    12e55d <PRIVATE0000000000221173+0x6fd>
  12df21:	xor    %r15d,%r15d
  12df24:	mov    0x0(%r13),%r9d
  12df28:	test   %r9d,%r9d
  12df2b:	jl     12df3e <PRIVATE0000000000221173+0xde>
  12df2d:	imul   %r9d,%esi
  12df31:	cmp    %ebp,%esi
  12df33:	jle    12e567 <PRIVATE0000000000221173+0x707>
  12df39:	jmp    130b68 <PRIVATE0000000000221173+0x2d08>
  12df3e:	mov    0xb0(%rbx),%rdx
  12df45:	mov    %rdx,0x38(%rsp)
  12df4a:	mov    0xc8(%rbx),%rdx
  12df51:	mov    0xb8(%rbx),%r10
  12df58:	mov    %rdx,0x18(%rsp)
  12df5d:	mov    0x48(%rbx),%r11
  12df61:	mov    0x50(%rbx),%rdx
  12df65:	mov    0xd0(%rbx),%r15
  12df6c:	mov    %r10,0x30(%rsp)
  12df71:	mov    %r11,0x28(%rsp)
  12df76:	mov    %r15,0x20(%rsp)
  12df7b:	mov    %rdx,0x10(%rsp)
  12df80:	mov    0x10(%r12),%r10
  12df85:	mov    0x10(%r13),%rsi
  12df89:	jle    13036d <PRIVATE0000000000221173+0x250d>
  12df8f:	pxor   %xmm0,%xmm0
  12df93:	pxor   %xmm4,%xmm4
  12df97:	cvtsi2sd %r9d,%xmm0
  12df9c:	cvtsi2sd %ebp,%xmm4
  12dfa0:	mulsd  0x2930690(%rip),%xmm0        # 2a5e638 <_fini+0x1a34>
  12dfa8:	mov    0x8(%r13),%rcx
  12dfac:	addsd  %xmm4,%xmm0
  12dfb0:	addsd  0x200(%rbx),%xmm0
  12dfb8:	movsd  %xmm0,0x200(%rbx)
  12dfc0:	test   %ebp,%ebp
  12dfc2:	jle    12e073 <PRIVATE0000000000221173+0x213>
  12dfc8:	movslq %ebp,%rdi
  12dfcb:	cmp    $0x8,%rdi
  12dfcf:	jl     130365 <PRIVATE0000000000221173+0x2505>
  12dfd5:	mov    %r10,%r15
  12dfd8:	and    $0xf,%r15
  12dfdc:	test   %r15d,%r15d
  12dfdf:	je     12dff4 <PRIVATE0000000000221173+0x194>
  12dfe1:	test   $0x7,%r15d
  12dfe8:	jne    130365 <PRIVATE0000000000221173+0x2505>
  12dfee:	mov    $0x1,%r15d
  12dff4:	mov    %r15d,%r15d
  12dff7:	mov    %r15,(%rsp)
  12dffb:	lea    0x8(%r15),%rdx
  12dfff:	cmp    %rdx,%rdi
  12e002:	jl     130365 <PRIVATE0000000000221173+0x2505>
  12e008:	mov    %ebp,%r11d
  12e00b:	xor    %edx,%edx
  12e00d:	sub    %r15d,%r11d
  12e010:	and    $0x7,%r11d
  12e014:	neg    %r11d
  12e017:	add    %ebp,%r11d
  12e01a:	movslq %r11d,%r11
  12e01d:	test   %r15d,%r15d
  12e020:	jbe    12e038 <PRIVATE0000000000221173+0x1d8>
  12e022:	mov    %r15,%r8
  12e025:	xor    %r15d,%r15d
  12e028:	mov    %r15,(%r10,%rdx,8)
  12e02c:	inc    %rdx
  12e02f:	cmp    %r8,%rdx
  12e032:	jb     12e028 <PRIVATE0000000000221173+0x1c8>
  12e034:	mov    %r8,(%rsp)
  12e038:	mov    (%rsp),%rdx
  12e03c:	pxor   %xmm0,%xmm0
  12e040:	movups %xmm0,(%r10,%rdx,8)
  12e045:	movups %xmm0,0x10(%r10,%rdx,8)
  12e04b:	movups %xmm0,0x20(%r10,%rdx,8)
  12e051:	movups %xmm0,0x30(%r10,%rdx,8)
  12e057:	add    $0x8,%rdx
  12e05b:	cmp    %r11,%rdx
  12e05e:	jb     12e040 <PRIVATE0000000000221173+0x1e0>
  12e060:	cmp    %rdi,%r11
  12e063:	jae    12e073 <PRIVATE0000000000221173+0x213>
  12e065:	xor    %edx,%edx
  12e067:	mov    %rdx,(%r10,%r11,8)
  12e06b:	inc    %r11
  12e06e:	cmp    %rdi,%r11
  12e071:	jb     12e067 <PRIVATE0000000000221173+0x207>
  12e073:	movslq %r9d,%r9
  12e076:	mov    %r9,0x8(%rsp)
  12e07b:	cmp    $0x2,%r9
  12e07f:	jl     13035d <PRIVATE0000000000221173+0x24fd>
  12e085:	mov    %rsi,%rdx
  12e088:	and    $0xf,%rdx
  12e08c:	test   %edx,%edx
  12e08e:	je     12e09e <PRIVATE0000000000221173+0x23e>
  12e090:	test   $0x7,%dl
  12e093:	jne    13035d <PRIVATE0000000000221173+0x24fd>
  12e099:	mov    $0x1,%edx
  12e09e:	mov    %edx,%edx
  12e0a0:	mov    %rdx,(%rsp)
  12e0a4:	lea    0x2(%rdx),%r11
  12e0a8:	cmp    0x8(%rsp),%r11
  12e0ad:	jg     13035d <PRIVATE0000000000221173+0x24fd>
  12e0b3:	mov    %r9d,%r11d
  12e0b6:	sub    %edx,%r11d
  12e0b9:	and    $0x1,%r11d
  12e0bd:	sub    %r11d,%r9d
  12e0c0:	xor    %r11d,%r11d
  12e0c3:	movslq %r9d,%r9
  12e0c6:	test   %edx,%edx
  12e0c8:	jbe    12e0e5 <PRIVATE0000000000221173+0x285>
  12e0ca:	mov    %rdx,%rdi
  12e0cd:	movslq (%rcx,%r11,4),%r15
  12e0d1:	mov    (%rsi,%r11,8),%rdx
  12e0d5:	inc    %r11
  12e0d8:	mov    %rdx,(%r10,%r15,8)
  12e0dc:	cmp    %rdi,%r11
  12e0df:	jb     12e0cd <PRIVATE0000000000221173+0x26d>
  12e0e1:	mov    %rdi,(%rsp)
  12e0e5:	mov    (%rsp),%r15
  12e0e9:	movslq (%rcx,%r15,4),%rdx
  12e0ed:	movslq 0x4(%rcx,%r15,4),%r11
  12e0f2:	movups (%rsi,%r15,8),%xmm0
  12e0f7:	add    $0x2,%r15
  12e0fb:	movsd  %xmm0,(%r10,%rdx,8)
  12e101:	psrldq $0x8,%xmm0
  12e106:	movsd  %xmm0,(%r10,%r11,8)
  12e10c:	cmp    %r9,%r15
  12e10f:	jb     12e0e9 <PRIVATE0000000000221173+0x289>
  12e111:	cmp    0x8(%rsp),%r9
  12e116:	jae    12e131 <PRIVATE0000000000221173+0x2d1>
  12e118:	mov    0x8(%rsp),%r15
  12e11d:	movslq (%rcx,%r9,4),%r11
  12e121:	mov    (%rsi,%r9,8),%rdx
  12e125:	inc    %r9
  12e128:	mov    %rdx,(%r10,%r11,8)
  12e12c:	cmp    %r15,%r9
  12e12f:	jb     12e11d <PRIVATE0000000000221173+0x2bd>
  12e131:	movaps %xmm4,%xmm0
  12e134:	xor    %r9d,%r9d
  12e137:	addsd  %xmm4,%xmm0
  12e13b:	addsd  0x200(%rbx),%xmm0
  12e143:	movsd  %xmm0,0x200(%rbx)
  12e14b:	test   %ebp,%ebp
  12e14d:	jle    12e37e <PRIVATE0000000000221173+0x51e>
  12e153:	mov    %rcx,0x8(%rsp)
  12e158:	mov    %eax,0x48(%rsp)
  12e15c:	mov    %ebp,0xe8(%rsp)
  12e163:	mov    %rbx,0x110(%rsp)
  12e16b:	movsd  0x29304c5(%rip),%xmm1        # 2a5e638 <_fini+0x1a34>
  12e173:	movslq %ebp,%rdx
  12e176:	mov    %rsi,(%rsp)
  12e17a:	mov    %r14,0x40(%rsp)
  12e17f:	mov    %r13,0xd8(%rsp)
  12e187:	mov    %r12,0xe0(%rsp)
  12e18f:	movsd  0x100(%rsp),%xmm0
  12e198:	mov    0x10(%rsp),%rax
  12e19d:	mov    0x18(%rsp),%rcx
  12e1a2:	mov    0x20(%rsp),%rbx
  12e1a7:	mov    0x28(%rsp),%rbp
  12e1ac:	movslq 0x0(%rbp,%r9,4),%r8
  12e1b1:	movsd  (%r10,%r8,8),%xmm3
  12e1b7:	movaps %xmm3,%xmm2
  12e1ba:	andps  0x293042f(%rip),%xmm2        # 2a5e5f0 <_fini+0x19ec>
  12e1c1:	comisd %xmm0,%xmm2
  12e1c5:	jbe    12e339 <PRIVATE0000000000221173+0x4d9>
  12e1cb:	divsd  (%rax,%r8,8),%xmm3
  12e1d1:	andps  0x2930418(%rip),%xmm3        # 2a5e5f0 <_fini+0x19ec>
  12e1d8:	comisd %xmm0,%xmm3
  12e1dc:	jbe    12e339 <PRIVATE0000000000221173+0x4d9>
  12e1e2:	mov    0x30(%rsp),%r13
  12e1e7:	pxor   %xmm2,%xmm2
  12e1eb:	mov    0x38(%rsp),%rdi
  12e1f0:	mov    0x110(%rsp),%rsi
  12e1f8:	mov    0x0(%r13,%r8,8),%r11
  12e1fd:	mov    %r11,%r13
  12e200:	mov    (%rdi,%r8,8),%rdi
  12e204:	sub    %rdi,%r13
  12e207:	cvtsi2sd %r13,%xmm2
  12e20c:	mulsd  %xmm1,%xmm2
  12e210:	addsd  0x200(%rsi),%xmm2
  12e218:	movsd  %xmm2,0x200(%rsi)
  12e220:	movsd  (%r10,%r8,8),%xmm2
  12e226:	divsd  (%rax,%r8,8),%xmm2
  12e22c:	movsd  %xmm2,(%r10,%r8,8)
  12e232:	cmp    %r11,%rdi
  12e235:	jge    12e341 <PRIVATE0000000000221173+0x4e1>
  12e23b:	xor    %r8d,%r8d
  12e23e:	mov    %r13,%r12
  12e241:	xor    %r11d,%r11d
  12e244:	mov    $0x1,%r14d
  12e24a:	xor    %esi,%esi
  12e24c:	shr    $0x2,%r12
  12e250:	je     12e301 <PRIVATE0000000000221173+0x4a1>
  12e256:	lea    (%rbx,%rdi,4),%r15
  12e25a:	lea    (%rcx,%rdi,8),%r14
  12e25e:	movsd  (%rsi,%r14,1),%xmm3
  12e264:	inc    %r8
  12e267:	mulsd  %xmm2,%xmm3
  12e26b:	movslq (%r11,%r15,1),%rbp
  12e26f:	movsd  (%r10,%rbp,8),%xmm5
  12e275:	subsd  %xmm3,%xmm5
  12e279:	movsd  %xmm5,(%r10,%rbp,8)
  12e27f:	movsd  0x8(%rsi,%r14,1),%xmm6
  12e286:	mulsd  %xmm2,%xmm6
  12e28a:	movslq 0x4(%r11,%r15,1),%rbp
  12e28f:	movsd  (%r10,%rbp,8),%xmm7
  12e295:	subsd  %xmm6,%xmm7
  12e299:	movsd  %xmm7,(%r10,%rbp,8)
  12e29f:	movsd  0x10(%rsi,%r14,1),%xmm8
  12e2a6:	mulsd  %xmm2,%xmm8
  12e2ab:	movslq 0x8(%r11,%r15,1),%rbp
  12e2b0:	movsd  (%r10,%rbp,8),%xmm9
  12e2b6:	subsd  %xmm8,%xmm9
  12e2bb:	movsd  %xmm9,(%r10,%rbp,8)
  12e2c1:	movsd  0x18(%rsi,%r14,1),%xmm10
  12e2c8:	add    $0x20,%rsi
  12e2cc:	mulsd  %xmm2,%xmm10
  12e2d1:	movslq 0xc(%r11,%r15,1),%rbp
  12e2d6:	add    $0x10,%r11
  12e2da:	movsd  (%r10,%rbp,8),%xmm11
  12e2e0:	subsd  %xmm10,%xmm11
  12e2e5:	movsd  %xmm11,(%r10,%rbp,8)
  12e2eb:	cmp    %r12,%r8
  12e2ee:	jb     12e25e <PRIVATE0000000000221173+0x3fe>
  12e2f4:	mov    0x28(%rsp),%rbp
  12e2f9:	lea    0x1(,%r8,4),%r14
  12e301:	dec    %r14
  12e304:	cmp    %r13,%r14
  12e307:	jae    12e341 <PRIVATE0000000000221173+0x4e1>
  12e309:	lea    (%rbx,%rdi,4),%r8
  12e30d:	lea    (%rcx,%rdi,8),%rsi
  12e311:	movsd  (%rsi,%r14,8),%xmm3
  12e317:	mulsd  %xmm2,%xmm3
  12e31b:	movslq (%r8,%r14,4),%rdi
  12e31f:	inc    %r14
  12e322:	movsd  (%r10,%rdi,8),%xmm5
  12e328:	subsd  %xmm3,%xmm5
  12e32c:	movsd  %xmm5,(%r10,%rdi,8)
  12e332:	cmp    %r13,%r14
  12e335:	jb     12e311 <PRIVATE0000000000221173+0x4b1>
  12e337:	jmp    12e341 <PRIVATE0000000000221173+0x4e1>
  12e339:	movq   $0x0,(%r10,%r8,8)
  12e341:	inc    %r9
  12e344:	cmp    %rdx,%r9
  12e347:	jb     12e1ac <PRIVATE0000000000221173+0x34c>
  12e34d:	mov    (%rsp),%rsi
  12e351:	mov    0x8(%rsp),%rcx
  12e356:	mov    0x40(%rsp),%r14
  12e35b:	mov    0x48(%rsp),%eax
  12e35f:	mov    0xe8(%rsp),%ebp
  12e366:	mov    0x110(%rsp),%rbx
  12e36e:	mov    0xd8(%rsp),%r13
  12e376:	mov    0xe0(%rsp),%r12
  12e37e:	mov    $0xffffffff,%r11d
  12e384:	movl   $0xffffffff,(%r12)
  12e38c:	test   %eax,%eax
  12e38e:	je     12ec37 <PRIVATE0000000000221173+0xdd7>
  12e394:	mov    %eax,%edx
  12e396:	mov    %r11d,0x0(%r13)
  12e39a:	xor    %r11d,%r11d
  12e39d:	mov    0x8(%r12),%r15
  12e3a2:	xor    %r9d,%r9d
  12e3a5:	mov    %rcx,0x8(%r12)
  12e3aa:	dec    %edx
  12e3ac:	mov    %rsi,0x10(%r12)
  12e3b1:	mov    %r15,0x8(%r13)
  12e3b5:	mov    %r10,0x10(%r13)
  12e3b9:	movl   $0xffffffff,(%r12)
  12e3c1:	js     12e52a <PRIVATE0000000000221173+0x6ca>
  12e3c7:	movslq %eax,%rax
  12e3ca:	mov    0x50(%rsp),%rdx
  12e3cf:	movsd  0x100(%rsp),%xmm1
  12e3d8:	mov    %ebp,0xe8(%rsp)
  12e3df:	mov    %rbx,0x110(%rsp)
  12e3e7:	lea    (%rax,%rax,2),%r14
  12e3eb:	mov    %r13,0xd8(%rsp)
  12e3f3:	lea    (%rdx,%r14,8),%rdx
  12e3f7:	mov    %r12,0xe0(%rsp)
  12e3ff:	xor    %r12d,%r12d
  12e402:	movslq -0x18(%r9,%rdx,1),%rdi
  12e407:	mov    -0x10(%r9,%rdx,1),%rbp
  12e40c:	mov    -0x8(%r9,%rdx,1),%rsi
  12e411:	movsd  (%r10,%rdi,8),%xmm0
  12e417:	movaps %xmm0,%xmm2
  12e41a:	andps  0x29301cf(%rip),%xmm2        # 2a5e5f0 <_fini+0x19ec>
  12e421:	comisd %xmm1,%xmm2
  12e425:	jbe    12e4f7 <PRIVATE0000000000221173+0x697>
  12e42b:	mov    -0x14(%r9,%rdx,1),%r8d
  12e430:	test   %r8d,%r8d
  12e433:	jle    12e4fb <PRIVATE0000000000221173+0x69b>
  12e439:	mov    %r8d,%ecx
  12e43c:	mov    %r12,%r14
  12e43f:	mov    $0x1,%ebx
  12e444:	mov    %r14,%r13
  12e447:	shr    %ecx
  12e449:	je     12e4b8 <PRIVATE0000000000221173+0x658>
  12e44b:	movslq 0x0(%rbp,%r14,8),%rbx
  12e450:	cmp    %rdi,%rbx
  12e453:	je     12e468 <PRIVATE0000000000221173+0x608>
  12e455:	movsd  0x0(%r13,%rsi,1),%xmm2
  12e45c:	mulsd  %xmm0,%xmm2
  12e460:	addsd  (%r10,%rbx,8),%xmm2
  12e466:	jmp    12e473 <PRIVATE0000000000221173+0x613>
  12e468:	movsd  0x0(%r13,%rsi,1),%xmm2
  12e46f:	mulsd  %xmm0,%xmm2
  12e473:	movsd  %xmm2,(%r10,%rbx,8)
  12e479:	movslq 0x4(%rbp,%r14,8),%rbx
  12e47e:	cmp    %rdi,%rbx
  12e481:	je     12e496 <PRIVATE0000000000221173+0x636>
  12e483:	movsd  0x8(%rsi,%r13,1),%xmm2
  12e48a:	mulsd  %xmm0,%xmm2
  12e48e:	addsd  (%r10,%rbx,8),%xmm2
  12e494:	jmp    12e4a1 <PRIVATE0000000000221173+0x641>
  12e496:	movsd  0x8(%rsi,%r13,1),%xmm2
  12e49d:	mulsd  %xmm0,%xmm2
  12e4a1:	inc    %r14
  12e4a4:	add    $0x10,%r13
  12e4a8:	movsd  %xmm2,(%r10,%rbx,8)
  12e4ae:	cmp    %rcx,%r14
  12e4b1:	jb     12e44b <PRIVATE0000000000221173+0x5eb>
  12e4b3:	lea    0x1(%r14,%r14,1),%ebx
  12e4b8:	lea    -0x1(%rbx),%ecx
  12e4bb:	cmp    %r8d,%ecx
  12e4be:	jae    12e4fb <PRIVATE0000000000221173+0x69b>
  12e4c0:	movslq %ebx,%rbx
  12e4c3:	movslq -0x4(%rbp,%rbx,4),%rcx
  12e4c8:	cmp    %rdi,%rcx
  12e4cb:	je     12e4e5 <PRIVATE0000000000221173+0x685>
  12e4cd:	movsd  -0x8(%rsi,%rbx,8),%xmm2
  12e4d3:	mulsd  %xmm2,%xmm0
  12e4d7:	addsd  (%r10,%rcx,8),%xmm0
  12e4dd:	movsd  %xmm0,(%r10,%rcx,8)
  12e4e3:	jmp    12e4fb <PRIVATE0000000000221173+0x69b>
  12e4e5:	movsd  -0x8(%rsi,%rbx,8),%xmm2
  12e4eb:	mulsd  %xmm2,%xmm0
  12e4ef:	movsd  %xmm0,(%r10,%rcx,8)
  12e4f5:	jmp    12e4fb <PRIVATE0000000000221173+0x69b>
  12e4f7:	mov    %r12,(%r10,%rdi,8)
  12e4fb:	inc    %r11
  12e4fe:	add    $0xffffffffffffffe8,%r9
  12e502:	cmp    %rax,%r11
  12e505:	jb     12e402 <PRIVATE0000000000221173+0x5a2>
  12e50b:	mov    0xe8(%rsp),%ebp
  12e512:	mov    0x110(%rsp),%rbx
  12e51a:	mov    0xd8(%rsp),%r13
  12e522:	mov    0xe0(%rsp),%r12
  12e52a:	pxor   %xmm1,%xmm1
  12e52e:	cvtsi2sdl 0x184(%rbx),%xmm1
  12e536:	mulsd  0x29300fa(%rip),%xmm1        # 2a5e638 <_fini+0x1a34>
  12e53e:	movl   $0xffffffff,0x0(%r13)
  12e546:	mov    0x8(%r12),%r9
  12e54b:	mov    0x10(%r12),%rdx
  12e550:	addsd  0x200(%rbx),%xmm1
  12e558:	jmp    12e912 <PRIVATE0000000000221173+0xab2>
  12e55d:	mov    (%rdi),%r9d
  12e560:	mov    %r9d,0x1d8(%rbx)
  12e567:	add    $0xfffffffffffffff0,%rsp
  12e56b:	mov    %rbx,%rdi
  12e56e:	mov    %ebp,%esi
  12e570:	mov    %r13,%rcx
  12e573:	mov    %r12,%r8
  12e576:	mov    %r14,(%rsp)
  12e57a:	mov    0x118(%rsp),%r9
  12e582:	mov    %eax,0x58(%rsp)
  12e586:	call   130b70 <PRIVATE000000000021a941>
  12e58b:	mov    0x58(%rsp),%eax
  12e58f:	add    $0x10,%rsp
  12e593:	mov    (%r12),%r11d
  12e597:	test   %r11d,%r11d
  12e59a:	je     13034b <PRIVATE0000000000221173+0x24eb>
  12e5a0:	test   %r15,%r15
  12e5a3:	je     12e5e0 <PRIVATE0000000000221173+0x780>
  12e5a5:	xor    %edx,%edx
  12e5a7:	test   %r11d,%r11d
  12e5aa:	jle    12e5d9 <PRIVATE0000000000221173+0x779>
  12e5ac:	mov    0x8(%r12),%rcx
  12e5b1:	mov    0x10(%r12),%r11
  12e5b6:	mov    0x8(%r15),%r10
  12e5ba:	mov    0x10(%r15),%r9
  12e5be:	mov    (%rcx,%rdx,4),%esi
  12e5c1:	mov    (%r11,%rdx,8),%rdi
  12e5c5:	mov    %esi,(%r10,%rdx,4)
  12e5c9:	mov    %rdi,(%r9,%rdx,8)
  12e5cd:	inc    %rdx
  12e5d0:	movslq (%r12),%r8
  12e5d4:	cmp    %r8,%rdx
  12e5d7:	jl     12e5be <PRIVATE0000000000221173+0x75e>
  12e5d9:	mov    %edx,(%r15)
  12e5dc:	mov    (%r12),%r11d
  12e5e0:	test   %eax,%eax
  12e5e2:	je     12ec22 <PRIVATE0000000000221173+0xdc2>
  12e5e8:	test   %r11d,%r11d
  12e5eb:	jl     1303b8 <PRIVATE0000000000221173+0x2558>
  12e5f1:	mov    0x58(%rsp),%edx
  12e5f5:	imul   %r11d,%edx
  12e5f9:	cmp    %ebp,%edx
  12e5fb:	jle    12e91d <PRIVATE0000000000221173+0xabd>
  12e601:	mov    0x10(%r13),%r10
  12e605:	mov    0x8(%r12),%r9
  12e60a:	mov    0x10(%r12),%rdx
  12e60f:	test   %r11d,%r11d
  12e612:	jle    12e768 <PRIVATE0000000000221173+0x908>
  12e618:	test   %ebp,%ebp
  12e61a:	jle    12e6b9 <PRIVATE0000000000221173+0x859>
  12e620:	movslq %ebp,%rsi
  12e623:	cmp    $0x8,%rsi
  12e627:	jl     1303b1 <PRIVATE0000000000221173+0x2551>
  12e62d:	mov    %r10,%r14
  12e630:	and    $0xf,%r14
  12e634:	test   %r14d,%r14d
  12e637:	je     12e64c <PRIVATE0000000000221173+0x7ec>
  12e639:	test   $0x7,%r14d
  12e640:	jne    1303b1 <PRIVATE0000000000221173+0x2551>
  12e646:	mov    $0x1,%r14d
  12e64c:	mov    %r14d,%r15d
  12e64f:	lea    0x8(%r15),%rcx
  12e653:	cmp    %rcx,%rsi
  12e656:	jl     1303b1 <PRIVATE0000000000221173+0x2551>
  12e65c:	mov    %ebp,%ecx
  12e65e:	xor    %edi,%edi
  12e660:	sub    %r14d,%ecx
  12e663:	and    $0x7,%ecx
  12e666:	neg    %ecx
  12e668:	add    %ebp,%ecx
  12e66a:	movslq %ecx,%rcx
  12e66d:	test   %r14d,%r14d
  12e670:	jbe    12e681 <PRIVATE0000000000221173+0x821>
  12e672:	xor    %r14d,%r14d
  12e675:	mov    %r14,(%r10,%rdi,8)
  12e679:	inc    %rdi
  12e67c:	cmp    %r15,%rdi
  12e67f:	jb     12e675 <PRIVATE0000000000221173+0x815>
  12e681:	pxor   %xmm0,%xmm0
  12e685:	movups %xmm0,(%r10,%r15,8)
  12e68a:	movups %xmm0,0x10(%r10,%r15,8)
  12e690:	movups %xmm0,0x20(%r10,%r15,8)
  12e696:	movups %xmm0,0x30(%r10,%r15,8)
  12e69c:	add    $0x8,%r15
  12e6a0:	cmp    %rcx,%r15
  12e6a3:	jb     12e685 <PRIVATE0000000000221173+0x825>
  12e6a5:	cmp    %rsi,%rcx
  12e6a8:	jae    12e6b9 <PRIVATE0000000000221173+0x859>
  12e6aa:	xor    %r14d,%r14d
  12e6ad:	mov    %r14,(%r10,%rcx,8)
  12e6b1:	inc    %rcx
  12e6b4:	cmp    %rsi,%rcx
  12e6b7:	jb     12e6ad <PRIVATE0000000000221173+0x84d>
  12e6b9:	test   %r11d,%r11d
  12e6bc:	jle    12e768 <PRIVATE0000000000221173+0x908>
  12e6c2:	movslq %r11d,%r15
  12e6c5:	cmp    $0x2,%r15
  12e6c9:	jl     1303a9 <PRIVATE0000000000221173+0x2549>
  12e6cf:	mov    %rdx,%r14
  12e6d2:	and    $0xf,%r14
  12e6d6:	test   %r14d,%r14d
  12e6d9:	je     12e6ee <PRIVATE0000000000221173+0x88e>
  12e6db:	test   $0x7,%r14d
  12e6e2:	jne    1303a9 <PRIVATE0000000000221173+0x2549>
  12e6e8:	mov    $0x1,%r14d
  12e6ee:	mov    %r14d,%ecx
  12e6f1:	lea    0x2(%rcx),%rsi
  12e6f5:	cmp    %rsi,%r15
  12e6f8:	jl     1303a9 <PRIVATE0000000000221173+0x2549>
  12e6fe:	mov    %r11d,%esi
  12e701:	sub    %r14d,%esi
  12e704:	and    $0x1,%esi
  12e707:	sub    %esi,%r11d
  12e70a:	xor    %esi,%esi
  12e70c:	movslq %r11d,%r11
  12e70f:	test   %r14d,%r14d
  12e712:	jbe    12e728 <PRIVATE0000000000221173+0x8c8>
  12e714:	movslq (%r9,%rsi,4),%rdi
  12e718:	mov    (%rdx,%rsi,8),%r14
  12e71c:	inc    %rsi
  12e71f:	mov    %r14,(%r10,%rdi,8)
  12e723:	cmp    %rcx,%rsi
  12e726:	jb     12e714 <PRIVATE0000000000221173+0x8b4>
  12e728:	movslq (%r9,%rcx,4),%r14
  12e72c:	movups (%rdx,%rcx,8),%xmm0
  12e730:	movsd  %xmm0,(%r10,%r14,8)
  12e736:	movslq 0x4(%r9,%rcx,4),%r14
  12e73b:	add    $0x2,%rcx
  12e73f:	psrldq $0x8,%xmm0
  12e744:	movsd  %xmm0,(%r10,%r14,8)
  12e74a:	cmp    %r11,%rcx
  12e74d:	jb     12e728 <PRIVATE0000000000221173+0x8c8>
  12e74f:	cmp    %r15,%r11
  12e752:	jae    12e768 <PRIVATE0000000000221173+0x908>
  12e754:	movslq (%r9,%r11,4),%rcx
  12e758:	mov    (%rdx,%r11,8),%r14
  12e75c:	inc    %r11
  12e75f:	mov    %r14,(%r10,%rcx,8)
  12e763:	cmp    %r15,%r11
  12e766:	jb     12e754 <PRIVATE0000000000221173+0x8f4>
  12e768:	mov    %eax,%r11d
  12e76b:	xor    %r14d,%r14d
  12e76e:	xor    %r15d,%r15d
  12e771:	dec    %r11d
  12e774:	js     12e8e2 <PRIVATE0000000000221173+0xa82>
  12e77a:	movslq %eax,%rax
  12e77d:	xor    %r8d,%r8d
  12e780:	mov    0x50(%rsp),%r11
  12e785:	movsd  0x100(%rsp),%xmm1
  12e78e:	mov    %rdx,0x8(%rsp)
  12e793:	mov    %ebp,0xe8(%rsp)
  12e79a:	lea    (%rax,%rax,2),%rcx
  12e79e:	mov    %rbx,0x110(%rsp)
  12e7a6:	lea    (%r11,%rcx,8),%r11
  12e7aa:	mov    %r13,0xd8(%rsp)
  12e7b2:	mov    %r12,0xe0(%rsp)
  12e7ba:	movslq -0x18(%r15,%r11,1),%rbx
  12e7bf:	mov    -0x10(%r15,%r11,1),%rbp
  12e7c4:	mov    -0x8(%r15,%r11,1),%rsi
  12e7c9:	movsd  (%r10,%rbx,8),%xmm0
  12e7cf:	movaps %xmm0,%xmm2
  12e7d2:	andps  0x292fe17(%rip),%xmm2        # 2a5e5f0 <_fini+0x19ec>
  12e7d9:	comisd %xmm1,%xmm2
  12e7dd:	jbe    12e8aa <PRIVATE0000000000221173+0xa4a>
  12e7e3:	mov    -0x14(%r15,%r11,1),%edi
  12e7e8:	test   %edi,%edi
  12e7ea:	jle    12e8ae <PRIVATE0000000000221173+0xa4e>
  12e7f0:	mov    %edi,%ecx
  12e7f2:	mov    %r8,%r13
  12e7f5:	mov    $0x1,%edx
  12e7fa:	mov    %r13,%r12
  12e7fd:	shr    %ecx
  12e7ff:	je     12e86c <PRIVATE0000000000221173+0xa0c>
  12e801:	movslq 0x0(%rbp,%r13,8),%rdx
  12e806:	cmp    %rbx,%rdx
  12e809:	je     12e81d <PRIVATE0000000000221173+0x9bd>
  12e80b:	movsd  (%r12,%rsi,1),%xmm2
  12e811:	mulsd  %xmm0,%xmm2
  12e815:	addsd  (%r10,%rdx,8),%xmm2
  12e81b:	jmp    12e827 <PRIVATE0000000000221173+0x9c7>
  12e81d:	movsd  (%r12,%rsi,1),%xmm2
  12e823:	mulsd  %xmm0,%xmm2
  12e827:	movsd  %xmm2,(%r10,%rdx,8)
  12e82d:	movslq 0x4(%rbp,%r13,8),%rdx
  12e832:	cmp    %rbx,%rdx
  12e835:	je     12e84a <PRIVATE0000000000221173+0x9ea>
  12e837:	movsd  0x8(%rsi,%r12,1),%xmm2
  12e83e:	mulsd  %xmm0,%xmm2
  12e842:	addsd  (%r10,%rdx,8),%xmm2
  12e848:	jmp    12e855 <PRIVATE0000000000221173+0x9f5>
  12e84a:	movsd  0x8(%rsi,%r12,1),%xmm2
  12e851:	mulsd  %xmm0,%xmm2
  12e855:	inc    %r13
  12e858:	add    $0x10,%r12
  12e85c:	movsd  %xmm2,(%r10,%rdx,8)
  12e862:	cmp    %rcx,%r13
  12e865:	jb     12e801 <PRIVATE0000000000221173+0x9a1>
  12e867:	lea    0x1(%r13,%r13,1),%edx
  12e86c:	lea    -0x1(%rdx),%ecx
  12e86f:	cmp    %edi,%ecx
  12e871:	jae    12e8ae <PRIVATE0000000000221173+0xa4e>
  12e873:	movslq %edx,%rdx
  12e876:	movslq -0x4(%rbp,%rdx,4),%rcx
  12e87b:	cmp    %rbx,%rcx
  12e87e:	je     12e898 <PRIVATE0000000000221173+0xa38>
  12e880:	movsd  -0x8(%rsi,%rdx,8),%xmm2
  12e886:	mulsd  %xmm2,%xmm0
  12e88a:	addsd  (%r10,%rcx,8),%xmm0
  12e890:	movsd  %xmm0,(%r10,%rcx,8)
  12e896:	jmp    12e8ae <PRIVATE0000000000221173+0xa4e>
  12e898:	movsd  -0x8(%rsi,%rdx,8),%xmm2
  12e89e:	mulsd  %xmm2,%xmm0
  12e8a2:	movsd  %xmm0,(%r10,%rcx,8)
  12e8a8:	jmp    12e8ae <PRIVATE0000000000221173+0xa4e>
  12e8aa:	mov    %r8,(%r10,%rbx,8)
  12e8ae:	inc    %r14
  12e8b1:	add    $0xffffffffffffffe8,%r15
  12e8b5:	cmp    %rax,%r14
  12e8b8:	jb     12e7ba <PRIVATE0000000000221173+0x95a>
  12e8be:	mov    0x8(%rsp),%rdx
  12e8c3:	mov    0xe8(%rsp),%ebp
  12e8ca:	mov    0x110(%rsp),%rbx
  12e8d2:	mov    0xd8(%rsp),%r13
  12e8da:	mov    0xe0(%rsp),%r12
  12e8e2:	pxor   %xmm1,%xmm1
  12e8e6:	pxor   %xmm4,%xmm4
  12e8ea:	cvtsi2sdl 0x184(%rbx),%xmm1
  12e8f2:	cvtsi2sd %ebp,%xmm4
  12e8f6:	mulsd  0x292fd3a(%rip),%xmm1        # 2a5e638 <_fini+0x1a34>
  12e8fe:	movl   $0xffffffff,0x0(%r13)
  12e906:	mov    0x8(%r13),%r15
  12e90a:	addsd  0x200(%rbx),%xmm1
  12e912:	mov    $0xffffffff,%r11d
  12e918:	jmp    12fef8 <PRIVATE0000000000221173+0x2098>
  12e91d:	test   %r11d,%r11d
  12e920:	je     12ec00 <PRIVATE0000000000221173+0xda0>
  12e926:	mov    0x10(%r12),%rdx
  12e92b:	mov    $0x0,%esi
  12e930:	mov    0x8(%r12),%r10
  12e935:	movslq %r11d,%rcx
  12e938:	mov    %rdx,(%rsp)
  12e93c:	jle    12ea1b <PRIVATE0000000000221173+0xbbb>
  12e942:	mov    %r11d,%edi
  12e945:	xor    %r15d,%r15d
  12e948:	mov    $0x1,%edx
  12e94d:	xor    %r9d,%r9d
  12e950:	shr    %edi
  12e952:	je     12e9e1 <PRIVATE0000000000221173+0xb81>
  12e958:	mov    %rbx,0x110(%rsp)
  12e960:	mov    $0x1,%r8d
  12e966:	mov    %r12,0xe0(%rsp)
  12e96e:	mov    %edi,%edx
  12e970:	mov    %r13,0xd8(%rsp)
  12e978:	mov    (%rsp),%r12
  12e97c:	mov    0x8(%rsp),%rbx
  12e981:	mov    0x108(%rsp),%rdi
  12e989:	movslq (%r10,%r15,8),%rsi
  12e98d:	mov    %esi,(%rbx,%r15,8)
  12e991:	mov    (%r9,%r12,1),%r13
  12e995:	mov    %r8d,(%rdi,%rsi,4)
  12e999:	mov    %r13,(%r14,%rsi,8)
  12e99d:	movslq 0x4(%r10,%r15,8),%rsi
  12e9a2:	mov    %esi,0x4(%rbx,%r15,8)
  12e9a7:	inc    %r15
  12e9aa:	mov    0x8(%r9,%r12,1),%r13
  12e9af:	add    $0x10,%r9
  12e9b3:	mov    %r8d,(%rdi,%rsi,4)
  12e9b7:	mov    %r13,(%r14,%rsi,8)
  12e9bb:	cmp    %rdx,%r15
  12e9be:	jb     12e989 <PRIVATE0000000000221173+0xb29>
  12e9c0:	mov    0x110(%rsp),%rbx
  12e9c8:	lea    (%r15,%r15,1),%esi
  12e9cc:	mov    0xd8(%rsp),%r13
  12e9d4:	lea    0x1(%r15,%r15,1),%edx
  12e9d9:	mov    0xe0(%rsp),%r12
  12e9e1:	lea    -0x1(%rdx),%r9d
  12e9e5:	cmp    %r11d,%r9d
  12e9e8:	jae    12ea1b <PRIVATE0000000000221173+0xbbb>
  12e9ea:	movslq %edx,%rdx
  12e9ed:	mov    (%rsp),%r11
  12e9f1:	mov    0x108(%rsp),%r9
  12e9f9:	movslq -0x4(%r10,%rdx,4),%rsi
  12e9fe:	mov    0x8(%rsp),%r10
  12ea03:	mov    -0x8(%r11,%rdx,8),%r15
  12ea08:	mov    %r15,(%r14,%rsi,8)
  12ea0c:	mov    %esi,-0x4(%r10,%rdx,4)
  12ea11:	movl   $0x1,(%r9,%rsi,4)
  12ea19:	mov    %edx,%esi
  12ea1b:	pxor   %xmm0,%xmm0
  12ea1f:	pxor   %xmm1,%xmm1
  12ea23:	cvtsi2sd %eax,%xmm0
  12ea27:	cvtsi2sd %esi,%xmm1
  12ea2b:	addsd  %xmm0,%xmm0
  12ea2f:	mulsd  0x292fc09(%rip),%xmm1        # 2a5e640 <_fini+0x1a3c>
  12ea37:	movslq %eax,%rax
  12ea3a:	addsd  %xmm1,%xmm0
  12ea3e:	movsd  0x292fbda(%rip),%xmm1        # 2a5e620 <_fini+0x1a1c>
  12ea46:	lea    -0x3(%rax,%rax,2),%r15
  12ea4b:	shl    $0x3,%r15
  12ea4f:	dec    %rax
  12ea52:	jl     12eb4a <PRIVATE0000000000221173+0xcea>
  12ea58:	mov    %rbx,0x110(%rsp)
  12ea60:	xor    %r8d,%r8d
  12ea63:	mov    %r13,0xd8(%rsp)
  12ea6b:	pxor   %xmm2,%xmm2
  12ea6f:	mov    %r12,0xe0(%rsp)
  12ea77:	mov    0x8(%rsp),%r9
  12ea7c:	mov    0x108(%rsp),%rbx
  12ea84:	mov    0x50(%rsp),%rdi
  12ea89:	movslq (%r15,%rdi,1),%rdx
  12ea8d:	mov    0x8(%r15,%rdi,1),%r10
  12ea92:	mov    0x10(%r15,%rdi,1),%r11
  12ea97:	movsd  (%r14,%rdx,8),%xmm3
  12ea9d:	ucomisd %xmm2,%xmm3
  12eaa1:	jp     12eaa5 <PRIVATE0000000000221173+0xc45>
  12eaa3:	je     12eb1e <PRIVATE0000000000221173+0xcbe>
  12eaa5:	pxor   %xmm4,%xmm4
  12eaa9:	mov    %r8,%rdx
  12eaac:	mov    0x4(%r15,%rdi,1),%r12d
  12eab1:	cvtsi2sd %r12d,%xmm4
  12eab6:	mulsd  %xmm1,%xmm4
  12eaba:	addsd  %xmm4,%xmm0
  12eabe:	test   %r12d,%r12d
  12eac1:	jle    12eb1e <PRIVATE0000000000221173+0xcbe>
  12eac3:	movslq (%r10,%rdx,4),%rsi
  12eac7:	movslq (%r15,%rdi,1),%r13
  12eacb:	cmp    %r13,%rsi
  12eace:	je     12eb00 <PRIVATE0000000000221173+0xca0>
  12ead0:	movsd  (%r11),%xmm4
  12ead5:	mulsd  %xmm3,%xmm4
  12ead9:	cmpl   $0x0,(%rbx,%rsi,4)
  12eadd:	addsd  (%r14,%rsi,8),%xmm4
  12eae3:	movsd  %xmm4,(%r14,%rsi,8)
  12eae9:	jne    12eb0f <PRIVATE0000000000221173+0xcaf>
  12eaeb:	movl   $0x1,(%rbx,%rsi,4)
  12eaf2:	mov    %esi,(%r9,%rcx,4)
  12eaf6:	inc    %rcx
  12eaf9:	mov    0x4(%r15,%rdi,1),%r12d
  12eafe:	jmp    12eb0f <PRIVATE0000000000221173+0xcaf>
  12eb00:	movsd  (%r11),%xmm4
  12eb05:	mulsd  %xmm3,%xmm4
  12eb09:	movsd  %xmm4,(%r14,%rsi,8)
  12eb0f:	movslq %r12d,%r12
  12eb12:	inc    %rdx
  12eb15:	add    $0x8,%r11
  12eb19:	cmp    %r12,%rdx
  12eb1c:	jl     12eac3 <PRIVATE0000000000221173+0xc63>
  12eb1e:	nopl   0x0(%rax)
  12eb25:	add    $0xffffffffffffffe8,%r15
  12eb29:	dec    %rax
  12eb2c:	jns    12ea89 <PRIVATE0000000000221173+0xc29>
  12eb32:	mov    0x110(%rsp),%rbx
  12eb3a:	mov    0xd8(%rsp),%r13
  12eb42:	mov    0xe0(%rsp),%r12
  12eb4a:	mov    %ecx,%eax
  12eb4c:	pxor   %xmm2,%xmm2
  12eb50:	mov    (%r12),%edx
  12eb54:	xor    %r11d,%r11d
  12eb57:	neg    %edx
  12eb59:	add    %eax,%edx
  12eb5b:	cvtsi2sd %edx,%xmm2
  12eb5f:	mulsd  0x292fad1(%rip),%xmm2        # 2a5e638 <_fini+0x1a34>
  12eb67:	xor    %edx,%edx
  12eb69:	mov    0x8(%r13),%r15
  12eb6d:	addsd  %xmm2,%xmm0
  12eb71:	mov    0x10(%r13),%r10
  12eb75:	test   %rcx,%rcx
  12eb78:	jle    12ebcd <PRIVATE0000000000221173+0xd6d>
  12eb7a:	movsd  0x100(%rsp),%xmm4
  12eb83:	xor    %esi,%esi
  12eb85:	mov    0x8(%rsp),%rdi
  12eb8a:	mov    0x108(%rsp),%r8
  12eb92:	movslq (%rdi,%rdx,4),%r9
  12eb96:	movsd  (%r14,%r9,8),%xmm3
  12eb9c:	movaps %xmm3,%xmm2
  12eb9f:	andps  0x292fa4a(%rip),%xmm2        # 2a5e5f0 <_fini+0x19ec>
  12eba6:	comisd %xmm4,%xmm2
  12ebaa:	jb     12ebb9 <PRIVATE0000000000221173+0xd59>
  12ebac:	mov    %r9d,(%r15,%r11,4)
  12ebb0:	movsd  %xmm3,(%r10,%r11,8)
  12ebb6:	inc    %r11
  12ebb9:	inc    %rdx
  12ebbc:	mov    %rsi,(%r14,%r9,8)
  12ebc0:	movl   $0x0,(%r8,%r9,4)
  12ebc8:	cmp    %rcx,%rdx
  12ebcb:	jl     12eb92 <PRIVATE0000000000221173+0xd32>
  12ebcd:	pxor   %xmm2,%xmm2
  12ebd1:	cvtsi2sd %eax,%xmm2
  12ebd5:	mulsd  %xmm2,%xmm1
  12ebd9:	mov    %r11d,0x0(%r13)
  12ebdd:	addsd  %xmm1,%xmm0
  12ebe1:	addsd  0x200(%rbx),%xmm0
  12ebe9:	movsd  %xmm0,0x200(%rbx)
  12ebf1:	test   %r11d,%r11d
  12ebf4:	je     12ec08 <PRIVATE0000000000221173+0xda8>
  12ebf6:	pxor   %xmm4,%xmm4
  12ebfa:	cvtsi2sd %ebp,%xmm4
  12ebfe:	jmp    12ec5a <PRIVATE0000000000221173+0xdfa>
  12ec00:	movl   $0x0,0x0(%r13)
  12ec08:	movl   $0x0,(%r12)
  12ec10:	add    $0x118,%rsp
  12ec17:	pop    %rbp
  12ec18:	pop    %rbx
  12ec19:	pop    %r15
  12ec1b:	pop    %r14
  12ec1d:	pop    %r13
  12ec1f:	pop    %r12
  12ec21:	ret    
  12ec22:	pxor   %xmm4,%xmm4
  12ec26:	mov    0x8(%r13),%rcx
  12ec2a:	mov    0x10(%r13),%rsi
  12ec2e:	mov    0x10(%r12),%r10
  12ec33:	cvtsi2sd %ebp,%xmm4
  12ec37:	mov    0x0(%r13),%eax
  12ec3b:	mov    0x8(%r12),%r15
  12ec40:	mov    %eax,(%r12)
  12ec44:	mov    %rcx,0x8(%r12)
  12ec49:	mov    %rsi,0x10(%r12)
  12ec4e:	mov    %r11d,0x0(%r13)
  12ec52:	mov    %r15,0x8(%r13)
  12ec56:	mov    %r10,0x10(%r13)
  12ec5a:	test   %r11d,%r11d
  12ec5d:	jl     12fee6 <PRIVATE0000000000221173+0x2086>
  12ec63:	mov    0x58(%rsp),%eax
  12ec67:	imul   %r11d,%eax
  12ec6b:	cmp    %ebp,%eax
  12ec6d:	jg     12fee6 <PRIVATE0000000000221173+0x2086>
  12ec73:	pxor   %xmm3,%xmm3
  12ec77:	mov    0x140(%rbx),%esi
  12ec7d:	cvtsi2sd %esi,%xmm3
  12ec81:	movsd  0x292f9a7(%rip),%xmm2        # 2a5e630 <_fini+0x1a2c>
  12ec89:	mulsd  %xmm4,%xmm2
  12ec8d:	comisd %xmm2,%xmm3
  12ec91:	jae    12fee6 <PRIVATE0000000000221173+0x2086>
  12ec97:	mov    0x78(%rbx),%rdx
  12ec9b:	mov    %ebp,%r8d
  12ec9e:	mov    0x70(%rbx),%rax
  12eca2:	movslq %ebp,%rbp
  12eca5:	mov    %rdx,0xb0(%rsp)
  12ecad:	mov    %rax,0xd0(%rsp)
  12ecb5:	mov    0xe4(%rbx),%edx
  12ecbb:	sub    %edx,%r8d
  12ecbe:	mov    0x58(%rbx),%rax
  12ecc2:	movslq %edx,%rdx
  12ecc5:	mov    0x80(%rbx),%r9
  12eccc:	mov    (%rax,%rbp,8),%rdi
  12ecd0:	mov    %r9,0xa8(%rsp)
  12ecd8:	sub    (%rax,%rdx,8),%rdi
  12ecdc:	mov    0xd8(%rbx),%ecx
  12ece2:	mov    %rbp,0xa0(%rsp)
  12ecea:	mov    %rdx,0x18(%rsp)
  12ecef:	test   %r11d,%r11d
  12ecf2:	je     12fecc <PRIVATE0000000000221173+0x206c>
  12ecf8:	mov    0x8(%r12),%rax
  12ecfd:	mov    0x10(%r12),%r9
  12ed02:	mov    %rax,0xc8(%rsp)
  12ed0a:	mov    %r9,0xc0(%rsp)
  12ed12:	xor    %r9d,%r9d
  12ed15:	cmp    $0x5,%r8d
  12ed19:	jle    12ef17 <PRIVATE0000000000221173+0x10b7>
  12ed1f:	pxor   %xmm0,%xmm0
  12ed23:	pxor   %xmm1,%xmm1
  12ed27:	cvtsi2sd %r8d,%xmm0
  12ed2c:	cvtsi2sd %rdi,%xmm1
  12ed31:	movsd  0x292f92f(%rip),%xmm6        # 2a5e668 <_fini+0x1a64>
  12ed39:	mulsd  %xmm0,%xmm6
  12ed3d:	movaps %xmm6,%xmm5
  12ed40:	mulsd  %xmm0,%xmm5
  12ed44:	comisd %xmm4,%xmm5
  12ed48:	jbe    12ef2e <PRIVATE0000000000221173+0x10ce>
  12ed4e:	add    %r8d,%r8d
  12ed51:	cmp    %ecx,%r8d
  12ed54:	jle    12ef2e <PRIVATE0000000000221173+0x10ce>
  12ed5a:	comisd %xmm6,%xmm1
  12ed5e:	jbe    12ef2e <PRIVATE0000000000221173+0x10ce>
  12ed64:	test   %r11d,%r11d
  12ed67:	jle    12ef2e <PRIVATE0000000000221173+0x10ce>
  12ed6d:	cmp    $0x8,%r11d
  12ed71:	jl     1303df <PRIVATE0000000000221173+0x257f>
  12ed77:	mov    %r15,%r8
  12ed7a:	and    $0xf,%r8
  12ed7e:	mov    %r8d,%r8d
  12ed81:	test   %r8d,%r8d
  12ed84:	je     12ed9e <PRIVATE0000000000221173+0xf3e>
  12ed86:	test   $0x3,%r8d
  12ed8d:	jne    1303df <PRIVATE0000000000221173+0x257f>
  12ed93:	neg    %r8d
  12ed96:	add    $0x10,%r8d
  12ed9a:	shr    $0x2,%r8d
  12ed9e:	lea    0x8(%r8),%eax
  12eda2:	cmp    %eax,%r11d
  12eda5:	jl     1303df <PRIVATE0000000000221173+0x257f>
  12edab:	mov    %r11d,%eax
  12edae:	xor    %edi,%edi
  12edb0:	sub    %r8d,%eax
  12edb3:	and    $0x7,%eax
  12edb6:	neg    %eax
  12edb8:	add    %r11d,%eax
  12edbb:	mov    %r8,(%rsp)
  12edbf:	test   %r8d,%r8d
  12edc2:	jbe    12edef <PRIVATE0000000000221173+0xf8f>
  12edc4:	mov    %r12,0xe0(%rsp)
  12edcc:	mov    %r8,%r12
  12edcf:	cmp    (%r15,%rdi,4),%edx
  12edd3:	lea    0x1(%r9),%r8d
  12edd7:	cmovle %r8d,%r9d
  12eddb:	inc    %rdi
  12edde:	cmp    %r12,%rdi
  12ede1:	jb     12edcf <PRIVATE0000000000221173+0xf6f>
  12ede3:	mov    %r12,(%rsp)
  12ede7:	mov    0xe0(%rsp),%r12
  12edef:	movd   %edx,%xmm5
  12edf3:	movd   %r9d,%xmm7
  12edf8:	pshufd $0x0,%xmm5,%xmm6
  12edfd:	movslq %eax,%rdi
  12ee00:	pcmpeqd %xmm5,%xmm5
  12ee04:	mov    (%rsp),%r9
  12ee08:	movdqa %xmm6,%xmm8
  12ee0d:	movdqa %xmm6,%xmm9
  12ee12:	pcmpgtd (%r15,%r9,4),%xmm8
  12ee18:	pcmpgtd 0x10(%r15,%r9,4),%xmm9
  12ee1f:	pxor   %xmm5,%xmm8
  12ee24:	add    $0x8,%r9
  12ee28:	psubd  %xmm8,%xmm7
  12ee2d:	pxor   %xmm5,%xmm9
  12ee32:	psubd  %xmm9,%xmm7
  12ee37:	cmp    %rdi,%r9
  12ee3a:	jb     12ee08 <PRIVATE0000000000221173+0xfa8>
  12ee3c:	movdqa %xmm7,%xmm8
  12ee41:	lea    0x1(%rax),%r8d
  12ee45:	psrldq $0x8,%xmm8
  12ee4b:	paddd  %xmm8,%xmm7
  12ee50:	movdqa %xmm7,%xmm9
  12ee55:	psrlq  $0x20,%xmm9
  12ee5b:	paddd  %xmm9,%xmm7
  12ee60:	movd   %xmm7,%r9d
  12ee65:	cmp    %r11d,%r8d
  12ee68:	ja     12ef2e <PRIVATE0000000000221173+0x10ce>
  12ee6e:	movslq %r11d,%r8
  12ee71:	sub    %rdi,%r8
  12ee74:	mov    %r8,0x8(%rsp)
  12ee79:	cmp    $0x4,%r8
  12ee7d:	jl     1303d2 <PRIVATE0000000000221173+0x2572>
  12ee83:	lea    (%r15,%rdi,4),%rdi
  12ee87:	movd   %r9d,%xmm7
  12ee8c:	xor    %r9d,%r9d
  12ee8f:	and    $0xfffffffc,%r8d
  12ee93:	movslq %r8d,%r8
  12ee96:	mov    %r8,(%rsp)
  12ee9a:	movdqa %xmm6,%xmm8
  12ee9f:	pcmpgtd (%rdi,%r9,4),%xmm8
  12eea5:	add    $0x4,%r9
  12eea9:	pxor   %xmm5,%xmm8
  12eeae:	psubd  %xmm8,%xmm7
  12eeb3:	cmp    %r8,%r9
  12eeb6:	jb     12ee9a <PRIVATE0000000000221173+0x103a>
  12eeb8:	movdqa %xmm7,%xmm5
  12eebc:	psrldq $0x8,%xmm5
  12eec1:	paddd  %xmm5,%xmm7
  12eec5:	movdqa %xmm7,%xmm6
  12eec9:	psrlq  $0x20,%xmm6
  12eece:	paddd  %xmm6,%xmm7
  12eed2:	mov    %r8,(%rsp)
  12eed6:	movd   %xmm7,%r9d
  12eedb:	mov    0x8(%rsp),%rdi
  12eee0:	cmp    (%rsp),%rdi
  12eee4:	jbe    12ef2e <PRIVATE0000000000221173+0x10ce>
  12eee6:	movslq %eax,%rax
  12eee9:	mov    %r12,0xe0(%rsp)
  12eef1:	mov    (%rsp),%r12
  12eef5:	lea    (%r15,%rax,4),%rax
  12eef9:	cmp    (%rax,%r12,4),%edx
  12eefd:	lea    0x1(%r9),%r8d
  12ef01:	cmovle %r8d,%r9d
  12ef05:	inc    %r12
  12ef08:	cmp    %rdi,%r12
  12ef0b:	jb     12eef9 <PRIVATE0000000000221173+0x1099>
  12ef0d:	mov    0xe0(%rsp),%r12
  12ef15:	jmp    12ef2e <PRIVATE0000000000221173+0x10ce>
  12ef17:	test   %r8d,%r8d
  12ef1a:	je     12ef3c <PRIVATE0000000000221173+0x10dc>
  12ef1c:	pxor   %xmm0,%xmm0
  12ef20:	pxor   %xmm1,%xmm1
  12ef24:	cvtsi2sd %r8d,%xmm0
  12ef29:	cvtsi2sd %rdi,%xmm1
  12ef2e:	mulsd  0x292f74a(%rip),%xmm1        # 2a5e680 <_fini+0x1a7c>
  12ef36:	divsd  %xmm0,%xmm1
  12ef3a:	jmp    12ef43 <PRIVATE0000000000221173+0x10e3>
  12ef3c:	pxor   %xmm1,%xmm1
  12ef40:	movaps %xmm1,%xmm0
  12ef43:	pxor   %xmm5,%xmm5
  12ef47:	cvtsi2sd %r9d,%xmm5
  12ef4c:	mulsd  %xmm1,%xmm5
  12ef50:	mulsd  %xmm1,%xmm5
  12ef54:	comisd %xmm0,%xmm5
  12ef58:	jbe    12f63c <PRIVATE0000000000221173+0x17dc>
  12ef5e:	pxor   %xmm2,%xmm2
  12ef62:	xor    %eax,%eax
  12ef64:	cvtsi2sd %r11d,%xmm2
  12ef69:	mulsd  0x292f6af(%rip),%xmm2        # 2a5e620 <_fini+0x1a1c>
  12ef71:	xor    %r9d,%r9d
  12ef74:	movsd  0x292f6bc(%rip),%xmm0        # 2a5e638 <_fini+0x1a34>
  12ef7c:	xor    %esi,%esi
  12ef7e:	test   %r11d,%r11d
  12ef81:	jle    12f1a1 <PRIVATE0000000000221173+0x1341>
  12ef87:	movslq %r11d,%r11
  12ef8a:	mov    %edx,0x8(%rsp)
  12ef8e:	mov    %r11,(%rsp)
  12ef92:	mov    %ecx,0x10(%rsp)
  12ef96:	mov    %ebp,0xe8(%rsp)
  12ef9d:	mov    %rbx,0x110(%rsp)
  12efa5:	mov    %r13,0xd8(%rsp)
  12efad:	mov    %r12,0xe0(%rsp)
  12efb5:	mov    0xd0(%rsp),%rdx
  12efbd:	mov    (%r15,%rsi,4),%ebp
  12efc1:	cmp    0x8(%rsp),%ebp
  12efc5:	jl     12efde <PRIVATE0000000000221173+0x117e>
  12efc7:	cmp    %r9d,%ebp
  12efca:	mov    (%r10,%rsi,8),%rcx
  12efce:	cmovg  %ebp,%r9d
  12efd2:	movslq %ebp,%rbp
  12efd5:	mov    %rcx,(%r14,%rbp,8)
  12efd9:	jmp    12f16d <PRIVATE0000000000221173+0x130d>
  12efde:	movslq %ebp,%rdi
  12efe1:	cmp    0x10(%rsp),%ebp
  12efe5:	jge    12effe <PRIVATE0000000000221173+0x119e>
  12efe7:	movsd  (%r14,%rdi,8),%xmm1
  12efed:	addsd  (%r10,%rsi,8),%xmm1
  12eff3:	movsd  %xmm1,(%r14,%rdi,8)
  12eff9:	jmp    12f16d <PRIVATE0000000000221173+0x130d>
  12effe:	pxor   %xmm1,%xmm1
  12f002:	mov    0xc8(%rsp),%rbx
  12f00a:	mov    (%rdx,%rdi,8),%r11
  12f00e:	mov    0x8(%rdx,%rdi,8),%rcx
  12f013:	sub    %r11,%rcx
  12f016:	cvtsi2sd %rcx,%xmm1
  12f01b:	mulsd  %xmm0,%xmm1
  12f01f:	mov    %ebp,(%rbx,%rax,4)
  12f022:	addsd  %xmm1,%xmm2
  12f026:	mov    0xc0(%rsp),%rbp
  12f02e:	movsd  (%r10,%rsi,8),%xmm1
  12f034:	movsd  %xmm1,0x0(%rbp,%rax,8)
  12f03a:	inc    %rax
  12f03d:	mov    0x8(%rdx,%rdi,8),%rbp
  12f042:	cmp    %rbp,%r11
  12f045:	jge    12f16d <PRIVATE0000000000221173+0x130d>
  12f04b:	sub    %r11,%rbp
  12f04e:	xor    %ebx,%ebx
  12f050:	mov    %rbp,%r8
  12f053:	xor    %edi,%edi
  12f055:	mov    $0x1,%r12d
  12f05b:	xor    %ecx,%ecx
  12f05d:	shr    $0x2,%r8
  12f061:	je     12f127 <PRIVATE0000000000221173+0x12c7>
  12f067:	mov    0xa8(%rsp),%r13
  12f06f:	mov    0xb0(%rsp),%r12
  12f077:	mov    %r10,0x20(%rsp)
  12f07c:	lea    0x0(%r13,%r11,4),%r13
  12f081:	lea    (%r12,%r11,8),%r12
  12f085:	movsd  (%rcx,%r12,1),%xmm3
  12f08b:	inc    %rbx
  12f08e:	mulsd  %xmm1,%xmm3
  12f092:	movslq (%rdi,%r13,1),%r10
  12f096:	movsd  (%r14,%r10,8),%xmm4
  12f09c:	subsd  %xmm3,%xmm4
  12f0a0:	movsd  %xmm4,(%r14,%r10,8)
  12f0a6:	movsd  0x8(%rcx,%r12,1),%xmm5
  12f0ad:	mulsd  %xmm1,%xmm5
  12f0b1:	movslq 0x4(%rdi,%r13,1),%r10
  12f0b6:	movsd  (%r14,%r10,8),%xmm6
  12f0bc:	subsd  %xmm5,%xmm6
  12f0c0:	movsd  %xmm6,(%r14,%r10,8)
  12f0c6:	movsd  0x10(%rcx,%r12,1),%xmm7
  12f0cd:	mulsd  %xmm1,%xmm7
  12f0d1:	movslq 0x8(%rdi,%r13,1),%r10
  12f0d6:	movsd  (%r14,%r10,8),%xmm8
  12f0dc:	subsd  %xmm7,%xmm8
  12f0e1:	movsd  %xmm8,(%r14,%r10,8)
  12f0e7:	movsd  0x18(%rcx,%r12,1),%xmm9
  12f0ee:	add    $0x20,%rcx
  12f0f2:	mulsd  %xmm1,%xmm9
  12f0f7:	movslq 0xc(%rdi,%r13,1),%r10
  12f0fc:	add    $0x10,%rdi
  12f100:	movsd  (%r14,%r10,8),%xmm10
  12f106:	subsd  %xmm9,%xmm10
  12f10b:	movsd  %xmm10,(%r14,%r10,8)
  12f111:	cmp    %r8,%rbx
  12f114:	jb     12f085 <PRIVATE0000000000221173+0x1225>
  12f11a:	mov    0x20(%rsp),%r10
  12f11f:	lea    0x1(,%rbx,4),%r12
  12f127:	dec    %r12
  12f12a:	cmp    %rbp,%r12
  12f12d:	jae    12f16d <PRIVATE0000000000221173+0x130d>
  12f12f:	mov    0xa8(%rsp),%rbx
  12f137:	mov    0xb0(%rsp),%rcx
  12f13f:	lea    (%rbx,%r11,4),%rbx
  12f143:	lea    (%rcx,%r11,8),%rcx
  12f147:	movsd  (%rcx,%r12,8),%xmm3
  12f14d:	mulsd  %xmm1,%xmm3
  12f151:	movslq (%rbx,%r12,4),%rdi
  12f155:	inc    %r12
  12f158:	movsd  (%r14,%rdi,8),%xmm4
  12f15e:	subsd  %xmm3,%xmm4
  12f162:	movsd  %xmm4,(%r14,%rdi,8)
  12f168:	cmp    %rbp,%r12
  12f16b:	jb     12f147 <PRIVATE0000000000221173+0x12e7>
  12f16d:	inc    %rsi
  12f170:	cmp    (%rsp),%rsi
  12f174:	jb     12efbd <PRIVATE0000000000221173+0x115d>
  12f17a:	mov    0x8(%rsp),%edx
  12f17e:	mov    0x10(%rsp),%ecx
  12f182:	mov    0xe8(%rsp),%ebp
  12f189:	mov    0x110(%rsp),%rbx
  12f191:	mov    0xd8(%rsp),%r13
  12f199:	mov    0xe0(%rsp),%r12
  12f1a1:	mov    %ecx,%r10d
  12f1a4:	pxor   %xmm1,%xmm1
  12f1a8:	sub    %edx,%r10d
  12f1ab:	xor    %r11d,%r11d
  12f1ae:	add    %r9d,%r10d
  12f1b1:	xor    %r15d,%r15d
  12f1b4:	cvtsi2sd %r10d,%xmm1
  12f1b9:	mulsd  %xmm0,%xmm1
  12f1bd:	movslq %r9d,%rsi
  12f1c0:	addsd  %xmm2,%xmm1
  12f1c4:	lea    0x1(%rsi),%r10
  12f1c8:	cmp    %edx,%r9d
  12f1cb:	jl     12f400 <PRIVATE0000000000221173+0x15a0>
  12f1d1:	mov    0xd0(%rsp),%rdi
  12f1d9:	lea    (%r14,%rsi,8),%rdx
  12f1dd:	movsd  0x100(%rsp),%xmm3
  12f1e6:	mov    %r9d,0x8(%rsp)
  12f1eb:	mov    %ecx,0x10(%rsp)
  12f1ef:	lea    (%rdi,%rsi,8),%rdi
  12f1f3:	sub    0x18(%rsp),%rsi
  12f1f8:	inc    %rsi
  12f1fb:	mov    %rsi,0x20(%rsp)
  12f200:	mov    %rdi,(%rsp)
  12f204:	mov    %r14,0x40(%rsp)
  12f209:	mov    %ebp,0xe8(%rsp)
  12f210:	mov    %rbx,0x110(%rsp)
  12f218:	mov    %r13,0xd8(%rsp)
  12f220:	mov    %r12,0xe0(%rsp)
  12f228:	mov    0xa8(%rsp),%rsi
  12f230:	mov    0xb0(%rsp),%rdi
  12f238:	movsd  (%rdx,%r15,8),%xmm2
  12f23e:	movaps %xmm2,%xmm4
  12f241:	andps  0x292f3a8(%rip),%xmm4        # 2a5e5f0 <_fini+0x19ec>
  12f248:	comisd %xmm3,%xmm4
  12f24c:	jbe    12f3bc <PRIVATE0000000000221173+0x155c>
  12f252:	mov    (%rsp),%rcx
  12f256:	pxor   %xmm4,%xmm4
  12f25a:	mov    %r11d,%ebp
  12f25d:	mov    0xc8(%rsp),%r8
  12f265:	neg    %ebp
  12f267:	mov    (%rcx,%r15,8),%r13
  12f26b:	mov    0x8(%rcx,%r15,8),%rbx
  12f270:	sub    %r13,%rbx
  12f273:	cvtsi2sd %rbx,%xmm4
  12f278:	mulsd  %xmm0,%xmm4
  12f27c:	mov    0xc0(%rsp),%r9
  12f284:	addsd  %xmm4,%xmm1
  12f288:	mov    0xd0(%rsp),%r12
  12f290:	add    0x8(%rsp),%ebp
  12f294:	mov    %ebp,(%r8,%rax,4)
  12f298:	movsd  %xmm2,(%r9,%rax,8)
  12f29e:	inc    %rax
  12f2a1:	mov    (%r12,%r10,8),%r12
  12f2a5:	cmp    %r12,%r13
  12f2a8:	jge    12f3bc <PRIVATE0000000000221173+0x155c>
  12f2ae:	sub    %r13,%r12
  12f2b1:	xor    %r14d,%r14d
  12f2b4:	mov    %r12,%r9
  12f2b7:	xor    %r8d,%r8d
  12f2ba:	mov    $0x1,%ecx
  12f2bf:	xor    %ebp,%ebp
  12f2c1:	shr    $0x2,%r9
  12f2c5:	je     12f381 <PRIVATE0000000000221173+0x1521>
  12f2cb:	lea    (%rsi,%r13,4),%rcx
  12f2cf:	mov    0x40(%rsp),%rsi
  12f2d4:	lea    (%rdi,%r13,8),%rbx
  12f2d8:	movsd  0x0(%rbp,%rbx,1),%xmm4
  12f2de:	inc    %r14
  12f2e1:	mulsd  %xmm2,%xmm4
  12f2e5:	movslq (%r8,%rcx,1),%rdi
  12f2e9:	movsd  (%rsi,%rdi,8),%xmm5
  12f2ee:	subsd  %xmm4,%xmm5
  12f2f2:	movsd  %xmm5,(%rsi,%rdi,8)
  12f2f7:	movsd  0x8(%rbp,%rbx,1),%xmm6
  12f2fd:	mulsd  %xmm2,%xmm6
  12f301:	movslq 0x4(%r8,%rcx,1),%rdi
  12f306:	movsd  (%rsi,%rdi,8),%xmm7
  12f30b:	subsd  %xmm6,%xmm7
  12f30f:	movsd  %xmm7,(%rsi,%rdi,8)
  12f314:	movsd  0x10(%rbp,%rbx,1),%xmm8
  12f31b:	mulsd  %xmm2,%xmm8
  12f320:	movslq 0x8(%r8,%rcx,1),%rdi
  12f325:	movsd  (%rsi,%rdi,8),%xmm9
  12f32b:	subsd  %xmm8,%xmm9
  12f330:	movsd  %xmm9,(%rsi,%rdi,8)
  12f336:	movsd  0x18(%rbp,%rbx,1),%xmm10
  12f33d:	add    $0x20,%rbp
  12f341:	mulsd  %xmm2,%xmm10
  12f346:	movslq 0xc(%r8,%rcx,1),%rdi
  12f34b:	add    $0x10,%r8
  12f34f:	movsd  (%rsi,%rdi,8),%xmm11
  12f355:	subsd  %xmm10,%xmm11
  12f35a:	movsd  %xmm11,(%rsi,%rdi,8)
  12f360:	cmp    %r9,%r14
  12f363:	jb     12f2d8 <PRIVATE0000000000221173+0x1478>
  12f369:	mov    0xa8(%rsp),%rsi
  12f371:	lea    0x1(,%r14,4),%rcx
  12f379:	mov    0xb0(%rsp),%rdi
  12f381:	dec    %rcx
  12f384:	cmp    %r12,%rcx
  12f387:	jae    12f3bc <PRIVATE0000000000221173+0x155c>
  12f389:	mov    0x40(%rsp),%r9
  12f38e:	lea    (%rsi,%r13,4),%rbp
  12f392:	lea    (%rdi,%r13,8),%rbx
  12f396:	movsd  (%rbx,%rcx,8),%xmm4
  12f39b:	mulsd  %xmm2,%xmm4
  12f39f:	movslq 0x0(%rbp,%rcx,4),%r8
  12f3a4:	inc    %rcx
  12f3a7:	movsd  (%r9,%r8,8),%xmm5
  12f3ad:	subsd  %xmm4,%xmm5
  12f3b1:	movsd  %xmm5,(%r9,%r8,8)
  12f3b7:	cmp    %r12,%rcx
  12f3ba:	jb     12f396 <PRIVATE0000000000221173+0x1536>
  12f3bc:	inc    %r11
  12f3bf:	dec    %r10
  12f3c2:	movq   $0x0,(%rdx,%r15,8)
  12f3ca:	dec    %r15
  12f3cd:	cmp    0x20(%rsp),%r11
  12f3d2:	jb     12f238 <PRIVATE0000000000221173+0x13d8>
  12f3d8:	mov    0x10(%rsp),%ecx
  12f3dc:	mov    0x40(%rsp),%r14
  12f3e1:	mov    0xe8(%rsp),%ebp
  12f3e8:	mov    0x110(%rsp),%rbx
  12f3f0:	mov    0xd8(%rsp),%r13
  12f3f8:	mov    0xe0(%rsp),%r12
  12f400:	mov    %ecx,%edx
  12f402:	xor    %r10d,%r10d
  12f405:	xor    %r11d,%r11d
  12f408:	dec    %edx
  12f40a:	js     12fe81 <PRIVATE0000000000221173+0x2021>
  12f410:	movslq %ecx,%rsi
  12f413:	mov    0xd0(%rsp),%rdx
  12f41b:	mov    %rsi,%r9
  12f41e:	mov    %ecx,0x10(%rsp)
  12f422:	mov    %rbx,0x110(%rsp)
  12f42a:	mov    %rsi,0x8(%rsp)
  12f42f:	lea    (%rdx,%rsi,8),%r15
  12f433:	mov    %r15,(%rsp)
  12f437:	lea    (%r14,%rsi,8),%rdx
  12f43b:	mov    %r14,0x40(%rsp)
  12f440:	mov    %ebp,0xe8(%rsp)
  12f447:	mov    %r13,0xd8(%rsp)
  12f44f:	mov    %r12,0xe0(%rsp)
  12f457:	movsd  0x100(%rsp),%xmm3
  12f460:	mov    0xa8(%rsp),%rcx
  12f468:	mov    0xb0(%rsp),%rbx
  12f470:	movsd  -0x8(%rdx,%r11,8),%xmm2
  12f477:	movaps %xmm2,%xmm4
  12f47a:	andps  0x292f16f(%rip),%xmm4        # 2a5e5f0 <_fini+0x19ec>
  12f481:	comisd %xmm3,%xmm4
  12f485:	jbe    12f5fb <PRIVATE0000000000221173+0x179b>
  12f48b:	mov    (%rsp),%rbp
  12f48f:	pxor   %xmm4,%xmm4
  12f493:	mov    %r10d,%edi
  12f496:	mov    0xc8(%rsp),%r8
  12f49e:	neg    %edi
  12f4a0:	mov    -0x8(%rbp,%r11,8),%r13
  12f4a5:	mov    0x0(%rbp,%r11,8),%rsi
  12f4aa:	sub    %r13,%rsi
  12f4ad:	cvtsi2sd %rsi,%xmm4
  12f4b2:	mulsd  %xmm0,%xmm4
  12f4b6:	mov    0xc0(%rsp),%r12
  12f4be:	addsd  %xmm4,%xmm1
  12f4c2:	mov    0xd0(%rsp),%r14
  12f4ca:	add    0x10(%rsp),%edi
  12f4ce:	dec    %edi
  12f4d0:	mov    %edi,(%r8,%rax,4)
  12f4d4:	movsd  %xmm2,(%r12,%rax,8)
  12f4da:	inc    %rax
  12f4dd:	mov    (%r14,%r9,8),%rbp
  12f4e1:	cmp    %rbp,%r13
  12f4e4:	jge    12f5fb <PRIVATE0000000000221173+0x179b>
  12f4ea:	sub    %r13,%rbp
  12f4ed:	xor    %esi,%esi
  12f4ef:	mov    %rbp,%r14
  12f4f2:	xor    %r15d,%r15d
  12f4f5:	mov    $0x1,%r8d
  12f4fb:	xor    %edi,%edi
  12f4fd:	shr    $0x2,%r14
  12f501:	je     12f5be <PRIVATE0000000000221173+0x175e>
  12f507:	lea    (%rcx,%r13,4),%r8
  12f50b:	mov    0x40(%rsp),%rcx
  12f510:	lea    (%rbx,%r13,8),%r12
  12f514:	movsd  (%rdi,%r12,1),%xmm4
  12f51a:	inc    %rsi
  12f51d:	mulsd  %xmm2,%xmm4
  12f521:	movslq (%r15,%r8,1),%rbx
  12f525:	movsd  (%rcx,%rbx,8),%xmm5
  12f52a:	subsd  %xmm4,%xmm5
  12f52e:	movsd  %xmm5,(%rcx,%rbx,8)
  12f533:	movsd  0x8(%rdi,%r12,1),%xmm6
  12f53a:	mulsd  %xmm2,%xmm6
  12f53e:	movslq 0x4(%r15,%r8,1),%rbx
  12f543:	movsd  (%rcx,%rbx,8),%xmm7
  12f548:	subsd  %xmm6,%xmm7
  12f54c:	movsd  %xmm7,(%rcx,%rbx,8)
  12f551:	movsd  0x10(%rdi,%r12,1),%xmm8
  12f558:	mulsd  %xmm2,%xmm8
  12f55d:	movslq 0x8(%r15,%r8,1),%rbx
  12f562:	movsd  (%rcx,%rbx,8),%xmm9
  12f568:	subsd  %xmm8,%xmm9
  12f56d:	movsd  %xmm9,(%rcx,%rbx,8)
  12f573:	movsd  0x18(%rdi,%r12,1),%xmm10
  12f57a:	add    $0x20,%rdi
  12f57e:	mulsd  %xmm2,%xmm10
  12f583:	movslq 0xc(%r15,%r8,1),%rbx
  12f588:	add    $0x10,%r15
  12f58c:	movsd  (%rcx,%rbx,8),%xmm11
  12f592:	subsd  %xmm10,%xmm11
  12f597:	movsd  %xmm11,(%rcx,%rbx,8)
  12f59d:	cmp    %r14,%rsi
  12f5a0:	jb     12f514 <PRIVATE0000000000221173+0x16b4>
  12f5a6:	mov    0xa8(%rsp),%rcx
  12f5ae:	lea    0x1(,%rsi,4),%r8
  12f5b6:	mov    0xb0(%rsp),%rbx
  12f5be:	dec    %r8
  12f5c1:	cmp    %rbp,%r8
  12f5c4:	jae    12f5fb <PRIVATE0000000000221173+0x179b>
  12f5c6:	lea    (%rcx,%r13,4),%rdi
  12f5ca:	lea    (%rbx,%r13,8),%rsi
  12f5ce:	mov    0x40(%rsp),%r13
  12f5d3:	movsd  (%rsi,%r8,8),%xmm4
  12f5d9:	mulsd  %xmm2,%xmm4
  12f5dd:	movslq (%rdi,%r8,4),%r12
  12f5e1:	inc    %r8
  12f5e4:	movsd  0x0(%r13,%r12,8),%xmm5
  12f5eb:	subsd  %xmm4,%xmm5
  12f5ef:	movsd  %xmm5,0x0(%r13,%r12,8)
  12f5f6:	cmp    %rbp,%r8
  12f5f9:	jb     12f5d3 <PRIVATE0000000000221173+0x1773>
  12f5fb:	inc    %r10
  12f5fe:	dec    %r9
  12f601:	movq   $0x0,-0x8(%rdx,%r11,8)
  12f60a:	dec    %r11
  12f60d:	cmp    0x8(%rsp),%r10
  12f612:	jb     12f470 <PRIVATE0000000000221173+0x1610>
  12f618:	mov    0xe8(%rsp),%ebp
  12f61f:	mov    0x110(%rsp),%rbx
  12f627:	mov    0xd8(%rsp),%r13
  12f62f:	mov    0xe0(%rsp),%r12
  12f637:	jmp    12fe81 <PRIVATE0000000000221173+0x2021>
  12f63c:	test   %esi,%esi
  12f63e:	jl     130443 <PRIVATE0000000000221173+0x25e3>
  12f644:	comisd %xmm3,%xmm2
  12f648:	jbe    130443 <PRIVATE0000000000221173+0x25e3>
  12f64e:	mov    0x130(%rbx),%rax
  12f655:	mov    %rax,0xb8(%rsp)
  12f65d:	lea    0x1(%rsi),%eax
  12f660:	movslq %esi,%rsi
  12f663:	mov    %eax,0x88(%rsp)
  12f66a:	mov    0x1a8(%rbx),%rdx
  12f671:	mov    0xa0(%rsp),%rax
  12f679:	mov    0x138(%rbx),%r9
  12f680:	mov    %rdx,0x98(%rsp)
  12f688:	mov    %r9,0x90(%rsp)
  12f690:	lea    (%rdx,%rax,4),%rdi
  12f694:	mov    %r15,0xf0(%rsp)
  12f69c:	lea    0x4(,%rsi,4),%rdx
  12f6a4:	xor    %esi,%esi
  12f6a6:	mov    %r10,0x20(%rsp)
  12f6ab:	mov    %r11d,0x80(%rsp)
  12f6b3:	call   bc770 <memset@plt>
  12f6b8:	mov    0x80(%rsp),%r11d
  12f6c0:	mov    0x20(%rsp),%r10
  12f6c5:	test   %r11d,%r11d
  12f6c8:	jle    12fe3e <PRIVATE0000000000221173+0x1fde>
  12f6ce:	xor    %ecx,%ecx
  12f6d0:	mov    %r11d,%edi
  12f6d3:	xor    %r9d,%r9d
  12f6d6:	xor    %edx,%edx
  12f6d8:	shr    $0x4,%edi
  12f6db:	je     130400 <PRIVATE0000000000221173+0x25a0>
  12f6e1:	mov    0x98(%rsp),%rax
  12f6e9:	mov    0xa0(%rsp),%rsi
  12f6f1:	mov    %rdi,(%rsp)
  12f6f5:	mov    %ebp,0xe8(%rsp)
  12f6fc:	lea    (%rax,%rsi,4),%rax
  12f700:	mov    %rax,0x78(%rsp)
  12f705:	mov    $0x1,%eax
  12f70a:	mov    %rbx,0x110(%rsp)
  12f712:	mov    %r15,0xf0(%rsp)
  12f71a:	mov    %rcx,0x8(%rsp)
  12f71f:	mov    %r11d,0x80(%rsp)
  12f727:	mov    %r13,0xd8(%rsp)
  12f72f:	mov    %r12,0xe0(%rsp)
  12f737:	mov    0x78(%rsp),%r8
  12f73c:	mov    0x98(%rsp),%rbx
  12f744:	mov    0x90(%rsp),%rbp
  12f74c:	mov    0xb8(%rsp),%rsi
  12f754:	mov    0x108(%rsp),%rdi
  12f75c:	mov    0xf0(%rsp),%r11
  12f764:	movslq (%r9,%r11,1),%r12
  12f768:	movslq (%rsi,%r12,4),%rcx
  12f76c:	mov    0x0(%rbp,%rcx,4),%r13d
  12f771:	add    (%r8,%rcx,4),%r13d
  12f775:	movslq %r13d,%r13
  12f778:	mov    %r12d,(%rbx,%r13,4)
  12f77c:	incl   (%r8,%rcx,4)
  12f780:	mov    (%rdx,%r10,1),%r13
  12f784:	mov    %eax,(%rdi,%r12,4)
  12f788:	mov    %r13,(%r14,%r12,8)
  12f78c:	movslq 0x4(%r9,%r11,1),%r12
  12f791:	movslq (%rsi,%r12,4),%r13
  12f795:	mov    %r13,0x10(%rsp)
  12f79a:	mov    0x0(%rbp,%r13,4),%r15d
  12f79f:	add    (%r8,%r13,4),%r15d
  12f7a3:	movslq %r15d,%r15
  12f7a6:	mov    %r12d,(%rbx,%r15,4)
  12f7aa:	incl   (%r8,%r13,4)
  12f7ae:	mov    0x8(%rdx,%r10,1),%r13
  12f7b3:	mov    %eax,(%rdi,%r12,4)
  12f7b7:	mov    %r13,(%r14,%r12,8)
  12f7bb:	movslq 0x8(%r9,%r11,1),%r12
  12f7c0:	movslq (%rsi,%r12,4),%r13
  12f7c4:	mov    %r13,0x18(%rsp)
  12f7c9:	mov    0x0(%rbp,%r13,4),%r15d
  12f7ce:	add    (%r8,%r13,4),%r15d
  12f7d2:	movslq %r15d,%r15
  12f7d5:	mov    %r12d,(%rbx,%r15,4)
  12f7d9:	incl   (%r8,%r13,4)
  12f7dd:	mov    0x10(%rdx,%r10,1),%r13
  12f7e2:	mov    %eax,(%rdi,%r12,4)
  12f7e6:	mov    %r13,(%r14,%r12,8)
  12f7ea:	movslq 0xc(%r9,%r11,1),%r12
  12f7ef:	movslq (%rsi,%r12,4),%r13
  12f7f3:	mov    %r13,0x20(%rsp)
  12f7f8:	mov    0x0(%rbp,%r13,4),%r15d
  12f7fd:	add    (%r8,%r13,4),%r15d
  12f801:	movslq %r15d,%r15
  12f804:	mov    %r12d,(%rbx,%r15,4)
  12f808:	incl   (%r8,%r13,4)
  12f80c:	mov    0x18(%rdx,%r10,1),%r13
  12f811:	mov    %eax,(%rdi,%r12,4)
  12f815:	mov    %r13,(%r14,%r12,8)
  12f819:	movslq 0x10(%r9,%r11,1),%r12
  12f81e:	movslq (%rsi,%r12,4),%r13
  12f822:	mov    %r13,0x28(%rsp)
  12f827:	mov    0x0(%rbp,%r13,4),%r15d
  12f82c:	add    (%r8,%r13,4),%r15d
  12f830:	movslq %r15d,%r15
  12f833:	mov    %r12d,(%rbx,%r15,4)
  12f837:	incl   (%r8,%r13,4)
  12f83b:	mov    0x20(%rdx,%r10,1),%r13
  12f840:	mov    %eax,(%rdi,%r12,4)
  12f844:	mov    %r13,(%r14,%r12,8)
  12f848:	movslq 0x14(%r9,%r11,1),%r12
  12f84d:	movslq (%rsi,%r12,4),%r13
  12f851:	mov    %r13,0x30(%rsp)
  12f856:	mov    0x0(%rbp,%r13,4),%r15d
  12f85b:	add    (%r8,%r13,4),%r15d
  12f85f:	movslq %r15d,%r15
  12f862:	mov    %r12d,(%rbx,%r15,4)
  12f866:	incl   (%r8,%r13,4)
  12f86a:	mov    0x28(%rdx,%r10,1),%r13
  12f86f:	mov    %eax,(%rdi,%r12,4)
  12f873:	mov    %r13,(%r14,%r12,8)
  12f877:	movslq 0x18(%r9,%r11,1),%r12
  12f87c:	xchg   %ax,%ax
  12f87e:	movslq (%rsi,%r12,4),%r13
  12f882:	mov    %r13,0x38(%rsp)
  12f887:	mov    0x0(%rbp,%r13,4),%r15d
  12f88c:	nopl   0x0(%rax)
  12f890:	add    (%r8,%r13,4),%r15d
  12f894:	movslq %r15d,%r15
  12f897:	mov    %r12d,(%rbx,%r15,4)
  12f89b:	incl   (%r8,%r13,4)
  12f89f:	nopl   (%rax)
  12f8a2:	mov    0x30(%rdx,%r10,1),%r13
  12f8a7:	mov    %eax,(%rdi,%r12,4)
  12f8ab:	mov    %r13,(%r14,%r12,8)
  12f8af:	movslq 0x1c(%r9,%r11,1),%r12
  12f8b4:	movslq (%rsi,%r12,4),%r13
  12f8b8:	mov    %r13,0x40(%rsp)
  12f8bd:	mov    0x0(%rbp,%r13,4),%r15d
  12f8c2:	add    (%r8,%r13,4),%r15d
  12f8c6:	movslq %r15d,%r15
  12f8c9:	xchg   %ax,%ax
  12f8cb:	mov    %r12d,(%rbx,%r15,4)
  12f8cf:	incl   (%r8,%r13,4)
  12f8d3:	mov    0x38(%rdx,%r10,1),%r13
  12f8d8:	mov    %eax,(%rdi,%r12,4)
  12f8dc:	mov    %r13,(%r14,%r12,8)
  12f8e0:	movslq 0x20(%r9,%r11,1),%r12
  12f8e5:	movslq (%rsi,%r12,4),%r13
  12f8e9:	mov    %r13,0x48(%rsp)
  12f8ee:	mov    0x0(%rbp,%r13,4),%r15d
  12f8f3:	nopl   0x0(%rax)
  12f8f7:	add    (%r8,%r13,4),%r15d
  12f8fb:	movslq %r15d,%r15
  12f8fe:	mov    %r12d,(%rbx,%r15,4)
  12f902:	incl   (%r8,%r13,4)
  12f906:	mov    0x40(%rdx,%r10,1),%r13
  12f90b:	mov    %eax,(%rdi,%r12,4)
  12f90f:	mov    %r13,(%r14,%r12,8)
  12f913:	movslq 0x24(%r9,%r11,1),%r12
  12f918:	movslq (%rsi,%r12,4),%r13
  12f91c:	mov    %r13,0x50(%rsp)
  12f921:	mov    0x0(%rbp,%r13,4),%r15d
  12f926:	add    (%r8,%r13,4),%r15d
  12f92a:	movslq %r15d,%r15
  12f92d:	mov    %r12d,(%rbx,%r15,4)
  12f931:	incl   (%r8,%r13,4)
  12f935:	mov    0x48(%rdx,%r10,1),%r13
  12f93a:	mov    %eax,(%rdi,%r12,4)
  12f93e:	mov    %r13,(%r14,%r12,8)
  12f942:	movslq 0x28(%r9,%r11,1),%r12
  12f947:	movslq (%rsi,%r12,4),%r13
  12f94b:	mov    %r13,0x58(%rsp)
  12f950:	mov    0x0(%rbp,%r13,4),%r15d
  12f955:	add    (%r8,%r13,4),%r15d
  12f959:	movslq %r15d,%r15
  12f95c:	nopl   (%rax)
  12f95f:	mov    %r12d,(%rbx,%r15,4)
  12f963:	incl   (%r8,%r13,4)
  12f967:	mov    0x50(%rdx,%r10,1),%r13
  12f96c:	nopl   0x0(%rax)
  12f970:	mov    %eax,(%rdi,%r12,4)
  12f974:	mov    %r13,(%r14,%r12,8)
  12f978:	xchg   %ax,%ax
  12f97a:	movslq 0x2c(%r9,%r11,1),%r12
  12f97f:	movslq (%rsi,%r12,4),%r13
  12f983:	mov    %r13,0x60(%rsp)
  12f988:	xchg   %ax,%ax
  12f98a:	mov    0x0(%rbp,%r13,4),%r15d
  12f98f:	add    (%r8,%r13,4),%r15d
  12f993:	movslq %r15d,%r15
  12f996:	mov    %r12d,(%rbx,%r15,4)
  12f99a:	nopl   0x0(%rax)
  12f99e:	incl   (%r8,%r13,4)
  12f9a2:	mov    0x58(%rdx,%r10,1),%r13
  12f9a7:	nop
  12f9a8:	mov    %eax,(%rdi,%r12,4)
  12f9ac:	mov    %r13,(%r14,%r12,8)
  12f9b0:	movslq 0x30(%r9,%r11,1),%r13
  12f9b5:	movslq (%rsi,%r13,4),%r12
  12f9b9:	mov    %r12,0x68(%rsp)
  12f9be:	mov    0x0(%rbp,%r12,4),%r15d
  12f9c3:	add    (%r8,%r12,4),%r15d
  12f9c7:	movslq %r15d,%r15
  12f9ca:	mov    %r13d,(%rbx,%r15,4)
  12f9ce:	nopl   0x0(%rax)
  12f9d2:	incl   (%r8,%r12,4)
  12f9d6:	mov    0x60(%rdx,%r10,1),%r12
  12f9db:	mov    %eax,(%rdi,%r13,4)
  12f9df:	mov    %r12,(%r14,%r13,8)
  12f9e3:	movslq 0x34(%r9,%r11,1),%r13
  12f9e8:	movslq (%rsi,%r13,4),%r12
  12f9ec:	mov    %r12,0x70(%rsp)
  12f9f1:	mov    0x0(%rbp,%r12,4),%r15d
  12f9f6:	add    (%r8,%r12,4),%r15d
  12f9fa:	movslq %r15d,%r15
  12f9fd:	nopl   0x0(%rax)
  12fa01:	mov    %r13d,(%rbx,%r15,4)
  12fa05:	incl   (%r8,%r12,4)
  12fa09:	mov    0x68(%rdx,%r10,1),%r12
  12fa0e:	mov    %eax,(%rdi,%r13,4)
  12fa12:	mov    %r12,(%r14,%r13,8)
  12fa16:	movslq 0x38(%r9,%r11,1),%r12
  12fa1b:	movslq (%rsi,%r12,4),%r13
  12fa1f:	mov    0x0(%rbp,%r13,4),%r15d
  12fa24:	add    (%r8,%r13,4),%r15d
  12fa28:	movslq %r15d,%r15
  12fa2b:	mov    %r12d,(%rbx,%r15,4)
  12fa2f:	incl   (%r8,%r13,4)
  12fa33:	nop
  12fa34:	mov    %eax,(%rdi,%r12,4)
  12fa38:	movslq 0x3c(%r9,%r11,1),%r11
  12fa3d:	add    $0x40,%r9
  12fa41:	mov    0x70(%rdx,%r10,1),%r15
  12fa46:	mov    %r15,(%r14,%r12,8)
  12fa4a:	movslq (%rsi,%r11,4),%r12
  12fa4e:	mov    0x0(%rbp,%r12,4),%r15d
  12fa53:	add    (%r8,%r12,4),%r15d
  12fa57:	movslq %r15d,%r15
  12fa5a:	mov    %r11d,(%rbx,%r15,4)
  12fa5e:	mov    %ecx,%r15d
  12fa61:	mov    0x88(%rsp),%ecx
  12fa68:	cmp    %r15d,%ecx
  12fa6b:	cmovl  %ecx,%r15d
  12fa6f:	nopl   (%rax)
  12fa72:	mov    0x10(%rsp),%rcx
  12fa77:	cmp    %ecx,%r15d
  12fa7a:	cmovl  %r15d,%ecx
  12fa7e:	mov    0x18(%rsp),%r15
  12fa83:	cmp    %r15d,%ecx
  12fa86:	cmovl  %ecx,%r15d
  12fa8a:	mov    0x20(%rsp),%rcx
  12fa8f:	cmp    %ecx,%r15d
  12fa92:	cmovl  %r15d,%ecx
  12fa96:	xchg   %ax,%ax
  12fa98:	mov    0x28(%rsp),%r15
  12fa9d:	cmp    %r15d,%ecx
  12faa0:	cmovl  %ecx,%r15d
  12faa4:	mov    0x30(%rsp),%rcx
  12faa9:	cmp    %ecx,%r15d
  12faac:	cmovl  %r15d,%ecx
  12fab0:	mov    0x38(%rsp),%r15
  12fab5:	cmp    %r15d,%ecx
  12fab8:	cmovl  %ecx,%r15d
  12fabc:	mov    0x40(%rsp),%rcx
  12fac1:	cmp    %ecx,%r15d
  12fac4:	cmovl  %r15d,%ecx
  12fac8:	mov    0x48(%rsp),%r15
  12facd:	cmp    %r15d,%ecx
  12fad0:	cmovl  %ecx,%r15d
  12fad4:	mov    0x50(%rsp),%rcx
  12fad9:	cmp    %ecx,%r15d
  12fadc:	cmovl  %r15d,%ecx
  12fae0:	mov    0x58(%rsp),%r15
  12fae5:	cmp    %r15d,%ecx
  12fae8:	cmovl  %ecx,%r15d
  12faec:	mov    0x60(%rsp),%rcx
  12faf1:	cmp    %ecx,%r15d
  12faf4:	cmovl  %r15d,%ecx
  12faf8:	mov    0x68(%rsp),%r15
  12fafd:	cmp    %r15d,%ecx
  12fb00:	cmovl  %ecx,%r15d
  12fb04:	mov    0x70(%rsp),%rcx
  12fb09:	cmp    %ecx,%r15d
  12fb0c:	cmovl  %r15d,%ecx
  12fb10:	cmp    %r13d,%ecx
  12fb13:	cmovl  %ecx,%r13d
  12fb17:	mov    0x78(%rdx,%r10,1),%rcx
  12fb1c:	add    $0x80,%rdx
  12fb23:	incl   (%r8,%r12,4)
  12fb27:	cmp    %r12d,%r13d
  12fb2a:	mov    %eax,(%rdi,%r11,4)
  12fb2e:	mov    %rcx,(%r14,%r11,8)
  12fb32:	cmovl  %r13d,%r12d
  12fb36:	mov    0x8(%rsp),%r11
  12fb3b:	inc    %r11
  12fb3e:	mov    %r12d,0x88(%rsp)
  12fb46:	mov    %r11,0x8(%rsp)
  12fb4b:	cmp    (%rsp),%r11
  12fb4f:	jb     12f75c <PRIVATE0000000000221173+0x18fc>
  12fb55:	mov    %r11,%rcx
  12fb58:	mov    0x80(%rsp),%r11d
  12fb60:	shl    $0x4,%ecx
  12fb63:	movslq %r11d,%r11
  12fb66:	mov    0xf0(%rsp),%r15
  12fb6e:	mov    0xe8(%rsp),%ebp
  12fb75:	mov    0x110(%rsp),%rbx
  12fb7d:	lea    0x1(%rcx),%eax
  12fb80:	movslq %eax,%rax
  12fb83:	dec    %rax
  12fb86:	mov    0xd8(%rsp),%r13
  12fb8e:	mov    0xe0(%rsp),%r12
  12fb96:	cmp    %r11,%rax
  12fb99:	jae    12fc40 <PRIVATE0000000000221173+0x1de0>
  12fb9f:	mov    %rbx,0x110(%rsp)
  12fba7:	mov    %r13,0xd8(%rsp)
  12fbaf:	mov    %r12,0xe0(%rsp)
  12fbb7:	mov    0x78(%rsp),%r8
  12fbbc:	mov    0x88(%rsp),%r9d
  12fbc4:	mov    0x98(%rsp),%rdx
  12fbcc:	mov    0x90(%rsp),%rcx
  12fbd4:	mov    0xb8(%rsp),%rbx
  12fbdc:	mov    0x108(%rsp),%rdi
  12fbe4:	movslq (%r15,%rax,4),%r13
  12fbe8:	movslq (%rbx,%r13,4),%r12
  12fbec:	mov    (%rcx,%r12,4),%esi
  12fbf0:	add    (%r8,%r12,4),%esi
  12fbf4:	movslq %esi,%rsi
  12fbf7:	mov    %r13d,(%rdx,%rsi,4)
  12fbfb:	incl   (%r8,%r12,4)
  12fbff:	cmp    %r12d,%r9d
  12fc02:	movl   $0x1,(%rdi,%r13,4)
  12fc0a:	cmovge %r12d,%r9d
  12fc0e:	mov    (%r10,%rax,8),%r12
  12fc12:	inc    %rax
  12fc15:	mov    %r12,(%r14,%r13,8)
  12fc19:	cmp    %r11,%rax
  12fc1c:	jb     12fbe4 <PRIVATE0000000000221173+0x1d84>
  12fc1e:	mov    %r9d,0x88(%rsp)
  12fc26:	mov    %eax,%ecx
  12fc28:	mov    0x110(%rsp),%rbx
  12fc30:	mov    0xd8(%rsp),%r13
  12fc38:	mov    0xe0(%rsp),%r12
  12fc40:	movslq 0x88(%rsp),%rdx
  12fc48:	xor    %eax,%eax
  12fc4a:	test   %ecx,%ecx
  12fc4c:	jle    12fe40 <PRIVATE0000000000221173+0x1fe0>
  12fc52:	mov    0x78(%rsp),%r10
  12fc57:	xor    %r9d,%r9d
  12fc5a:	mov    0x90(%rsp),%r11
  12fc62:	mov    %ebp,0xe8(%rsp)
  12fc69:	mov    %rbx,0x110(%rsp)
  12fc71:	lea    (%r10,%rdx,4),%r10
  12fc75:	mov    %r13,0xd8(%rsp)
  12fc7d:	lea    (%r11,%rdx,4),%r11
  12fc81:	mov    %r12,0xe0(%rsp)
  12fc89:	movsd  0x100(%rsp),%xmm0
  12fc92:	mov    0x78(%rsp),%r13
  12fc97:	mov    0x98(%rsp),%r12
  12fc9f:	mov    0x90(%rsp),%rbp
  12fca7:	mov    0x108(%rsp),%rbx
  12fcaf:	inc    %r9
  12fcb2:	cmpl   $0x0,-0x4(%r10,%r9,4)
  12fcb8:	je     12fe15 <PRIVATE0000000000221173+0x1fb5>
  12fcbe:	mov    -0x4(%r11,%r9,4),%edi
  12fcc3:	lea    -0x1(%r9,%rdx,1),%rsi
  12fcc8:	mov    0x0(%r13,%rsi,4),%r15d
  12fccd:	add    0x0(%rbp,%rsi,4),%r15d
  12fcd2:	movslq %edi,%r8
  12fcd5:	cmp    %r15d,%edi
  12fcd8:	jge    12fe15 <PRIVATE0000000000221173+0x1fb5>
  12fcde:	mov    %r11,(%rsp)
  12fce2:	mov    %r10,0x8(%rsp)
  12fce7:	mov    %r9,0x10(%rsp)
  12fcec:	mov    %rdx,0x18(%rsp)
  12fcf1:	movslq (%r12,%r8,4),%r10
  12fcf5:	dec    %ecx
  12fcf7:	movsd  (%r14,%r10,8),%xmm1
  12fcfd:	movaps %xmm1,%xmm2
  12fd00:	andps  0x292e8e9(%rip),%xmm2        # 2a5e5f0 <_fini+0x19ec>
  12fd07:	movl   $0x0,(%rbx,%r10,4)
  12fd0f:	movq   $0x0,(%r14,%r10,8)
  12fd17:	comisd %xmm2,%xmm0
  12fd1b:	ja     12fdec <PRIVATE0000000000221173+0x1f8c>
  12fd21:	mov    0xc0(%rsp),%r15
  12fd29:	mov    0xc8(%rsp),%rdx
  12fd31:	mov    0xd0(%rsp),%r9
  12fd39:	movsd  %xmm1,(%r15,%rax,8)
  12fd3f:	mov    %r10d,(%rdx,%rax,4)
  12fd43:	inc    %rax
  12fd46:	mov    (%r9,%r10,8),%r15
  12fd4a:	mov    0x8(%r9,%r10,8),%rdx
  12fd4f:	xor    %r9d,%r9d
  12fd52:	cmp    %rdx,%r15
  12fd55:	jge    12fdec <PRIVATE0000000000221173+0x1f8c>
  12fd5b:	mov    0xa8(%rsp),%r10
  12fd63:	sub    %r15,%rdx
  12fd66:	mov    0xb0(%rsp),%r11
  12fd6e:	mov    %rax,0x20(%rsp)
  12fd73:	mov    %r8,0x30(%rsp)
  12fd78:	lea    (%r10,%r15,4),%r10
  12fd7c:	mov    %edi,0x28(%rsp)
  12fd80:	lea    (%r11,%r15,8),%r11
  12fd84:	mov    0xb8(%rsp),%rax
  12fd8c:	movslq (%r10,%r9,4),%rdi
  12fd90:	cmpl   $0x0,(%rbx,%rdi,4)
  12fd94:	jne    12fdb9 <PRIVATE0000000000221173+0x1f59>
  12fd96:	movslq (%rax,%rdi,4),%r8
  12fd9a:	inc    %ecx
  12fd9c:	movl   $0x1,(%rbx,%rdi,4)
  12fda3:	mov    0x0(%rbp,%r8,4),%r15d
  12fda8:	add    0x0(%r13,%r8,4),%r15d
  12fdad:	movslq %r15d,%r15
  12fdb0:	mov    %edi,(%r12,%r15,4)
  12fdb4:	incl   0x0(%r13,%r8,4)
  12fdb9:	movsd  (%r11,%r9,8),%xmm2
  12fdbf:	inc    %r9
  12fdc2:	mulsd  %xmm1,%xmm2
  12fdc6:	xorps  0x292e833(%rip),%xmm2        # 2a5e600 <_fini+0x19fc>
  12fdcd:	addsd  (%r14,%rdi,8),%xmm2
  12fdd3:	movsd  %xmm2,(%r14,%rdi,8)
  12fdd9:	cmp    %rdx,%r9
  12fddc:	jb     12fd8c <PRIVATE0000000000221173+0x1f2c>
  12fdde:	mov    0x30(%rsp),%r8
  12fde3:	mov    0x28(%rsp),%edi
  12fde7:	mov    0x20(%rsp),%rax
  12fdec:	mov    0x0(%r13,%rsi,4),%edx
  12fdf1:	inc    %edi
  12fdf3:	add    0x0(%rbp,%rsi,4),%edx
  12fdf7:	inc    %r8
  12fdfa:	cmp    %edx,%edi
  12fdfc:	jl     12fcf1 <PRIVATE0000000000221173+0x1e91>
  12fe02:	mov    (%rsp),%r11
  12fe06:	mov    0x8(%rsp),%r10
  12fe0b:	mov    0x10(%rsp),%r9
  12fe10:	mov    0x18(%rsp),%rdx
  12fe15:	test   %ecx,%ecx
  12fe17:	jg     12fcaf <PRIVATE0000000000221173+0x1e4f>
  12fe1d:	mov    0xe8(%rsp),%ebp
  12fe24:	mov    0x110(%rsp),%rbx
  12fe2c:	mov    0xd8(%rsp),%r13
  12fe34:	mov    0xe0(%rsp),%r12
  12fe3c:	jmp    12fe40 <PRIVATE0000000000221173+0x1fe0>
  12fe3e:	xor    %eax,%eax
  12fe40:	pxor   %xmm1,%xmm1
  12fe44:	pxor   %xmm0,%xmm0
  12fe48:	cvtsi2sd %eax,%xmm1
  12fe4c:	mov    0x1e0(%rbx),%rdx
  12fe53:	lea    0x1(%rbp),%r11d
  12fe57:	mov    0xa0(%rsp),%r9
  12fe5f:	pxor   %xmm2,%xmm2
  12fe63:	mulsd  0x292e7d5(%rip),%xmm1        # 2a5e640 <_fini+0x1a3c>
  12fe6b:	cvtsi2sd %r11d,%xmm2
  12fe70:	lea    (%rdx,%r9,2),%r10
  12fe74:	cvtsi2sd %r10,%xmm0
  12fe79:	mulsd  %xmm0,%xmm1
  12fe7d:	divsd  %xmm2,%xmm1
  12fe81:	mov    %eax,(%r12)
  12fe85:	xor    %r9d,%r9d
  12fe88:	addsd  0x200(%rbx),%xmm1
  12fe90:	movsd  %xmm1,0x200(%rbx)
  12fe98:	test   %eax,%eax
  12fe9a:	jl     13042c <PRIVATE0000000000221173+0x25cc>
  12fea0:	jle    13034b <PRIVATE0000000000221173+0x24eb>
  12fea6:	mov    0x8(%r12),%rdx
  12feab:	mov    0x20(%rbx),%rax
  12feaf:	movslq (%rdx),%rcx
  12feb2:	inc    %r9
  12feb5:	mov    (%rax,%rcx,4),%ebx
  12feb8:	mov    %ebx,(%rdx)
  12feba:	add    $0x4,%rdx
  12febe:	movslq (%r12),%rbp
  12fec2:	cmp    %rbp,%r9
  12fec5:	jl     12feaf <PRIVATE0000000000221173+0x204f>
  12fec7:	jmp    13034b <PRIVATE0000000000221173+0x24eb>
  12fecc:	movl   $0x0,(%r12)
  12fed4:	add    $0x118,%rsp
  12fedb:	pop    %rbp
  12fedc:	pop    %rbx
  12fedd:	pop    %r15
  12fedf:	pop    %r14
  12fee1:	pop    %r13
  12fee3:	pop    %r12
  12fee5:	ret    
  12fee6:	mov    0x8(%r12),%r9
  12feeb:	mov    0x10(%r12),%rdx
  12fef0:	movsd  0x200(%rbx),%xmm1
  12fef8:	mov    0x80(%rbx),%rax
  12feff:	mov    0x78(%rbx),%r14
  12ff03:	mov    0x70(%rbx),%rdi
  12ff07:	mov    %rax,0x18(%rsp)
  12ff0c:	mov    %r14,0x10(%rsp)
  12ff11:	test   %r11d,%r11d
  12ff14:	jle    130b35 <PRIVATE0000000000221173+0x2cd5>
  12ff1a:	pxor   %xmm0,%xmm0
  12ff1e:	cvtsi2sd %r11d,%xmm0
  12ff23:	mulsd  0x292e70d(%rip),%xmm0        # 2a5e638 <_fini+0x1a34>
  12ff2b:	addsd  %xmm4,%xmm0
  12ff2f:	addsd  %xmm0,%xmm1
  12ff33:	movsd  %xmm1,0x200(%rbx)
  12ff3b:	test   %ebp,%ebp
  12ff3d:	jle    12ffd2 <PRIVATE0000000000221173+0x2172>
  12ff43:	movslq %ebp,%rsi
  12ff46:	cmp    $0x8,%rsi
  12ff4a:	jl     130b2e <PRIVATE0000000000221173+0x2cce>
  12ff50:	mov    %rdx,%rax
  12ff53:	and    $0xf,%rax
  12ff57:	test   %eax,%eax
  12ff59:	je     12ff68 <PRIVATE0000000000221173+0x2108>
  12ff5b:	test   $0x7,%al
  12ff5d:	jne    130b2e <PRIVATE0000000000221173+0x2cce>
  12ff63:	mov    $0x1,%eax
  12ff68:	mov    %eax,%r8d
  12ff6b:	lea    0x8(%r8),%r14
  12ff6f:	cmp    %r14,%rsi
  12ff72:	jl     130b2e <PRIVATE0000000000221173+0x2cce>
  12ff78:	mov    %ebp,%ecx
  12ff7a:	xor    %r14d,%r14d
  12ff7d:	sub    %eax,%ecx
  12ff7f:	and    $0x7,%ecx
  12ff82:	neg    %ecx
  12ff84:	add    %ebp,%ecx
  12ff86:	movslq %ecx,%rcx
  12ff89:	test   %eax,%eax
  12ff8b:	jbe    12ff9b <PRIVATE0000000000221173+0x213b>
  12ff8d:	xor    %eax,%eax
  12ff8f:	mov    %rax,(%rdx,%r14,8)
  12ff93:	inc    %r14
  12ff96:	cmp    %r8,%r14
  12ff99:	jb     12ff8f <PRIVATE0000000000221173+0x212f>
  12ff9b:	pxor   %xmm0,%xmm0
  12ff9f:	movups %xmm0,(%rdx,%r8,8)
  12ffa4:	movups %xmm0,0x10(%rdx,%r8,8)
  12ffaa:	movups %xmm0,0x20(%rdx,%r8,8)
  12ffb0:	movups %xmm0,0x30(%rdx,%r8,8)
  12ffb6:	add    $0x8,%r8
  12ffba:	cmp    %rcx,%r8
  12ffbd:	jb     12ff9f <PRIVATE0000000000221173+0x213f>
  12ffbf:	cmp    %rsi,%rcx
  12ffc2:	jae    12ffd2 <PRIVATE0000000000221173+0x2172>
  12ffc4:	xor    %eax,%eax
  12ffc6:	mov    %rax,(%rdx,%rcx,8)
  12ffca:	inc    %rcx
  12ffcd:	cmp    %rsi,%rcx
  12ffd0:	jb     12ffc6 <PRIVATE0000000000221173+0x2166>
  12ffd2:	test   %r11d,%r11d
  12ffd5:	jle    13007a <PRIVATE0000000000221173+0x221a>
  12ffdb:	movslq %r11d,%rcx
  12ffde:	cmp    $0x2,%rcx
  12ffe2:	jl     130b26 <PRIVATE0000000000221173+0x2cc6>
  12ffe8:	mov    %r10,%rax
  12ffeb:	and    $0xf,%rax
  12ffef:	test   %eax,%eax
  12fff1:	je     130000 <PRIVATE0000000000221173+0x21a0>
  12fff3:	test   $0x7,%al
  12fff5:	jne    130b26 <PRIVATE0000000000221173+0x2cc6>
  12fffb:	mov    $0x1,%eax
  130000:	mov    %eax,%esi
  130002:	lea    0x2(%rsi),%r14
  130006:	cmp    %r14,%rcx
  130009:	jl     130b26 <PRIVATE0000000000221173+0x2cc6>
  13000f:	mov    %r11d,%r14d
  130012:	sub    %eax,%r14d
  130015:	and    $0x1,%r14d
  130019:	sub    %r14d,%r11d
  13001c:	xor    %r14d,%r14d
  13001f:	movslq %r11d,%r11
  130022:	test   %eax,%eax
  130024:	jbe    13003a <PRIVATE0000000000221173+0x21da>
  130026:	movslq (%r15,%r14,4),%r8
  13002a:	mov    (%r10,%r14,8),%rax
  13002e:	inc    %r14
  130031:	mov    %rax,(%rdx,%r8,8)
  130035:	cmp    %rsi,%r14
  130038:	jb     130026 <PRIVATE0000000000221173+0x21c6>
  13003a:	movslq (%r15,%rsi,4),%rax
  13003e:	movslq 0x4(%r15,%rsi,4),%r14
  130043:	movups (%r10,%rsi,8),%xmm0
  130048:	add    $0x2,%rsi
  13004c:	movsd  %xmm0,(%rdx,%rax,8)
  130051:	psrldq $0x8,%xmm0
  130056:	movsd  %xmm0,(%rdx,%r14,8)
  13005c:	cmp    %r11,%rsi
  13005f:	jb     13003a <PRIVATE0000000000221173+0x21da>
  130061:	cmp    %rcx,%r11
  130064:	jae    13007a <PRIVATE0000000000221173+0x221a>
  130066:	movslq (%r15,%r11,4),%r14
  13006a:	mov    (%r10,%r11,8),%rax
  13006e:	inc    %r11
  130071:	mov    %rax,(%rdx,%r14,8)
  130075:	cmp    %rcx,%r11
  130078:	jb     130066 <PRIVATE0000000000221173+0x2206>
  13007a:	movsd  0x200(%rbx),%xmm1
  130082:	mov    %ebp,%eax
  130084:	xor    %r11d,%r11d
  130087:	xor    %r14d,%r14d
  13008a:	dec    %eax
  13008c:	addsd  %xmm4,%xmm4
  130090:	addsd  %xmm4,%xmm1
  130094:	movsd  %xmm1,0x200(%rbx)
  13009c:	js     1302b8 <PRIVATE0000000000221173+0x2458>
  1300a2:	movslq %ebp,%r8
  1300a5:	mov    %r8,0x30(%rsp)
  1300aa:	mov    %r8,%rax
  1300ad:	mov    %rdi,0x28(%rsp)
  1300b2:	movsd  0x292e57e(%rip),%xmm0        # 2a5e638 <_fini+0x1a34>
  1300ba:	mov    %r15,0xf0(%rsp)
  1300c2:	lea    (%rdi,%r8,8),%rcx
  1300c6:	mov    %r9,(%rsp)
  1300ca:	lea    (%rdx,%r8,8),%rsi
  1300ce:	mov    %rdx,0x8(%rsp)
  1300d3:	mov    %r10,0x20(%rsp)
  1300d8:	mov    %ebp,0xe8(%rsp)
  1300df:	mov    %rbx,0x110(%rsp)
  1300e7:	mov    %r13,0xd8(%rsp)
  1300ef:	mov    %r12,0xe0(%rsp)
  1300f7:	movsd  0x100(%rsp),%xmm2
  130100:	mov    0x10(%rsp),%rdi
  130105:	mov    0x18(%rsp),%r8
  13010a:	movsd  -0x8(%rsi,%r14,8),%xmm1
  130111:	andps  0x292e4d8(%rip),%xmm1        # 2a5e5f0 <_fini+0x19ec>
  130118:	comisd %xmm2,%xmm1
  13011c:	jbe    130277 <PRIVATE0000000000221173+0x2417>
  130122:	mov    -0x8(%rcx,%r14,8),%r13
  130127:	pxor   %xmm1,%xmm1
  13012b:	mov    (%rcx,%r14,8),%rdx
  13012f:	sub    %r13,%rdx
  130132:	cvtsi2sd %rdx,%xmm1
  130137:	mulsd  %xmm0,%xmm1
  13013b:	mov    0x110(%rsp),%rbx
  130143:	mov    0x28(%rsp),%rbp
  130148:	addsd  0x200(%rbx),%xmm1
  130150:	movsd  %xmm1,0x200(%rbx)
  130158:	mov    0x0(%rbp,%rax,8),%r12
  13015d:	movsd  -0x8(%rsi,%r14,8),%xmm1
  130164:	cmp    %r12,%r13
  130167:	jge    130277 <PRIVATE0000000000221173+0x2417>
  13016d:	sub    %r13,%r12
  130170:	xor    %r15d,%r15d
  130173:	mov    %r12,%r10
  130176:	xor    %r9d,%r9d
  130179:	mov    $0x1,%edx
  13017e:	xor    %ebp,%ebp
  130180:	shr    $0x2,%r10
  130184:	je     13023c <PRIVATE0000000000221173+0x23dc>
  13018a:	lea    (%r8,%r13,4),%rbx
  13018e:	lea    (%rdi,%r13,8),%rdx
  130192:	mov    0x8(%rsp),%rdi
  130197:	movsd  0x0(%rbp,%rdx,1),%xmm3
  13019d:	inc    %r15
  1301a0:	mulsd  %xmm1,%xmm3
  1301a4:	movslq (%r9,%rbx,1),%r8
  1301a8:	movsd  (%rdi,%r8,8),%xmm4
  1301ae:	subsd  %xmm3,%xmm4
  1301b2:	movsd  %xmm4,(%rdi,%r8,8)
  1301b8:	movsd  0x8(%rbp,%rdx,1),%xmm5
  1301be:	mulsd  %xmm1,%xmm5
  1301c2:	movslq 0x4(%r9,%rbx,1),%r8
  1301c7:	movsd  (%rdi,%r8,8),%xmm6
  1301cd:	subsd  %xmm5,%xmm6
  1301d1:	movsd  %xmm6,(%rdi,%r8,8)
  1301d7:	movsd  0x10(%rbp,%rdx,1),%xmm7
  1301dd:	mulsd  %xmm1,%xmm7
  1301e1:	movslq 0x8(%r9,%rbx,1),%r8
  1301e6:	movsd  (%rdi,%r8,8),%xmm8
  1301ec:	subsd  %xmm7,%xmm8
  1301f1:	movsd  %xmm8,(%rdi,%r8,8)
  1301f7:	movsd  0x18(%rbp,%rdx,1),%xmm9
  1301fe:	add    $0x20,%rbp
  130202:	mulsd  %xmm1,%xmm9
  130207:	movslq 0xc(%r9,%rbx,1),%r8
  13020c:	add    $0x10,%r9
  130210:	movsd  (%rdi,%r8,8),%xmm10
  130216:	subsd  %xmm9,%xmm10
  13021b:	movsd  %xmm10,(%rdi,%r8,8)
  130221:	cmp    %r10,%r15
  130224:	jb     130197 <PRIVATE0000000000221173+0x2337>
  13022a:	mov    0x10(%rsp),%rdi
  13022f:	lea    0x1(,%r15,4),%rdx
  130237:	mov    0x18(%rsp),%r8
  13023c:	dec    %rdx
  13023f:	cmp    %r12,%rdx
  130242:	jae    130277 <PRIVATE0000000000221173+0x2417>
  130244:	mov    0x8(%rsp),%r10
  130249:	lea    (%r8,%r13,4),%rbp
  13024d:	lea    (%rdi,%r13,8),%rbx
  130251:	movsd  (%rbx,%rdx,8),%xmm3
  130256:	mulsd  %xmm1,%xmm3
  13025a:	movslq 0x0(%rbp,%rdx,4),%r9
  13025f:	inc    %rdx
  130262:	movsd  (%r10,%r9,8),%xmm4
  130268:	subsd  %xmm3,%xmm4
  13026c:	movsd  %xmm4,(%r10,%r9,8)
  130272:	cmp    %r12,%rdx
  130275:	jb     130251 <PRIVATE0000000000221173+0x23f1>
  130277:	inc    %r11
  13027a:	dec    %r14
  13027d:	dec    %rax
  130280:	cmp    0x30(%rsp),%r11
  130285:	jb     13010a <PRIVATE0000000000221173+0x22aa>
  13028b:	mov    0xf0(%rsp),%r15
  130293:	mov    (%rsp),%r9
  130297:	mov    0x8(%rsp),%rdx
  13029c:	mov    0x20(%rsp),%r10
  1302a1:	mov    0xe8(%rsp),%ebp
  1302a8:	mov    0xd8(%rsp),%r13
  1302b0:	mov    0xe0(%rsp),%r12
  1302b8:	mov    $0xffffffff,%eax
  1302bd:	movl   $0xffffffff,(%r12)
  1302c5:	test   %ebp,%ebp
  1302c7:	jle    13032d <PRIVATE0000000000221173+0x24cd>
  1302c9:	mov    %ebp,%edi
  1302cb:	xor    %r11d,%r11d
  1302ce:	mov    $0x1,%ecx
  1302d3:	xor    %r14d,%r14d
  1302d6:	shr    %edi
  1302d8:	je     13030d <PRIVATE0000000000221173+0x24ad>
  1302da:	mov    0xf8(%rsp),%rbx
  1302e2:	movslq (%rbx,%r11,8),%rcx
  1302e6:	movslq 0x4(%rbx,%r11,8),%r8
  1302eb:	inc    %r11
  1302ee:	mov    (%rdx,%rcx,8),%rsi
  1302f2:	mov    %rsi,(%r14,%r10,1)
  1302f6:	mov    (%rdx,%r8,8),%rcx
  1302fa:	mov    %rcx,0x8(%r14,%r10,1)
  1302ff:	add    $0x10,%r14
  130303:	cmp    %rdi,%r11
  130306:	jb     1302e2 <PRIVATE0000000000221173+0x2482>
  130308:	lea    0x1(%r11,%r11,1),%ecx
  13030d:	lea    -0x1(%rcx),%ebx
  130310:	cmp    %ebp,%ebx
  130312:	jae    13032d <PRIVATE0000000000221173+0x24cd>
  130314:	movslq %ecx,%rcx
  130317:	mov    0xf8(%rsp),%rbx
  13031f:	movslq -0x4(%rbx,%rcx,4),%rbp
  130324:	mov    (%rdx,%rbp,8),%rsi
  130328:	mov    %rsi,-0x8(%r10,%rcx,8)
  13032d:	mov    %eax,0x0(%r13)
  130331:	mov    %r15,0x8(%r12)
  130336:	mov    %r10,0x10(%r12)
  13033b:	mov    %r9,0x8(%r13)
  13033f:	mov    %rdx,0x10(%r13)
  130343:	movl   $0xffffffff,(%r12)
  13034b:	add    $0x118,%rsp
  130352:	pop    %rbp
  130353:	pop    %rbx
  130354:	pop    %r15
  130356:	pop    %r14
  130358:	pop    %r13
  13035a:	pop    %r12
  13035c:	ret    
  13035d:	xor    %r9d,%r9d
  130360:	jmp    12e111 <PRIVATE0000000000221173+0x2b1>
  130365:	xor    %r11d,%r11d
  130368:	jmp    12e060 <PRIVATE0000000000221173+0x200>
  13036d:	mov    (%r12),%edx
  130371:	pxor   %xmm4,%xmm4
  130375:	mov    %r9d,(%r12)
  130379:	mov    0x8(%r13),%r9
  13037d:	mov    %rsi,0x10(%r12)
  130382:	mov    %r10,%rsi
  130385:	mov    0x8(%r12),%rcx
  13038a:	mov    %r10,0x10(%r13)
  13038e:	mov    %r9,0x8(%r12)
  130393:	mov    %edx,0x0(%r13)
  130397:	mov    %rcx,0x8(%r13)
  13039b:	mov    0x10(%r12),%r10
  1303a0:	cvtsi2sd %ebp,%xmm4
  1303a4:	jmp    12e131 <PRIVATE0000000000221173+0x2d1>
  1303a9:	xor    %r11d,%r11d
  1303ac:	jmp    12e74f <PRIVATE0000000000221173+0x8ef>
  1303b1:	xor    %ecx,%ecx
  1303b3:	jmp    12e6a5 <PRIVATE0000000000221173+0x845>
  1303b8:	pxor   %xmm4,%xmm4
  1303bc:	mov    0x8(%r13),%rcx
  1303c0:	mov    0x10(%r13),%rsi
  1303c4:	mov    0x10(%r12),%r10
  1303c9:	cvtsi2sd %ebp,%xmm4
  1303cd:	jmp    12e394 <PRIVATE0000000000221173+0x534>
  1303d2:	movq   $0x0,(%rsp)
  1303da:	jmp    12eedb <PRIVATE0000000000221173+0x107b>
  1303df:	xor    %eax,%eax
  1303e1:	cmp    $0x1,%r11d
  1303e5:	jb     12ef2e <PRIVATE0000000000221173+0x10ce>
  1303eb:	movslq %r11d,%r11
  1303ee:	mov    %r11,0x8(%rsp)
  1303f3:	movq   $0x0,(%rsp)
  1303fb:	jmp    12eedb <PRIVATE0000000000221173+0x107b>
  130400:	movslq %r11d,%r11
  130403:	xor    %eax,%eax
  130405:	test   %r11,%r11
  130408:	jbe    12fe40 <PRIVATE0000000000221173+0x1fe0>
  13040e:	mov    0x98(%rsp),%rdx
  130416:	mov    0xa0(%rsp),%r9
  13041e:	lea    (%rdx,%r9,4),%rdx
  130422:	mov    %rdx,0x78(%rsp)
  130427:	jmp    12fb9f <PRIVATE0000000000221173+0x1d3f>
  13042c:	mov    0x8(%r12),%r9
  130431:	mov    0x10(%r12),%rdx
  130436:	mov    0x8(%r13),%r15
  13043a:	mov    0x10(%r13),%r10
  13043e:	jmp    1302c5 <PRIVATE0000000000221173+0x2465>
  130443:	test   %r11d,%r11d
  130446:	jle    130525 <PRIVATE0000000000221173+0x26c5>
  13044c:	xor    %edx,%edx
  13044e:	mov    %r11d,%esi
  130451:	xor    %r9d,%r9d
  130454:	mov    $0x1,%edi
  130459:	xor    %ecx,%ecx
  13045b:	shr    %esi
  13045d:	je     1304e5 <PRIVATE0000000000221173+0x2685>
  130463:	mov    0xc8(%rsp),%rax
  13046b:	mov    0xa0(%rsp),%rdi
  130473:	mov    0x108(%rsp),%r8
  13047b:	mov    %r13,0xd8(%rsp)
  130483:	lea    (%rax,%rdi,4),%rax
  130487:	mov    %r12,0xe0(%rsp)
  13048f:	mov    $0x1,%r12d
  130495:	movslq (%r15,%rdx,8),%rdi
  130499:	mov    %edi,-0x4(%r9,%rax,1)
  13049e:	mov    (%rcx,%r10,1),%r13
  1304a2:	mov    %r12d,(%r8,%rdi,4)
  1304a6:	mov    %r13,(%r14,%rdi,8)
  1304aa:	movslq 0x4(%r15,%rdx,8),%rdi
  1304af:	inc    %rdx
  1304b2:	mov    0x8(%rcx,%r10,1),%r13
  1304b7:	add    $0x10,%rcx
  1304bb:	mov    %edi,-0x8(%r9,%rax,1)
  1304c0:	add    $0xfffffffffffffff8,%r9
  1304c4:	mov    %r13,(%r14,%rdi,8)
  1304c8:	mov    %r12d,(%r8,%rdi,4)
  1304cc:	cmp    %rsi,%rdx
  1304cf:	jb     130495 <PRIVATE0000000000221173+0x2635>
  1304d1:	mov    0xd8(%rsp),%r13
  1304d9:	lea    0x1(%rdx,%rdx,1),%edi
  1304dd:	mov    0xe0(%rsp),%r12
  1304e5:	lea    -0x1(%rdi),%eax
  1304e8:	cmp    %r11d,%eax
  1304eb:	jae    130525 <PRIVATE0000000000221173+0x26c5>
  1304ed:	movslq %edi,%rdi
  1304f0:	mov    0xa0(%rsp),%rax
  1304f8:	sub    %rdi,%rax
  1304fb:	mov    0x108(%rsp),%rdx
  130503:	movslq -0x4(%r15,%rdi,4),%r9
  130508:	mov    -0x8(%r10,%rdi,8),%r10
  13050d:	mov    0xc8(%rsp),%r15
  130515:	mov    %r10,(%r14,%r9,8)
  130519:	mov    %r9d,(%r15,%rax,4)
  13051d:	movl   $0x1,(%rdx,%r9,4)
  130525:	mov    0xc8(%rsp),%rax
  13052d:	mov    %r11d,%r9d
  130530:	mov    0xa0(%rsp),%rdx
  130538:	neg    %r9d
  13053b:	mov    $0x1,%r8d
  130541:	mov    $0xffffffffffffffff,%rdi
  130548:	lea    -0x4(%rax,%rdx,4),%rax
  13054d:	movslq %r9d,%rdx
  130550:	cmp    $0x1,%r11d
  130554:	jle    1305a7 <PRIVATE0000000000221173+0x2747>
  130556:	mov    (%rax,%rdi,4),%esi
  130559:	mov    %r8d,%ecx
  13055c:	test   %r8d,%r8d
  13055f:	jle    130594 <PRIVATE0000000000221173+0x2734>
  130561:	lea    -0x1(%rcx),%r15d
  130565:	shr    $0x1f,%r15d
  130569:	lea    -0x1(%rcx,%r15,1),%r15d
  13056e:	sar    %r15d
  130571:	mov    %r15d,%r10d
  130574:	neg    %r10d
  130577:	movslq %r10d,%r10
  13057a:	mov    (%rax,%r10,4),%r10d
  13057e:	cmp    %r10d,%esi
  130581:	jl     130594 <PRIVATE0000000000221173+0x2734>
  130583:	neg    %ecx
  130585:	movslq %ecx,%rcx
  130588:	mov    %r10d,(%rax,%rcx,4)
  13058c:	mov    %r15d,%ecx
  13058f:	test   %r15d,%r15d
  130592:	jg     130561 <PRIVATE0000000000221173+0x2701>
  130594:	neg    %ecx
  130596:	inc    %r8d
  130599:	movslq %ecx,%rcx
  13059c:	dec    %rdi
  13059f:	mov    %esi,(%rax,%rcx,4)
  1305a2:	cmp    %r11d,%r8d
  1305a5:	jl     130556 <PRIVATE0000000000221173+0x26f6>
  1305a7:	movq   $0x0,(%rsp)
  1305af:	test   %r11d,%r11d
  1305b2:	jle    13085c <PRIVATE0000000000221173+0x29fc>
  1305b8:	movsd  0x100(%rsp),%xmm1
  1305c1:	xor    %r10d,%r10d
  1305c4:	mov    %ebp,0xe8(%rsp)
  1305cb:	mov    %rbx,0x110(%rsp)
  1305d3:	xorps  0x292e026(%rip),%xmm1        # 2a5e600 <_fini+0x19fc>
  1305da:	movsd  0x292e06e(%rip),%xmm0        # 2a5e650 <_fini+0x1a4c>
  1305e2:	mov    %r13,0xd8(%rsp)
  1305ea:	mov    %r12,0xe0(%rsp)
  1305f2:	movsd  0x100(%rsp),%xmm3
  1305fb:	mov    (%rsp),%rbx
  1305ff:	mov    0xc0(%rsp),%rbp
  130607:	mov    0xc8(%rsp),%rsi
  13060f:	mov    0xd0(%rsp),%rdi
  130617:	mov    0x108(%rsp),%r8
  13061f:	mov    (%rax),%ecx
  130621:	cmp    $0x1,%r11d
  130625:	je     130646 <PRIVATE0000000000221173+0x27e6>
  130627:	pxor   %xmm5,%xmm5
  13062b:	pxor   %xmm2,%xmm2
  13062f:	cvtsi2sd %r11d,%xmm5
  130634:	cvtsi2sd %ecx,%xmm2
  130638:	mulsd  %xmm0,%xmm5
  13063c:	comisd %xmm5,%xmm2
  130640:	jbe    130839 <PRIVATE0000000000221173+0x29d9>
  130646:	inc    %rdx
  130649:	dec    %r11d
  13064c:	mov    $0x1,%r12d
  130652:	inc    %r9d
  130655:	movslq %ecx,%rcx
  130658:	xor    %r15d,%r15d
  13065b:	mov    (%rax,%rdx,4),%r13d
  13065f:	mov    %r13d,(%rax)
  130662:	cmp    $0x1,%r11d
  130666:	jle    1306d5 <PRIVATE0000000000221173+0x2875>
  130668:	mov    %r14,0x40(%rsp)
  13066d:	lea    0x1(%r12),%edi
  130672:	cmp    %r11d,%edi
  130675:	jge    130699 <PRIVATE0000000000221173+0x2839>
  130677:	mov    %r12d,%r8d
  13067a:	neg    %r8d
  13067d:	movslq %r8d,%r8
  130680:	mov    (%rax,%r8,4),%r14d
  130684:	movslq %r12d,%r8
  130687:	shl    $0x2,%r8
  13068b:	neg    %r8
  13068e:	add    %rax,%r8
  130691:	cmp    -0x4(%r8),%r14d
  130695:	cmovl  %edi,%r12d
  130699:	mov    %r12d,%edi
  13069c:	neg    %edi
  13069e:	movslq %edi,%rdi
  1306a1:	mov    (%rax,%rdi,4),%edi
  1306a4:	cmp    %r13d,%edi
  1306a7:	jl     1306c0 <PRIVATE0000000000221173+0x2860>
  1306a9:	neg    %r15d
  1306ac:	movslq %r15d,%r15
  1306af:	mov    %edi,(%rax,%r15,4)
  1306b3:	mov    %r12d,%r15d
  1306b6:	lea    0x1(%r12,%r12,1),%r12d
  1306bb:	cmp    %r11d,%r12d
  1306be:	jl     13066d <PRIVATE0000000000221173+0x280d>
  1306c0:	mov    0xd0(%rsp),%rdi
  1306c8:	mov    0x108(%rsp),%r8
  1306d0:	mov    0x40(%rsp),%r14
  1306d5:	neg    %r15d
  1306d8:	movslq %r15d,%r15
  1306db:	movsd  (%r14,%rcx,8),%xmm2
  1306e1:	mov    %r10,(%r14,%rcx,8)
  1306e5:	comisd %xmm3,%xmm2
  1306e9:	mov    %r13d,(%rax,%r15,4)
  1306ed:	movl   $0x0,(%r8,%rcx,4)
  1306f5:	jae    130701 <PRIVATE0000000000221173+0x28a1>
  1306f7:	comisd %xmm2,%xmm1
  1306fb:	jb     1307cc <PRIVATE0000000000221173+0x296c>
  130701:	mov    %ecx,(%rsi,%rbx,4)
  130704:	mov    (%rdi,%rcx,8),%r12
  130708:	mov    0x8(%rdi,%rcx,8),%rcx
  13070d:	movsd  %xmm2,0x0(%rbp,%rbx,8)
  130713:	inc    %rbx
  130716:	cmp    %rcx,%r12
  130719:	jge    1307cc <PRIVATE0000000000221173+0x296c>
  13071f:	mov    %rbx,(%rsp)
  130723:	mov    0xa8(%rsp),%r15
  13072b:	mov    0xb0(%rsp),%r13
  130733:	movsd  0x0(%r13,%r12,8),%xmm5
  13073a:	mulsd  %xmm2,%xmm5
  13073e:	movslq (%r15,%r12,4),%rdi
  130742:	movsd  (%r14,%rdi,8),%xmm6
  130748:	cmpl   $0x0,(%r8,%rdi,4)
  13074d:	subsd  %xmm5,%xmm6
  130751:	movsd  %xmm6,(%r14,%rdi,8)
  130757:	jne    1307a8 <PRIVATE0000000000221173+0x2948>
  130759:	movslq %r9d,%r9
  13075c:	mov    %r11d,%esi
  13075f:	movl   $0x1,(%r8,%rdi,4)
  130767:	dec    %rdx
  13076a:	inc    %r11d
  13076d:	mov    %edi,(%rax,%r9,4)
  130771:	dec    %r9d
  130774:	test   %esi,%esi
  130776:	jle    1307a0 <PRIVATE0000000000221173+0x2940>
  130778:	lea    -0x1(%rsi),%ebp
  13077b:	shr    $0x1f,%ebp
  13077e:	lea    -0x1(%rsi,%rbp,1),%ebp
  130782:	sar    %ebp
  130784:	mov    %ebp,%ebx
  130786:	neg    %ebx
  130788:	movslq %ebx,%rbx
  13078b:	mov    (%rax,%rbx,4),%ebx
  13078e:	cmp    %ebx,%edi
  130790:	jl     1307a0 <PRIVATE0000000000221173+0x2940>
  130792:	neg    %esi
  130794:	movslq %esi,%rsi
  130797:	mov    %ebx,(%rax,%rsi,4)
  13079a:	mov    %ebp,%esi
  13079c:	test   %ebp,%ebp
  13079e:	jg     130778 <PRIVATE0000000000221173+0x2918>
  1307a0:	neg    %esi
  1307a2:	movslq %esi,%rsi
  1307a5:	mov    %edi,(%rax,%rsi,4)
  1307a8:	inc    %r12
  1307ab:	cmp    %rcx,%r12
  1307ae:	jl     130733 <PRIVATE0000000000221173+0x28d3>
  1307b0:	mov    (%rsp),%rbx
  1307b4:	mov    0xc0(%rsp),%rbp
  1307bc:	mov    0xc8(%rsp),%rsi
  1307c4:	mov    0xd0(%rsp),%rdi
  1307cc:	test   %r11d,%r11d
  1307cf:	jg     13061f <PRIVATE0000000000221173+0x27bf>
  1307d5:	mov    %rbx,(%rsp)
  1307d9:	mov    0xe8(%rsp),%ebp
  1307e0:	mov    0x110(%rsp),%rbx
  1307e8:	mov    0xd8(%rsp),%r13
  1307f0:	mov    0xe0(%rsp),%r12
  1307f8:	pxor   %xmm1,%xmm1
  1307fc:	pxor   %xmm0,%xmm0
  130800:	mov    (%rsp),%rax
  130804:	lea    0x1(%rbp),%edx
  130807:	cvtsi2sd %eax,%xmm1
  13080b:	cvtsi2sdq 0x1e0(%rbx),%xmm0
  130814:	addsd  %xmm4,%xmm4
  130818:	mulsd  0x292de50(%rip),%xmm1        # 2a5e670 <_fini+0x1a6c>
  130820:	addsd  %xmm0,%xmm4
  130824:	pxor   %xmm2,%xmm2
  130828:	cvtsi2sd %edx,%xmm2
  13082c:	mulsd  %xmm4,%xmm1
  130830:	divsd  %xmm2,%xmm1
  130834:	jmp    12fe81 <PRIVATE0000000000221173+0x2021>
  130839:	mov    %rbx,(%rsp)
  13083d:	mov    0xe8(%rsp),%ebp
  130844:	mov    0x110(%rsp),%rbx
  13084c:	mov    0xd8(%rsp),%r13
  130854:	mov    0xe0(%rsp),%r12
  13085c:	test   %r11d,%r11d
  13085f:	jle    1307f8 <PRIVATE0000000000221173+0x2998>
  130861:	jle    1308f5 <PRIVATE0000000000221173+0x2a95>
  130867:	mov    %r11d,%esi
  13086a:	mov    $0x1,%edx
  13086f:	xor    %r10d,%r10d
  130872:	xor    %r15d,%r15d
  130875:	shr    %esi
  130877:	je     1308bf <PRIVATE0000000000221173+0x2a5f>
  130879:	mov    0xc8(%rsp),%r9
  130881:	mov    0xa0(%rsp),%rdx
  130889:	lea    (%r9,%rdx,4),%rcx
  13088d:	mov    0x108(%rsp),%r9
  130895:	mov    %esi,%edx
  130897:	movslq -0x4(%r15,%rcx,1),%rdi
  13089c:	xor    %esi,%esi
  13089e:	inc    %r10
  1308a1:	xor    %r8d,%r8d
  1308a4:	mov    %esi,(%r9,%rdi,4)
  1308a8:	movslq -0x8(%r15,%rcx,1),%rsi
  1308ad:	add    $0xfffffffffffffff8,%r15
  1308b1:	mov    %r8d,(%r9,%rsi,4)
  1308b5:	cmp    %rdx,%r10
  1308b8:	jb     130897 <PRIVATE0000000000221173+0x2a37>
  1308ba:	lea    0x1(%r10,%r10,1),%edx
  1308bf:	lea    -0x1(%rdx),%r9d
  1308c3:	cmp    %r11d,%r9d
  1308c6:	jae    1308f5 <PRIVATE0000000000221173+0x2a95>
  1308c8:	movslq %edx,%rdx
  1308cb:	neg    %rdx
  1308ce:	mov    0xa0(%rsp),%r10
  1308d6:	add    %rdx,%r10
  1308d9:	mov    0xc8(%rsp),%r9
  1308e1:	mov    0x108(%rsp),%r11
  1308e9:	movslq (%r9,%r10,4),%r15
  1308ed:	movl   $0x0,(%r11,%r15,4)
  1308f5:	xor    %r9d,%r9d
  1308f8:	movslq (%rax),%r11
  1308fb:	xor    %r10d,%r10d
  1308fe:	test   %r11,%r11
  130901:	jl     1307f8 <PRIVATE0000000000221173+0x2998>
  130907:	mov    0xd0(%rsp),%rax
  13090f:	lea    0x1(%r11),%r15
  130913:	movsd  0x100(%rsp),%xmm0
  13091c:	lea    (%r14,%r11,8),%rdx
  130920:	mov    %ebp,0xe8(%rsp)
  130927:	mov    %rbx,0x110(%rsp)
  13092f:	lea    (%rax,%r11,8),%rcx
  130933:	mov    %rcx,0x10(%rsp)
  130938:	mov    %rcx,%rax
  13093b:	xorps  0x292dcbe(%rip),%xmm0        # 2a5e600 <_fini+0x19fc>
  130942:	mov    %r11d,0x8(%rsp)
  130947:	mov    %r15,0x18(%rsp)
  13094c:	mov    %r14,0x40(%rsp)
  130951:	mov    %r13,0xd8(%rsp)
  130959:	mov    %r12,0xe0(%rsp)
  130961:	movsd  0x100(%rsp),%xmm2
  13096a:	mov    (%rsp),%rbp
  13096e:	mov    0xa8(%rsp),%rcx
  130976:	mov    0xb0(%rsp),%rbx
  13097e:	movsd  (%rdx,%r10,8),%xmm1
  130984:	movq   $0x0,(%rdx,%r10,8)
  13098c:	comisd %xmm2,%xmm1
  130990:	jae    13099c <PRIVATE0000000000221173+0x2b3c>
  130992:	comisd %xmm1,%xmm0
  130996:	jb     130ae9 <PRIVATE0000000000221173+0x2c89>
  13099c:	mov    %r9d,%esi
  13099f:	mov    0xc8(%rsp),%rdi
  1309a7:	neg    %esi
  1309a9:	mov    0xc0(%rsp),%r8
  1309b1:	mov    0x10(%rsp),%r11
  1309b6:	add    0x8(%rsp),%esi
  1309ba:	mov    %esi,(%rdi,%rbp,4)
  1309bd:	movsd  %xmm1,(%r8,%rbp,8)
  1309c3:	inc    %rbp
  1309c6:	mov    (%r11,%r10,8),%r13
  1309ca:	mov    0x8(%rax),%rsi
  1309ce:	cmp    %rsi,%r13
  1309d1:	jge    130ae9 <PRIVATE0000000000221173+0x2c89>
  1309d7:	sub    %r13,%rsi
  1309da:	xor    %edi,%edi
  1309dc:	mov    %rsi,%r14
  1309df:	xor    %r15d,%r15d
  1309e2:	mov    $0x1,%r8d
  1309e8:	xor    %r11d,%r11d
  1309eb:	shr    $0x2,%r14
  1309ef:	je     130aac <PRIVATE0000000000221173+0x2c4c>
  1309f5:	lea    (%rcx,%r13,4),%r8
  1309f9:	mov    0x40(%rsp),%rcx
  1309fe:	lea    (%rbx,%r13,8),%r12
  130a02:	movsd  (%r11,%r12,1),%xmm3
  130a08:	inc    %rdi
  130a0b:	mulsd  %xmm1,%xmm3
  130a0f:	movslq (%r15,%r8,1),%rbx
  130a13:	movsd  (%rcx,%rbx,8),%xmm5
  130a18:	subsd  %xmm3,%xmm5
  130a1c:	movsd  %xmm5,(%rcx,%rbx,8)
  130a21:	movsd  0x8(%r11,%r12,1),%xmm6
  130a28:	mulsd  %xmm1,%xmm6
  130a2c:	movslq 0x4(%r15,%r8,1),%rbx
  130a31:	movsd  (%rcx,%rbx,8),%xmm7
  130a36:	subsd  %xmm6,%xmm7
  130a3a:	movsd  %xmm7,(%rcx,%rbx,8)
  130a3f:	movsd  0x10(%r11,%r12,1),%xmm8
  130a46:	mulsd  %xmm1,%xmm8
  130a4b:	movslq 0x8(%r15,%r8,1),%rbx
  130a50:	movsd  (%rcx,%rbx,8),%xmm9
  130a56:	subsd  %xmm8,%xmm9
  130a5b:	movsd  %xmm9,(%rcx,%rbx,8)
  130a61:	movsd  0x18(%r11,%r12,1),%xmm10
  130a68:	add    $0x20,%r11
  130a6c:	mulsd  %xmm1,%xmm10
  130a71:	movslq 0xc(%r15,%r8,1),%rbx
  130a76:	add    $0x10,%r15
  130a7a:	movsd  (%rcx,%rbx,8),%xmm11
  130a80:	subsd  %xmm10,%xmm11
  130a85:	movsd  %xmm11,(%rcx,%rbx,8)
  130a8b:	cmp    %r14,%rdi
  130a8e:	jb     130a02 <PRIVATE0000000000221173+0x2ba2>
  130a94:	mov    0xa8(%rsp),%rcx
  130a9c:	lea    0x1(,%rdi,4),%r8
  130aa4:	mov    0xb0(%rsp),%rbx
  130aac:	dec    %r8
  130aaf:	cmp    %rsi,%r8
  130ab2:	jae    130ae9 <PRIVATE0000000000221173+0x2c89>
  130ab4:	lea    (%rcx,%r13,4),%r11
  130ab8:	lea    (%rbx,%r13,8),%rdi
  130abc:	mov    0x40(%rsp),%r13
  130ac1:	movsd  (%rdi,%r8,8),%xmm3
  130ac7:	mulsd  %xmm1,%xmm3
  130acb:	movslq (%r11,%r8,4),%r12
  130acf:	inc    %r8
  130ad2:	movsd  0x0(%r13,%r12,8),%xmm5
  130ad9:	subsd  %xmm3,%xmm5
  130add:	movsd  %xmm5,0x0(%r13,%r12,8)
  130ae4:	cmp    %rsi,%r8
  130ae7:	jb     130ac1 <PRIVATE0000000000221173+0x2c61>
  130ae9:	inc    %r9
  130aec:	add    $0xfffffffffffffff8,%rax
  130af0:	dec    %r10
  130af3:	cmp    0x18(%rsp),%r9
  130af8:	jb     13097e <PRIVATE0000000000221173+0x2b1e>
  130afe:	mov    %rbp,(%rsp)
  130b02:	mov    0xe8(%rsp),%ebp
  130b09:	mov    0x110(%rsp),%rbx
  130b11:	mov    0xd8(%rsp),%r13
  130b19:	mov    0xe0(%rsp),%r12
  130b21:	jmp    1307f8 <PRIVATE0000000000221173+0x2998>
  130b26:	xor    %r11d,%r11d
  130b29:	jmp    130061 <PRIVATE0000000000221173+0x2201>
  130b2e:	xor    %ecx,%ecx
  130b30:	jmp    12ffbf <PRIVATE0000000000221173+0x215f>
  130b35:	mov    %r15,0x8(%r12)
  130b3a:	mov    %r9,%r15
  130b3d:	mov    %r10,0x10(%r12)
  130b42:	mov    %rdx,%r10
  130b45:	mov    (%r12),%eax
  130b49:	mov    %r9,0x8(%r13)
  130b4d:	mov    %rdx,0x10(%r13)
  130b51:	mov    %r11d,(%r12)
  130b55:	mov    %eax,0x0(%r13)
  130b59:	mov    0x10(%r12),%rdx
  130b5e:	mov    0x8(%r12),%r9
  130b63:	jmp    130082 <PRIVATE0000000000221173+0x2222>
  130b68:	test   %r9d,%r9d
  130b6b:	jmp    12df3e <PRIVATE0000000000221173+0xde>
0000000000130b70 <PRIVATE000000000021a941>:
  130b70:	push   %r12
  130b72:	push   %r13
  130b74:	push   %r14
  130b76:	push   %r15
  130b78:	push   %rbx
  130b79:	push   %rbp
  130b7a:	sub    $0x88,%rsp
  130b81:	movaps %xmm0,%xmm1
  130b84:	mov    0x20(%rdx),%r11
  130b88:	mov    %rdi,%r12
  130b8b:	mov    0xa8(%rdx),%rbp
  130b92:	pxor   %xmm0,%xmm0
  130b96:	mov    0x98(%rdx),%r10
  130b9d:	mov    0xb0(%rdx),%rbx
  130ba4:	mov    (%rcx),%r15d
  130ba7:	mov    %r11,0x78(%rsp)
  130bac:	mov    %rbp,0x60(%rsp)
  130bb1:	mov    0xe0(%rdx),%eax
  130bb7:	movsd  0xe8(%rdx),%xmm2
  130bbf:	mov    0xd8(%rdx),%r11
  130bc6:	mov    0x90(%rdx),%r13
  130bcd:	mov    %r10,0x70(%rsp)
  130bd2:	mov    0x28(%rdx),%rdi
  130bd6:	mov    0x30(%rdx),%rbp
  130bda:	mov    %rbx,0x68(%rsp)
  130bdf:	test   %r15d,%r15d
  130be2:	je     1315f5 <PRIVATE000000000021a941+0xa85>
  130be8:	mov    0x8(%r8),%rdx
  130bec:	mov    0x10(%r8),%rbx
  130bf0:	mov    0x8(%rcx),%r10
  130bf4:	mov    0x10(%rcx),%r14
  130bf8:	mov    %rdx,0x48(%rsp)
  130bfd:	xor    %edx,%edx
  130bff:	mov    %rbx,0x40(%rsp)
  130c04:	mov    %rbx,(%rsp)
  130c08:	xor    %ebx,%ebx
  130c0a:	comisd 0x292da2e(%rip),%xmm2        # 2a5e640 <_fini+0x1a3c>
  130c12:	jbe    130fa3 <PRIVATE000000000021a941+0x433>
  130c18:	pxor   %xmm3,%xmm3
  130c1c:	mov    %esi,%edx
  130c1e:	cvtsi2sd %r15d,%xmm3
  130c23:	mulsd  0x292d9f5(%rip),%xmm3        # 2a5e620 <_fini+0x1a1c>
  130c2b:	xor    %eax,%eax
  130c2d:	movsd  0x292da03(%rip),%xmm2        # 2a5e638 <_fini+0x1a34>
  130c35:	test   %r15d,%r15d
  130c38:	jle    130e20 <PRIVATE000000000021a941+0x2b0>
  130c3e:	mov    %esi,0x38(%rsp)
  130c42:	mov    %rcx,0x28(%rsp)
  130c47:	mov    %r11,0x8(%rsp)
  130c4c:	mov    %rdi,0x30(%rsp)
  130c51:	mov    %r13,0x50(%rsp)
  130c56:	mov    %r12,0x10(%rsp)
  130c5b:	mov    %r8,0x18(%rsp)
  130c60:	mov    (%rsp),%rsi
  130c64:	mov    0x68(%rsp),%rcx
  130c69:	movslq (%r10,%rax,4),%rdi
  130c6d:	movsd  (%r14,%rax,8),%xmm5
  130c73:	movaps %xmm5,%xmm0
  130c76:	divsd  0x0(%rbp,%rdi,8),%xmm0
  130c7c:	movaps %xmm0,%xmm4
  130c7f:	andps  0x292d96a(%rip),%xmm4        # 2a5e5f0 <_fini+0x19ec>
  130c86:	comisd %xmm1,%xmm4
  130c8a:	jbe    130df9 <PRIVATE000000000021a941+0x289>
  130c90:	mov    0x8(%rsp),%r8
  130c95:	mov    (%r8,%rdi,4),%r8d
  130c99:	cmp    $0xffffffff,%r8d
  130c9d:	je     131639 <PRIVATE000000000021a941+0xac9>
  130ca3:	jle    13160e <PRIVATE000000000021a941+0xa9e>
  130ca9:	mov    0x48(%rsp),%r11
  130cae:	pxor   %xmm4,%xmm4
  130cb2:	mov    0x50(%rsp),%r15
  130cb7:	mov    0x70(%rsp),%r8
  130cbc:	mov    %edi,(%r11,%rbx,4)
  130cc0:	inc    %rbx
  130cc3:	mov    (%r15,%rdi,8),%r11
  130cc7:	mov    (%r8,%rdi,8),%rdi
  130ccb:	mov    %rdi,%r15
  130cce:	sub    %r11,%r15
  130cd1:	cvtsi2sd %r15,%xmm4
  130cd6:	mulsd  %xmm2,%xmm4
  130cda:	movsd  %xmm0,(%rsi)
  130cde:	add    $0x8,%rsi
  130ce2:	addsd  %xmm4,%xmm3
  130ce6:	cmp    %rdi,%r11
  130ce9:	jge    130df1 <PRIVATE000000000021a941+0x281>
  130cef:	mov    %r15,%r13
  130cf2:	xor    %r12d,%r12d
  130cf5:	mov    $0x1,%edi
  130cfa:	xor    %r9d,%r9d
  130cfd:	shr    %r13
  130d00:	je     130dab <PRIVATE000000000021a941+0x23b>
  130d06:	mov    0x60(%rsp),%rdi
  130d0b:	lea    (%rcx,%r11,4),%r8
  130d0f:	mov    %rbp,0x58(%rsp)
  130d14:	mov    %r10,(%rsp)
  130d18:	mov    0xc0(%rsp),%rcx
  130d20:	lea    (%rdi,%r11,8),%rdi
  130d24:	mov    0x78(%rsp),%rbp
  130d29:	nopl   0x0(%rax)
  130d30:	movsd  (%r9,%rdi,1),%xmm4
  130d36:	mulsd  %xmm0,%xmm4
  130d3a:	movslq (%r8,%r12,8),%r10
  130d3e:	xorps  0x292d8bb(%rip),%xmm4        # 2a5e600 <_fini+0x19fc>
  130d45:	addsd  (%rcx,%r10,8),%xmm4
  130d4b:	movsd  %xmm4,(%rcx,%r10,8)
  130d51:	movsd  0x8(%r9,%rdi,1),%xmm5
  130d58:	add    $0x10,%r9
  130d5c:	mulsd  %xmm0,%xmm5
  130d60:	mov    0x0(%rbp,%r10,4),%r10d
  130d65:	cmp    %edx,%r10d
  130d68:	xorps  0x292d891(%rip),%xmm5        # 2a5e600 <_fini+0x19fc>
  130d6f:	cmovl  %r10d,%edx
  130d73:	movslq 0x4(%r8,%r12,8),%r10
  130d78:	inc    %r12
  130d7b:	addsd  (%rcx,%r10,8),%xmm5
  130d81:	movsd  %xmm5,(%rcx,%r10,8)
  130d87:	mov    0x0(%rbp,%r10,4),%r10d
  130d8c:	cmp    %edx,%r10d
  130d8f:	cmovl  %r10d,%edx
  130d93:	cmp    %r13,%r12
  130d96:	jb     130d30 <PRIVATE000000000021a941+0x1c0>
  130d98:	mov    (%rsp),%r10
  130d9c:	lea    0x1(%r12,%r12,1),%rdi
  130da1:	mov    0x58(%rsp),%rbp
  130da6:	mov    0x68(%rsp),%rcx
  130dab:	lea    -0x1(%rdi),%r8
  130daf:	cmp    %r15,%r8
  130db2:	jae    130df1 <PRIVATE000000000021a941+0x281>
  130db4:	mov    0xc0(%rsp),%r9
  130dbc:	lea    -0x1(%r11,%rdi,1),%r8
  130dc1:	mov    0x60(%rsp),%rdi
  130dc6:	movslq (%rcx,%r8,4),%r12
  130dca:	mov    0x78(%rsp),%r11
  130dcf:	mulsd  (%rdi,%r8,8),%xmm0
  130dd5:	xorps  0x292d824(%rip),%xmm0        # 2a5e600 <_fini+0x19fc>
  130ddc:	cmp    (%r11,%r12,4),%edx
  130de0:	addsd  (%r9,%r12,8),%xmm0
  130de6:	movsd  %xmm0,(%r9,%r12,8)
  130dec:	cmovg  (%r11,%r12,4),%edx
  130df1:	mov    0x28(%rsp),%rdi
  130df6:	mov    (%rdi),%r15d
  130df9:	movslq %r15d,%r15
  130dfc:	inc    %rax
  130dff:	cmp    %r15,%rax
  130e02:	jl     130c69 <PRIVATE000000000021a941+0xf9>
  130e08:	mov    0x30(%rsp),%rdi
  130e0d:	mov    0x50(%rsp),%r13
  130e12:	mov    0x10(%rsp),%r12
  130e17:	mov    0x38(%rsp),%esi
  130e1b:	mov    0x18(%rsp),%r8
  130e20:	mov    %esi,%eax
  130e22:	pxor   %xmm0,%xmm0
  130e26:	sub    %edx,%eax
  130e28:	xor    %ecx,%ecx
  130e2a:	cvtsi2sd %eax,%xmm0
  130e2e:	mulsd  %xmm2,%xmm0
  130e32:	movslq %edx,%rax
  130e35:	addsd  %xmm3,%xmm0
  130e39:	cmp    %esi,%edx
  130e3b:	jge    1315cc <PRIVATE000000000021a941+0xa5c>
  130e41:	movslq %esi,%rsi
  130e44:	sub    %rax,%rsi
  130e47:	lea    (%rdi,%rax,4),%rax
  130e4b:	mov    %rsi,(%rsp)
  130e4f:	mov    %r8,0x18(%rsp)
  130e54:	mov    %r13,0x50(%rsp)
  130e59:	mov    %r12,0x10(%rsp)
  130e5e:	mov    0xc0(%rsp),%rsi
  130e66:	mov    0x60(%rsp),%rdi
  130e6b:	mov    0x68(%rsp),%r8
  130e70:	movslq (%rax,%rcx,4),%r9
  130e74:	movsd  (%rsi,%r9,8),%xmm3
  130e7a:	divsd  0x0(%rbp,%r9,8),%xmm3
  130e81:	movaps %xmm3,%xmm4
  130e84:	andps  0x292d765(%rip),%xmm4        # 2a5e5f0 <_fini+0x19ec>
  130e8b:	comisd %xmm1,%xmm4
  130e8f:	jbe    130f89 <PRIVATE000000000021a941+0x419>
  130e95:	mov    0x70(%rsp),%r11
  130e9a:	pxor   %xmm4,%xmm4
  130e9e:	mov    0x50(%rsp),%r10
  130ea3:	mov    0x48(%rsp),%rdx
  130ea8:	mov    (%r11,%r9,8),%r13
  130eac:	mov    %r13,%r11
  130eaf:	mov    (%r10,%r9,8),%r10
  130eb3:	sub    %r10,%r11
  130eb6:	cvtsi2sd %r11,%xmm4
  130ebb:	mulsd  %xmm2,%xmm4
  130ebf:	mov    0x40(%rsp),%r12
  130ec4:	addsd  %xmm4,%xmm0
  130ec8:	mov    %r9d,(%rdx,%rbx,4)
  130ecc:	movsd  %xmm3,(%r12,%rbx,8)
  130ed2:	inc    %rbx
  130ed5:	cmp    %r13,%r10
  130ed8:	jge    130f89 <PRIVATE000000000021a941+0x419>
  130ede:	mov    %r11,%r12
  130ee1:	xor    %edx,%edx
  130ee3:	mov    $0x1,%r14d
  130ee9:	xor    %r13d,%r13d
  130eec:	shr    %r12
  130eef:	je     130f5b <PRIVATE000000000021a941+0x3eb>
  130ef1:	lea    (%r8,%r10,4),%r15
  130ef5:	lea    (%rdi,%r10,8),%r14
  130ef9:	nopl   0x0(%rax)
  130f00:	movsd  0x0(%r13,%r14,1),%xmm4
  130f07:	mulsd  %xmm3,%xmm4
  130f0b:	movslq (%r15,%rdx,8),%r8
  130f0f:	xorps  0x292d6ea(%rip),%xmm4        # 2a5e600 <_fini+0x19fc>
  130f16:	addsd  (%rsi,%r8,8),%xmm4
  130f1c:	movsd  %xmm4,(%rsi,%r8,8)
  130f22:	movsd  0x8(%r13,%r14,1),%xmm5
  130f29:	add    $0x10,%r13
  130f2d:	mulsd  %xmm3,%xmm5
  130f31:	movslq 0x4(%r15,%rdx,8),%r8
  130f36:	inc    %rdx
  130f39:	xorps  0x292d6c0(%rip),%xmm5        # 2a5e600 <_fini+0x19fc>
  130f40:	addsd  (%rsi,%r8,8),%xmm5
  130f46:	movsd  %xmm5,(%rsi,%r8,8)
  130f4c:	cmp    %r12,%rdx
  130f4f:	jb     130f00 <PRIVATE000000000021a941+0x390>
  130f51:	mov    0x68(%rsp),%r8
  130f56:	lea    0x1(%rdx,%rdx,1),%r14
  130f5b:	lea    -0x1(%r14),%rdx
  130f5f:	cmp    %r11,%rdx
  130f62:	jae    130f89 <PRIVATE000000000021a941+0x419>
  130f64:	lea    -0x1(%r10,%r14,1),%rdx
  130f69:	movsd  (%rdi,%rdx,8),%xmm4
  130f6e:	mulsd  %xmm4,%xmm3
  130f72:	movslq (%r8,%rdx,4),%r10
  130f76:	xorps  0x292d683(%rip),%xmm3        # 2a5e600 <_fini+0x19fc>
  130f7d:	addsd  (%rsi,%r10,8),%xmm3
  130f83:	movsd  %xmm3,(%rsi,%r10,8)
  130f89:	inc    %rcx
  130f8c:	movq   $0x0,(%rsi,%r9,8)
  130f94:	cmp    (%rsp),%rcx
  130f98:	jb     130e70 <PRIVATE000000000021a941+0x300>
  130f9e:	jmp    1315c2 <PRIVATE000000000021a941+0xa52>
  130fa3:	movslq %esi,%rsi
  130fa6:	mov    0x48(%rsp),%r11
  130fab:	mov    %rsi,0x80(%rsp)
  130fb3:	lea    (%r11,%rsi,4),%r11
  130fb7:	test   %r15d,%r15d
  130fba:	jle    131664 <PRIVATE000000000021a941+0xaf4>
  130fc0:	movq   $0x0,0x8(%rsp)
  130fc9:	mov    %r11,(%rsp)
  130fcd:	mov    %rbp,0x58(%rsp)
  130fd2:	mov    %rdi,0x30(%rsp)
  130fd7:	mov    %esi,0x38(%rsp)
  130fdb:	mov    %r8,0x18(%rsp)
  130fe0:	mov    %r11,0x20(%rsp)
  130fe5:	mov    %r13,0x50(%rsp)
  130fea:	mov    %r12,0x10(%rsp)
  130fef:	mov    %rcx,0x28(%rsp)
  130ff4:	mov    (%rsp),%r8
  130ff8:	mov    0x8(%rsp),%rdi
  130ffd:	mov    0x78(%rsp),%rbp
  131002:	mov    0xc0(%rsp),%rsi
  13100a:	inc    %rdi
  13100d:	add    $0x8,%r14
  131011:	movslq -0x4(%r10,%rdi,4),%r11
  131016:	cmp    0x0(%rbp,%r11,4),%eax
  13101b:	jle    1310dd <PRIVATE000000000021a941+0x56d>
  131021:	mov    0x58(%rsp),%rcx
  131026:	movsd  -0x8(%r14),%xmm2
  13102c:	divsd  (%rcx,%r11,8),%xmm2
  131032:	movaps %xmm2,%xmm3
  131035:	andps  0x292d5b4(%rip),%xmm3        # 2a5e5f0 <_fini+0x19ec>
  13103c:	comisd %xmm1,%xmm3
  131040:	jbe    131125 <PRIVATE000000000021a941+0x5b5>
  131046:	mov    0x48(%rsp),%rcx
  13104b:	mov    0x50(%rsp),%r13
  131050:	mov    0x40(%rsp),%r12
  131055:	mov    0x70(%rsp),%r15
  13105a:	mov    %r11d,(%rcx,%rbx,4)
  13105e:	mov    0x0(%r13,%r11,8),%rcx
  131063:	movsd  %xmm2,(%r12,%rbx,8)
  131069:	inc    %rbx
  13106c:	mov    (%r15,%r11,8),%r11
  131070:	cmp    %r11,%rcx
  131073:	jge    1310d3 <PRIVATE000000000021a941+0x563>
  131075:	mov    0x68(%rsp),%r13
  13107a:	xor    %r12d,%r12d
  13107d:	mov    0x60(%rsp),%r15
  131082:	sub    %rcx,%r11
  131085:	lea    0x0(%r13,%rcx,4),%r13
  13108a:	lea    (%r15,%rcx,8),%r15
  13108e:	movsd  (%r15,%r12,8),%xmm3
  131094:	mulsd  %xmm2,%xmm3
  131098:	movslq 0x0(%r13,%r12,4),%rcx
  13109d:	xorps  0x292d55c(%rip),%xmm3        # 2a5e600 <_fini+0x19fc>
  1310a4:	cmpl   $0x0,(%r9,%rcx,4)
  1310a9:	addsd  (%rsi,%rcx,8),%xmm3
  1310ae:	movsd  %xmm3,(%rsi,%rcx,8)
  1310b3:	jne    1310cb <PRIVATE000000000021a941+0x55b>
  1310b5:	movl   $0x1,(%r9,%rcx,4)
  1310bd:	inc    %edx
  1310bf:	mov    0x0(%rbp,%rcx,4),%ecx
  1310c3:	mov    %ecx,-0x4(%r8)
  1310c7:	add    $0xfffffffffffffffc,%r8
  1310cb:	inc    %r12
  1310ce:	cmp    %r11,%r12
  1310d1:	jb     13108e <PRIVATE000000000021a941+0x51e>
  1310d3:	mov    0x28(%rsp),%rcx
  1310d8:	mov    (%rcx),%r15d
  1310db:	jmp    131125 <PRIVATE000000000021a941+0x5b5>
  1310dd:	movsd  (%rsi,%r11,8),%xmm2
  1310e3:	cmpl   $0x0,(%r9,%r11,4)
  1310e8:	addsd  -0x8(%r14),%xmm2
  1310ee:	movsd  %xmm2,(%rsi,%r11,8)
  1310f4:	jne    131125 <PRIVATE000000000021a941+0x5b5>
  1310f6:	movslq %edx,%rcx
  1310f9:	add    $0xfffffffffffffffc,%r8
  1310fd:	shl    $0x2,%rcx
  131101:	inc    %edx
  131103:	neg    %rcx
  131106:	add    0x20(%rsp),%rcx
  13110b:	movl   $0x1,(%r9,%r11,4)
  131113:	mov    0x0(%rbp,%r11,4),%r11d
  131118:	mov    0x28(%rsp),%r12
  13111d:	mov    %r11d,-0x4(%rcx)
  131121:	mov    (%r12),%r15d
  131125:	movslq %r15d,%r15
  131128:	cmp    %r15,%rdi
  13112b:	jl     13100a <PRIVATE000000000021a941+0x49a>
  131131:	mov    %edx,%r10d
  131134:	mov    $0x1,%r14d
  13113a:	neg    %r10d
  13113d:	mov    $0xffffffffffffffff,%rax
  131144:	mov    0x20(%rsp),%r11
  131149:	mov    0x58(%rsp),%rbp
  13114e:	mov    0x30(%rsp),%rdi
  131153:	mov    0x50(%rsp),%r13
  131158:	mov    0x10(%rsp),%r12
  13115d:	mov    0x38(%rsp),%esi
  131161:	mov    0x18(%rsp),%r8
  131166:	movslq %r10d,%rcx
  131169:	cmp    $0x1,%edx
  13116c:	jle    1311e2 <PRIVATE000000000021a941+0x672>
  13116e:	mov    %esi,0x38(%rsp)
  131172:	mov    %r8,0x18(%rsp)
  131177:	mov    %r9,0x20(%rsp)
  13117c:	mov    -0x4(%r11,%rax,4),%r15d
  131181:	mov    %r14d,%r9d
  131184:	test   %r14d,%r14d
  131187:	jle    1311be <PRIVATE000000000021a941+0x64e>
  131189:	lea    -0x1(%r9),%r8d
  13118d:	shr    $0x1f,%r8d
  131191:	lea    -0x1(%r9,%r8,1),%r8d
  131196:	sar    %r8d
  131199:	mov    %r8d,%esi
  13119c:	neg    %esi
  13119e:	movslq %esi,%rsi
  1311a1:	mov    -0x4(%r11,%rsi,4),%esi
  1311a6:	cmp    %esi,%r15d
  1311a9:	jg     1311be <PRIVATE000000000021a941+0x64e>
  1311ab:	neg    %r9d
  1311ae:	movslq %r9d,%r9
  1311b1:	mov    %esi,-0x4(%r11,%r9,4)
  1311b6:	mov    %r8d,%r9d
  1311b9:	test   %r8d,%r8d
  1311bc:	jg     131189 <PRIVATE000000000021a941+0x619>
  1311be:	neg    %r9d
  1311c1:	inc    %r14d
  1311c4:	movslq %r9d,%r9
  1311c7:	dec    %rax
  1311ca:	mov    %r15d,-0x4(%r11,%r9,4)
  1311cf:	cmp    %edx,%r14d
  1311d2:	jl     13117c <PRIVATE000000000021a941+0x60c>
  1311d4:	mov    0x38(%rsp),%esi
  1311d8:	mov    0x18(%rsp),%r8
  1311dd:	mov    0x20(%rsp),%r9
  1311e2:	test   %edx,%edx
  1311e4:	jle    131664 <PRIVATE000000000021a941+0xaf4>
  1311ea:	movsd  0x292d446(%rip),%xmm3        # 2a5e638 <_fini+0x1a34>
  1311f2:	movsd  0x292d456(%rip),%xmm2        # 2a5e650 <_fini+0x1a4c>
  1311fa:	mov    %r12,0x10(%rsp)
  1311ff:	mov    %r8,0x18(%rsp)
  131204:	mov    0xc0(%rsp),%rax
  13120c:	pxor   %xmm5,%xmm5
  131210:	mov    %esi,%r8d
  131213:	cvtsi2sd %edx,%xmm5
  131217:	mov    -0x4(%r11),%r14d
  13121b:	sub    %r14d,%r8d
  13121e:	pxor   %xmm4,%xmm4
  131222:	cvtsi2sd %r8d,%xmm4
  131227:	mulsd  %xmm2,%xmm5
  13122b:	comisd %xmm5,%xmm4
  13122f:	jbe    13165a <PRIVATE000000000021a941+0xaea>
  131235:	inc    %rcx
  131238:	dec    %edx
  13123a:	xor    %r8d,%r8d
  13123d:	inc    %r10d
  131240:	mov    $0x1,%r15d
  131246:	mov    -0x4(%r11,%rcx,4),%r12d
  13124b:	mov    %r12d,-0x4(%r11)
  13124f:	cmp    $0x1,%edx
  131252:	jle    1312e0 <PRIVATE000000000021a941+0x770>
  131258:	mov    %rbp,0x58(%rsp)
  13125d:	mov    %rdi,0x30(%rsp)
  131262:	mov    %r13,0x50(%rsp)
  131267:	mov    %esi,0x38(%rsp)
  13126b:	mov    %r9,0x20(%rsp)
  131270:	mov    %r15d,%ebp
  131273:	lea    0x1(%r15),%r9d
  131277:	neg    %ebp
  131279:	movslq %ebp,%rbp
  13127c:	cmp    %edx,%r9d
  13127f:	jge    1312a6 <PRIVATE000000000021a941+0x736>
  131281:	movslq %r15d,%rsi
  131284:	mov    %r9d,%r13d
  131287:	shl    $0x2,%rsi
  13128b:	neg    %r13d
  13128e:	neg    %rsi
  131291:	mov    -0x4(%r11,%rbp,4),%edi
  131296:	movslq %r13d,%r13
  131299:	cmp    -0x8(%rsi,%r11,1),%edi
  13129e:	cmovg  %r9d,%r15d
  1312a2:	cmovg  %r13,%rbp
  1312a6:	mov    -0x4(%r11,%rbp,4),%ebp
  1312ab:	cmp    %r12d,%ebp
  1312ae:	jg     1312c8 <PRIVATE000000000021a941+0x758>
  1312b0:	neg    %r8d
  1312b3:	movslq %r8d,%r8
  1312b6:	mov    %ebp,-0x4(%r11,%r8,4)
  1312bb:	mov    %r15d,%r8d
  1312be:	lea    0x1(%r15,%r15,1),%r15d
  1312c3:	cmp    %edx,%r15d
  1312c6:	jl     131270 <PRIVATE000000000021a941+0x700>
  1312c8:	mov    0x58(%rsp),%rbp
  1312cd:	mov    0x30(%rsp),%rdi
  1312d2:	mov    0x50(%rsp),%r13
  1312d7:	mov    0x38(%rsp),%esi
  1312db:	mov    0x20(%rsp),%r9
  1312e0:	neg    %r8d
  1312e3:	movslq %r8d,%r8
  1312e6:	movslq %r14d,%r14
  1312e9:	mov    %r12d,-0x4(%r11,%r8,4)
  1312ee:	movslq (%rdi,%r14,4),%r14
  1312f2:	movsd  (%rax,%r14,8),%xmm4
  1312f8:	divsd  0x0(%rbp,%r14,8),%xmm4
  1312ff:	movaps %xmm4,%xmm5
  131302:	andps  0x292d2e7(%rip),%xmm5        # 2a5e5f0 <_fini+0x19ec>
  131309:	comisd %xmm1,%xmm5
  13130d:	jbe    131427 <PRIVATE000000000021a941+0x8b7>
  131313:	mov    0x70(%rsp),%r12
  131318:	pxor   %xmm5,%xmm5
  13131c:	mov    0x0(%r13,%r14,8),%r8
  131321:	mov    (%r12,%r14,8),%r12
  131325:	mov    %r12,%r15
  131328:	sub    %r8,%r15
  13132b:	cvtsi2sd %r15,%xmm5
  131330:	mov    0x48(%rsp),%r15
  131335:	mulsd  %xmm3,%xmm5
  131339:	mov    %r14d,(%r15,%rbx,4)
  13133d:	addsd  %xmm5,%xmm0
  131341:	mov    0x40(%rsp),%r15
  131346:	movsd  %xmm4,(%r15,%rbx,8)
  13134c:	inc    %rbx
  13134f:	cmp    %r12,%r8
  131352:	jge    131427 <PRIVATE000000000021a941+0x8b7>
  131358:	mov    %r14,(%rsp)
  13135c:	mov    %r13,0x50(%rsp)
  131361:	mov    %rbx,0x8(%rsp)
  131366:	mov    %rbp,0x58(%rsp)
  13136b:	mov    %rdi,0x30(%rsp)
  131370:	mov    %esi,0x38(%rsp)
  131374:	mov    0x60(%rsp),%r14
  131379:	mov    0x68(%rsp),%r15
  13137e:	mov    0x78(%rsp),%r13
  131383:	movsd  (%r14,%r8,8),%xmm5
  131389:	mulsd  %xmm4,%xmm5
  13138d:	movslq (%r15,%r8,4),%rbx
  131391:	movsd  (%rax,%rbx,8),%xmm6
  131396:	cmpl   $0x0,(%r9,%rbx,4)
  13139b:	subsd  %xmm5,%xmm6
  13139f:	movsd  %xmm6,(%rax,%rbx,8)
  1313a4:	jne    1313ff <PRIVATE000000000021a941+0x88f>
  1313a6:	movslq %r10d,%r10
  1313a9:	mov    %edx,%esi
  1313ab:	movl   $0x1,(%r9,%rbx,4)
  1313b3:	dec    %rcx
  1313b6:	mov    0x0(%r13,%rbx,4),%edi
  1313bb:	inc    %edx
  1313bd:	mov    %edi,-0x4(%r11,%r10,4)
  1313c2:	dec    %r10d
  1313c5:	test   %esi,%esi
  1313c7:	jle    1313f5 <PRIVATE000000000021a941+0x885>
  1313c9:	lea    -0x1(%rsi),%ebp
  1313cc:	shr    $0x1f,%ebp
  1313cf:	lea    -0x1(%rsi,%rbp,1),%ebp
  1313d3:	sar    %ebp
  1313d5:	mov    %ebp,%ebx
  1313d7:	neg    %ebx
  1313d9:	movslq %ebx,%rbx
  1313dc:	mov    -0x4(%r11,%rbx,4),%ebx
  1313e1:	cmp    %ebx,%edi
  1313e3:	jg     1313f5 <PRIVATE000000000021a941+0x885>
  1313e5:	neg    %esi
  1313e7:	movslq %esi,%rsi
  1313ea:	mov    %ebx,-0x4(%r11,%rsi,4)
  1313ef:	mov    %ebp,%esi
  1313f1:	test   %ebp,%ebp
  1313f3:	jg     1313c9 <PRIVATE000000000021a941+0x859>
  1313f5:	neg    %esi
  1313f7:	movslq %esi,%rsi
  1313fa:	mov    %edi,-0x4(%r11,%rsi,4)
  1313ff:	inc    %r8
  131402:	cmp    %r12,%r8
  131405:	jl     131383 <PRIVATE000000000021a941+0x813>
  13140b:	mov    (%rsp),%r14
  13140f:	mov    0x8(%rsp),%rbx
  131414:	mov    0x58(%rsp),%rbp
  131419:	mov    0x30(%rsp),%rdi
  13141e:	mov    0x50(%rsp),%r13
  131423:	mov    0x38(%rsp),%esi
  131427:	movl   $0x0,(%r9,%r14,4)
  13142f:	movq   $0x0,(%rax,%r14,8)
  131437:	test   %edx,%edx
  131439:	jg     13120c <PRIVATE000000000021a941+0x69c>
  13143f:	mov    0x10(%rsp),%r12
  131444:	mov    0x18(%rsp),%r8
  131449:	mov    %esi,%ecx
  13144b:	movslq %ecx,%rax
  13144e:	xor    %edx,%edx
  131450:	cmp    %esi,%ecx
  131452:	jge    1315cc <PRIVATE000000000021a941+0xa5c>
  131458:	mov    %r8,0x18(%rsp)
  13145d:	sub    %rax,0x80(%rsp)
  131465:	lea    (%rdi,%rax,4),%rax
  131469:	mov    0x60(%rsp),%rdi
  13146e:	mov    0x68(%rsp),%r8
  131473:	movsd  0x292d1bd(%rip),%xmm2        # 2a5e638 <_fini+0x1a34>
  13147b:	mov    %r13,0x50(%rsp)
  131480:	mov    %r12,0x10(%rsp)
  131485:	mov    0xc0(%rsp),%rsi
  13148d:	movslq (%rax,%rdx,4),%r9
  131491:	movsd  (%rsi,%r9,8),%xmm3
  131497:	divsd  0x0(%rbp,%r9,8),%xmm3
  13149e:	movaps %xmm3,%xmm4
  1314a1:	andps  0x292d148(%rip),%xmm4        # 2a5e5f0 <_fini+0x19ec>
  1314a8:	comisd %xmm1,%xmm4
  1314ac:	jbe    1315a9 <PRIVATE000000000021a941+0xa39>
  1314b2:	mov    0x70(%rsp),%r11
  1314b7:	pxor   %xmm4,%xmm4
  1314bb:	mov    0x50(%rsp),%r10
  1314c0:	mov    0x48(%rsp),%rcx
  1314c5:	mov    (%r11,%r9,8),%r13
  1314c9:	mov    %r13,%r11
  1314cc:	mov    (%r10,%r9,8),%r10
  1314d0:	sub    %r10,%r11
  1314d3:	cvtsi2sd %r11,%xmm4
  1314d8:	mulsd  %xmm2,%xmm4
  1314dc:	mov    0x40(%rsp),%r12
  1314e1:	addsd  %xmm4,%xmm0
  1314e5:	mov    %r9d,(%rcx,%rbx,4)
  1314e9:	movsd  %xmm3,(%r12,%rbx,8)
  1314ef:	inc    %rbx
  1314f2:	cmp    %r13,%r10
  1314f5:	jge    1315a9 <PRIVATE000000000021a941+0xa39>
  1314fb:	mov    %r11,%r12
  1314fe:	xor    %ecx,%ecx
  131500:	mov    $0x1,%r14d
  131506:	xor    %r13d,%r13d
  131509:	shr    %r12
  13150c:	je     13157b <PRIVATE000000000021a941+0xa0b>
  13150e:	lea    (%r8,%r10,4),%r15
  131512:	lea    (%rdi,%r10,8),%r14
  131516:	nopl   (%rax)
  131519:	nopl   0x0(%rax)
  131520:	movsd  0x0(%r13,%r14,1),%xmm4
  131527:	mulsd  %xmm3,%xmm4
  13152b:	movslq (%r15,%rcx,8),%r8
  13152f:	xorps  0x292d0ca(%rip),%xmm4        # 2a5e600 <_fini+0x19fc>
  131536:	addsd  (%rsi,%r8,8),%xmm4
  13153c:	movsd  %xmm4,(%rsi,%r8,8)
  131542:	movsd  0x8(%r13,%r14,1),%xmm5
  131549:	add    $0x10,%r13
  13154d:	mulsd  %xmm3,%xmm5
  131551:	movslq 0x4(%r15,%rcx,8),%r8
  131556:	inc    %rcx
  131559:	xorps  0x292d0a0(%rip),%xmm5        # 2a5e600 <_fini+0x19fc>
  131560:	addsd  (%rsi,%r8,8),%xmm5
  131566:	movsd  %xmm5,(%rsi,%r8,8)
  13156c:	cmp    %r12,%rcx
  13156f:	jb     131520 <PRIVATE000000000021a941+0x9b0>
  131571:	mov    0x68(%rsp),%r8
  131576:	lea    0x1(%rcx,%rcx,1),%r14
  13157b:	lea    -0x1(%r14),%rcx
  13157f:	cmp    %r11,%rcx
  131582:	jae    1315a9 <PRIVATE000000000021a941+0xa39>
  131584:	lea    -0x1(%r10,%r14,1),%rcx
  131589:	movsd  (%rdi,%rcx,8),%xmm4
  13158e:	mulsd  %xmm4,%xmm3
  131592:	movslq (%r8,%rcx,4),%r10
  131596:	xorps  0x292d063(%rip),%xmm3        # 2a5e600 <_fini+0x19fc>
  13159d:	addsd  (%rsi,%r10,8),%xmm3
  1315a3:	movsd  %xmm3,(%rsi,%r10,8)
  1315a9:	inc    %rdx
  1315ac:	movq   $0x0,(%rsi,%r9,8)
  1315b4:	cmp    0x80(%rsp),%rdx
  1315bc:	jb     13148d <PRIVATE000000000021a941+0x91d>
  1315c2:	mov    0x10(%rsp),%r12
  1315c7:	mov    0x18(%rsp),%r8
  1315cc:	addsd  0x200(%r12),%xmm0
  1315d6:	mov    %ebx,(%r8)
  1315d9:	movsd  %xmm0,0x200(%r12)
  1315e3:	add    $0x88,%rsp
  1315ea:	pop    %rbp
  1315eb:	pop    %rbx
  1315ec:	pop    %r15
  1315ee:	pop    %r14
  1315f0:	pop    %r13
  1315f2:	pop    %r12
  1315f4:	ret    
  1315f5:	movl   $0x0,(%r8)
  1315fc:	add    $0x88,%rsp
  131603:	pop    %rbp
  131604:	pop    %rbx
  131605:	pop    %r15
  131607:	pop    %r14
  131609:	pop    %r13
  13160b:	pop    %r12
  13160d:	ret    
  13160e:	mov    0xc0(%rsp),%r8
  131616:	mov    0x78(%rsp),%r9
  13161b:	movsd  (%r8,%rdi,8),%xmm0
  131621:	cmp    (%r9,%rdi,4),%edx
  131625:	addsd  %xmm5,%xmm0
  131629:	cmovg  (%r9,%rdi,4),%edx
  13162e:	movsd  %xmm0,(%r8,%rdi,8)
  131634:	jmp    130df9 <PRIVATE000000000021a941+0x289>
  131639:	mov    0x48(%rsp),%r8
  13163e:	movsd  %xmm0,(%rsi)
  131642:	add    $0x8,%rsi
  131646:	mov    %edi,(%r8,%rbx,4)
  13164a:	inc    %rbx
  13164d:	mov    0x28(%rsp),%rdi
  131652:	mov    (%rdi),%r15d
  131655:	jmp    130df9 <PRIVATE000000000021a941+0x289>
  13165a:	mov    0x10(%rsp),%r12
  13165f:	mov    0x18(%rsp),%r8
  131664:	test   %edx,%edx
  131666:	jle    131449 <PRIVATE000000000021a941+0x8d9>
  13166c:	neg    %edx
  13166e:	inc    %edx
  131670:	test   %edx,%edx
  131672:	jg     1316fd <PRIVATE000000000021a941+0xb8d>
  131678:	mov    %edx,%r10d
  13167b:	mov    $0x1,%eax
  131680:	neg    %r10d
  131683:	xor    %r14d,%r14d
  131686:	inc    %r10d
  131689:	mov    %r10d,(%rsp)
  13168d:	shr    %r10d
  131690:	je     1316ce <PRIVATE000000000021a941+0xb5e>
  131692:	movslq %edx,%rcx
  131695:	add    0x80(%rsp),%rcx
  13169d:	mov    0x48(%rsp),%rax
  1316a2:	lea    (%rax,%rcx,4),%rax
  1316a6:	xor    %ecx,%ecx
  1316a8:	movslq -0x4(%rax,%r14,8),%r15
  1316ad:	movslq (%rdi,%r15,4),%r15
  1316b1:	mov    %ecx,(%r9,%r15,4)
  1316b5:	movslq (%rax,%r14,8),%r15
  1316b9:	inc    %r14
  1316bc:	movslq (%rdi,%r15,4),%r15
  1316c0:	mov    %ecx,(%r9,%r15,4)
  1316c4:	cmp    %r10,%r14
  1316c7:	jb     1316a8 <PRIVATE000000000021a941+0xb38>
  1316c9:	lea    0x1(%r14,%r14,1),%eax
  1316ce:	lea    -0x1(%rax),%ecx
  1316d1:	cmp    (%rsp),%ecx
  1316d4:	jae    1316fd <PRIVATE000000000021a941+0xb8d>
  1316d6:	movslq %edx,%rdx
  1316d9:	movslq %eax,%rax
  1316dc:	add    %rax,%rdx
  1316df:	add    0x80(%rsp),%rdx
  1316e7:	mov    0x48(%rsp),%rcx
  1316ec:	movslq -0x8(%rcx,%rdx,4),%r10
  1316f1:	movslq (%rdi,%r10,4),%r14
  1316f5:	movl   $0x0,(%r9,%r14,4)
  1316fd:	mov    -0x4(%r11),%ecx
  131701:	jmp    13144b <PRIVATE000000000021a941+0x8db>
