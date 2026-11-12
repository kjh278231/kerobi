00000000001c0070 <PRIVATE0000000000134a87>:
  1c0070:	push   %r12
  1c0072:	push   %r13
  1c0074:	push   %r14
  1c0076:	push   %r15
  1c0078:	push   %rbx
  1c0079:	push   %rbp
  1c007a:	sub    $0x20a8,%rsp
  1c0081:	mov    %rdi,%r14
  1c0084:	mov    0x20e8(%rsp),%rax
  1c008c:	movsd  0x28a130c(%rip),%xmm4        # 2a613a0 <__STRLITPACK_7+0xa74>
  1c0094:	pxor   %xmm6,%xmm6
  1c0098:	mov    %edx,0x1fa0(%rsp)
  1c009f:	mov    0x68(%r14),%ebp
  1c00a3:	mov    %ebp,0x2060(%rsp)
  1c00aa:	mov    0x168(%r14),%rbp
  1c00b1:	mov    %rbp,0x1fc0(%rsp)
  1c00b9:	mov    0x280(%r14),%rbp
  1c00c0:	mov    %rbp,0x2018(%rsp)
  1c00c8:	mov    0x1e8(%r14),%rbp
  1c00cf:	mov    %rbp,0x2010(%rsp)
  1c00d7:	mov    0x220(%r14),%rbp
  1c00de:	mov    0x70(%r14),%r12
  1c00e2:	mov    %rbp,0x2008(%rsp)
  1c00ea:	mov    0x228(%r14),%rbp
  1c00f1:	mov    %r12,0x2040(%rsp)
  1c00f9:	mov    %rbp,0x2000(%rsp)
  1c0101:	mov    0x208(%r14),%r12
  1c0108:	mov    0x210(%r14),%rbp
  1c010f:	mov    %rbp,0x2090(%rsp)
  1c0117:	mov    0x218(%r14),%rbp
  1c011e:	mov    0x470(%r14),%rbx
  1c0125:	mov    %rbp,0x1ff8(%rsp)
  1c012d:	mov    0x8(%r12),%rbp
  1c0132:	mov    %rbp,0x1fb8(%rsp)
  1c013a:	mov    0x1e0(%r14),%rbp
  1c0141:	mov    %rbx,0x2048(%rsp)
  1c0149:	movsd  0x380(%r14),%xmm3
  1c0152:	mov    0x18(%rbx),%rbx
  1c0156:	mov    0x78(%r14),%r13
  1c015a:	mov    0x4b8(%r14),%r10
  1c0161:	mov    %rbp,0x1ed8(%rsp)
  1c0169:	mov    0x10(%r12),%rbp
  1c016e:	mulsd  %xmm3,%xmm4
  1c0172:	mov    0x80(%r14),%rdx
  1c0179:	mov    %rbx,0x1fe8(%rsp)
  1c0181:	mov    %r13,0x2038(%rsp)
  1c0189:	xor    %r13d,%r13d
  1c018c:	mov    0x178(%r14),%ebx
  1c0193:	mov    %rbp,0x1ff0(%rsp)
  1c019b:	mov    0x64(%r14),%r11d
  1c019f:	mov    %rdx,0x2030(%rsp)
  1c01a7:	xor    %edx,%edx
  1c01a9:	mov    0x200(%r14),%rbp
  1c01b0:	mov    %ebx,0x1fe0(%rsp)
  1c01b7:	xor    %ebx,%ebx
  1c01b9:	mov    %edx,(%rax)
  1c01bb:	xor    %r12d,%r12d
  1c01be:	mov    %r9d,0x2050(%rsp)
  1c01c6:	mov    %r8,0x2058(%rsp)
  1c01ce:	mov    %rcx,0x1fa8(%rsp)
  1c01d6:	mov    0x100(%r10),%r15
  1c01dd:	mov    %r15,%rdi
  1c01e0:	mov    %r11d,0x2068(%rsp)
  1c01e8:	mov    %rbp,0x1fd0(%rsp)
  1c01f0:	mov    0x88(%r14),%rcx
  1c01f7:	mov    0x98(%r14),%r8
  1c01fe:	mov    0xa8(%r14),%r9
  1c0205:	mov    0xb0(%r14),%r10
  1c020c:	mov    0xa0(%r14),%r11
  1c0213:	mov    0x64(%r14),%eax
  1c0217:	mov    0x1f8(%r14),%rbp
  1c021e:	movsd  0x370(%r14),%xmm1
  1c0227:	movsd  0x398(%r14),%xmm2
  1c0230:	movsd  0x3b0(%r14),%xmm5
  1c0239:	movsd  %xmm0,0x1ee0(%rsp)
  1c0242:	mov    %esi,0x1f98(%rsp)
  1c0249:	mov    %rcx,0x2028(%rsp)
  1c0251:	mov    %r8,0x2020(%rsp)
  1c0259:	mov    %r9,0x1f90(%rsp)
  1c0261:	mov    %r10,0x1f88(%rsp)
  1c0269:	mov    %r11,0x1ed0(%rsp)
  1c0271:	mov    %rbp,0x1ec8(%rsp)
  1c0279:	xor    %ebp,%ebp
  1c027b:	movsd  %xmm1,0x1fc8(%rsp)
  1c0284:	movsd  %xmm3,0x2070(%rsp)
  1c028d:	movsd  %xmm2,0x2078(%rsp)
  1c0296:	movsd  %xmm4,0x2080(%rsp)
  1c029f:	movsd  %xmm5,0x2088(%rsp)
  1c02a8:	movl   $0x32,0x1fd8(%rsp)
  1c02b3:	movb   $0x0,0x1ec0(%rsp)
  1c02bb:	mov    %edx,0x1fb0(%rsp)
  1c02c2:	mov    %edx,0xc(%rsp)
  1c02c6:	mov    %r13,0x10(%rsp)
  1c02cb:	mov    %r13,0x18(%rsp)
  1c02d0:	mov    %r13,0x20(%rsp)
  1c02d5:	mov    %eax,0x8(%rsp)
  1c02d9:	mov    %edx,0x28(%rsp)
  1c02dd:	mov    %edx,0x2c(%rsp)
  1c02e1:	mov    %edx,0x40(%rsp)
  1c02e5:	movsd  %xmm6,0x38(%rsp)
  1c02eb:	mov    %edx,0x30(%rsp)
  1c02ef:	mov    %edx,0x1850(%rsp)
  1c02f6:	mov    %edx,0x184c(%rsp)
  1c02fd:	mov    %edx,0x1848(%rsp)
  1c0304:	movl   $0x1,0x1864(%rsp)
  1c030f:	call   8d33f0 <PRIVATE0000000000ac7d4e>
  1c0314:	mov    %eax,%edx
  1c0316:	mov    $0x40,%eax
  1c031b:	cmp    $0x40,%edx
  1c031e:	pxor   %xmm0,%xmm0
  1c0322:	cmovg  %eax,%edx
  1c0325:	mov    %edx,0x1864(%rsp)
  1c032c:	mov    0x8(%rsp),%edx
  1c0330:	lea    (%rdx,%rdx,4),%ecx
  1c0333:	add    %ecx,%ecx
  1c0335:	cvtsi2sd %ecx,%xmm0
  1c0339:	call   bc460 <log@plt>
  1c033e:	mulsd  0x28a0f9a(%rip),%xmm0        # 2a612e0 <__STRLITPACK_7+0x9b4>
  1c0346:	cvttsd2si %xmm0,%eax
  1c034a:	mov    0x1864(%rsp),%edx
  1c0351:	cmp    %edx,%eax
  1c0353:	cmovl  %eax,%edx
  1c0356:	cmp    $0x1,%edx
  1c0359:	jg     1c0368 <PRIVATE0000000000134a87+0x2f8>
  1c035b:	movl   $0x0,0x1848(%rsp)
  1c0366:	jmp    1c036f <PRIVATE0000000000134a87+0x2ff>
  1c0368:	mov    %edx,0x1848(%rsp)
  1c036f:	xor    %edx,%edx
  1c0371:	xor    %cl,%cl
  1c0373:	xor    %eax,%eax
  1c0375:	xor    %r8d,%r8d
  1c0378:	xor    %r9d,%r9d
  1c037b:	nopl   0x0(%rax,%rax,1)
  1c0380:	inc    %cl
  1c0382:	mov    %r8d,0x48(%rsp,%rdx,1)
  1c0387:	mov    %r9,0x50(%rsp,%rdx,1)
  1c038c:	mov    %r9,0x58(%rsp,%rdx,1)
  1c0391:	mov    %r8d,0x60(%rsp,%rdx,1)
  1c0396:	mov    %r9,0x68(%rsp,%rdx,1)
  1c039b:	mov    %r9,0x70(%rsp,%rdx,1)
  1c03a0:	mov    %r8d,0x78(%rsp,%rdx,1)
  1c03a5:	mov    %r9,0x80(%rsp,%rdx,1)
  1c03ad:	mov    %r9,0x88(%rsp,%rdx,1)
  1c03b5:	add    $0x60,%rdx
  1c03b9:	mov    %r9,0x1870(%rsp,%rax,1)
  1c03c1:	mov    %r9,0x1878(%rsp,%rax,1)
  1c03c9:	mov    %r9,0x1880(%rsp,%rax,1)
  1c03d1:	add    $0x18,%rax
  1c03d5:	cmp    $0x40,%cl
  1c03d8:	jl     1c0380 <PRIVATE0000000000134a87+0x310>
  1c03da:	xor    %eax,%eax
  1c03dc:	mov    %rax,0x1858(%rsp)
  1c03e4:	xor    %edx,%edx
  1c03e6:	mov    %rax,0x1868(%rsp)
  1c03ee:	cmpl   $0x0,0x1848(%rsp)
  1c03f6:	jle    1c0548 <PRIVATE0000000000134a87+0x4d8>
  1c03fc:	mov    %r13,0x1f60(%rsp)
  1c0404:	mov    %rax,%r13
  1c0407:	mov    %r14,0x1f68(%rsp)
  1c040f:	mov    %rdx,%r14
  1c0412:	movslq 0x8(%rsp),%rcx
  1c0417:	test   %rcx,%rcx
  1c041a:	jl     1c52e4 <PRIVATE0000000000134a87+0x5274>
  1c0420:	mov    %r15,%rdi
  1c0423:	lea    0x4(,%rcx,4),%rsi
  1c042b:	call   8d1880 <PRIVATE0000000000ac645d>
  1c0430:	mov    %rax,0x50(%rsp,%r13,1)
  1c0435:	test   %rax,%rax
  1c0438:	je     1c528b <PRIVATE0000000000134a87+0x521b>
  1c043e:	movslq 0x8(%rsp),%rsi
  1c0443:	test   %rsi,%rsi
  1c0446:	jl     1c52d6 <PRIVATE0000000000134a87+0x5266>
  1c044c:	inc    %rsi
  1c044f:	mov    %r15,%rdi
  1c0452:	mov    $0x8,%edx
  1c0457:	call   8d1bb0 <PRIVATE0000000000ac64e6>
  1c045c:	mov    %rax,0x58(%rsp,%r13,1)
  1c0461:	test   %rax,%rax
  1c0464:	je     1c528b <PRIVATE0000000000134a87+0x521b>
  1c046a:	movslq 0x8(%rsp),%rcx
  1c046f:	test   %rcx,%rcx
  1c0472:	jl     1c52c8 <PRIVATE0000000000134a87+0x5258>
  1c0478:	mov    %r15,%rdi
  1c047b:	lea    0x4(,%rcx,4),%rsi
  1c0483:	call   8d1880 <PRIVATE0000000000ac645d>
  1c0488:	mov    %rax,0x68(%rsp,%r13,1)
  1c048d:	test   %rax,%rax
  1c0490:	je     1c528b <PRIVATE0000000000134a87+0x521b>
  1c0496:	movslq 0x8(%rsp),%rsi
  1c049b:	test   %rsi,%rsi
  1c049e:	jl     1c52ba <PRIVATE0000000000134a87+0x524a>
  1c04a4:	inc    %rsi
  1c04a7:	mov    %r15,%rdi
  1c04aa:	mov    $0x8,%edx
  1c04af:	call   8d1bb0 <PRIVATE0000000000ac64e6>
  1c04b4:	mov    %rax,0x70(%rsp,%r13,1)
  1c04b9:	test   %rax,%rax
  1c04bc:	je     1c528b <PRIVATE0000000000134a87+0x521b>
  1c04c2:	movslq 0x8(%rsp),%rcx
... 3836 more lines (narrow --range/--grep or raise --max)
