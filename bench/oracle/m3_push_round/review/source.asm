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
  1c04c7:	test   %rcx,%rcx
  1c04ca:	jl     1c52a9 <PRIVATE0000000000134a87+0x5239>
  1c04d0:	mov    %r15,%rdi
  1c04d3:	lea    0x4(,%rcx,4),%rsi
  1c04db:	call   8d1880 <PRIVATE0000000000ac645d>
  1c04e0:	mov    %rax,0x80(%rsp,%r13,1)
  1c04e8:	test   %rax,%rax
  1c04eb:	je     1c528b <PRIVATE0000000000134a87+0x521b>
  1c04f1:	movslq 0x8(%rsp),%rsi
  1c04f6:	test   %rsi,%rsi
  1c04f9:	jl     1c5298 <PRIVATE0000000000134a87+0x5228>
  1c04ff:	inc    %rsi
  1c0502:	mov    %r15,%rdi
  1c0505:	mov    $0x8,%edx
  1c050a:	call   8d1bb0 <PRIVATE0000000000ac64e6>
  1c050f:	mov    %rax,0x88(%rsp,%r13,1)
  1c0517:	test   %rax,%rax
  1c051a:	je     1c528b <PRIVATE0000000000134a87+0x521b>
  1c0520:	inc    %r14
  1c0523:	add    $0x60,%r13
  1c0527:	movslq 0x1848(%rsp),%rcx
  1c052f:	cmp    %rcx,%r14
  1c0532:	jl     1c0412 <PRIVATE0000000000134a87+0x3a2>
  1c0538:	mov    0x1f60(%rsp),%r13
  1c0540:	mov    0x1f68(%rsp),%r14
  1c0548:	movslq 0x8(%rsp),%rsi
  1c054d:	test   %rsi,%rsi
  1c0550:	jle    1c527a <PRIVATE0000000000134a87+0x520a>
  1c0556:	mov    %r15,%rdi
  1c0559:	mov    $0x4,%edx
  1c055e:	call   8d1bb0 <PRIVATE0000000000ac64e6>
  1c0563:	mov    %rax,0x1858(%rsp)
  1c056b:	test   %rax,%rax
  1c056e:	je     1c520d <PRIVATE0000000000134a87+0x519d>
  1c0574:	cmpl   $0x1,0x1864(%rsp)
  1c057c:	jle    1c0666 <PRIVATE0000000000134a87+0x5f6>
  1c0582:	xor    %edx,%edx
  1c0584:	mov    %rbx,0x1f18(%rsp)
  1c058c:	mov    %rdx,%rbx
  1c058f:	mov    %r13,0x1f60(%rsp)
  1c0597:	mov    %rdx,%r13
  1c059a:	movslq 0x8(%rsp),%rsi
  1c059f:	test   %rsi,%rsi
  1c05a2:	jle    1c5258 <PRIVATE0000000000134a87+0x51e8>
  1c05a8:	mov    %r15,%rdi
  1c05ab:	mov    $0x8,%edx
  1c05b0:	call   8d1bb0 <PRIVATE0000000000ac64e6>
  1c05b5:	mov    %rax,0x1870(%rsp,%r13,1)
  1c05bd:	test   %rax,%rax
  1c05c0:	je     1c520d <PRIVATE0000000000134a87+0x519d>
  1c05c6:	movslq 0x8(%rsp),%rsi
  1c05cb:	test   %rsi,%rsi
  1c05ce:	jle    1c5269 <PRIVATE0000000000134a87+0x51f9>
  1c05d4:	mov    %r15,%rdi
  1c05d7:	mov    $0x4,%edx
  1c05dc:	call   8d1bb0 <PRIVATE0000000000ac64e6>
  1c05e1:	mov    %rax,0x1878(%rsp,%r13,1)
  1c05e9:	test   %rax,%rax
  1c05ec:	je     1c520d <PRIVATE0000000000134a87+0x519d>
  1c05f2:	mov    0x68(%r14),%ecx
  1c05f6:	add    0x8(%rsp),%ecx
  1c05fa:	lea    0x2(%rcx,%rcx,1),%r8d
  1c05ff:	lea    (%rcx,%rcx,1),%esi
  1c0602:	test   %r8d,%r8d
  1c0605:	jle    1c5247 <PRIVATE0000000000134a87+0x51d7>
  1c060b:	movslq %esi,%rsi
  1c060e:	mov    %r15,%rdi
  1c0611:	add    $0x2,%rsi
  1c0615:	mov    $0x4,%edx
  1c061a:	call   8d1bb0 <PRIVATE0000000000ac64e6>
  1c061f:	mov    %rax,0x1880(%rsp,%r13,1)
  1c0627:	test   %rax,%rax
  1c062a:	je     1c520d <PRIVATE0000000000134a87+0x519d>
  1c0630:	inc    %rbx
  1c0633:	add    $0x18,%r13
  1c0637:	movslq 0x1864(%rsp),%rcx
  1c063f:	cmp    %rcx,%rbx
  1c0642:	jl     1c059a <PRIVATE0000000000134a87+0x52a>
  1c0648:	mov    %r15,%rdi
  1c064b:	lea    0x1868(%rsp),%rsi
  1c0653:	mov    0x6b0(%rsi),%rbx
  1c065a:	mov    0x6f8(%rsi),%r13
  1c0661:	call   8ecaa0 <PRIVATE0000000000ae18ab>
  1c0666:	mov    0x2068(%rsp),%eax
  1c066d:	movslq %eax,%rax
  1c0670:	movl   $0x0,0x1f70(%rsp)
  1c067b:	mov    %rax,0x1f50(%rsp)
  1c0683:	test   %eax,%eax
  1c0685:	jle    1c51fc <PRIVATE0000000000134a87+0x518c>
  1c068b:	mov    %r15,%rdi
  1c068e:	lea    0x0(,%rax,4),%rsi
  1c0696:	call   8d1880 <PRIVATE0000000000ac645d>
  1c069b:	mov    %rax,0x1f78(%rsp)
  1c06a3:	cmpq   $0x0,0x1f78(%rsp)
  1c06ac:	je     1c51c1 <PRIVATE0000000000134a87+0x5151>
  1c06b2:	mov    0x2060(%rsp),%edx
  1c06b9:	mov    0x2068(%rsp),%eax
  1c06c0:	lea    (%rax,%rdx,1),%ecx
  1c06c3:	mov    %ecx,0x1f58(%rsp)
  1c06ca:	test   %ecx,%ecx
  1c06cc:	jle    1c51f2 <PRIVATE0000000000134a87+0x5182>
  1c06d2:	movslq 0x2060(%rsp),%rcx
  1c06da:	mov    %r15,%rdi
  1c06dd:	mov    0x1f50(%rsp),%rax
  1c06e5:	mov    $0x4,%edx
  1c06ea:	mov    %rcx,(%rsp)
  1c06ee:	lea    (%rax,%rcx,1),%rsi
  1c06f2:	call   8d1bb0 <PRIVATE0000000000ac64e6>
  1c06f7:	mov    %rax,%r13
  1c06fa:	test   %r13,%r13
  1c06fd:	je     1c51c1 <PRIVATE0000000000134a87+0x5151>
  1c0703:	mov    (%rsp),%rsi
  1c0707:	mov    %r15,%rdi
  1c070a:	add    0x1f50(%rsp),%rsi
  1c0712:	shl    $0x3,%rsi
  1c0716:	call   8d1880 <PRIVATE0000000000ac645d>
  1c071b:	mov    %rax,%rbx
  1c071e:	test   %rbx,%rbx
  1c0721:	je     1c51c1 <PRIVATE0000000000134a87+0x5151>
  1c0727:	mov    0x1f50(%rsp),%rax
  1c072f:	add    %rax,%rax
  1c0732:	je     1c51e8 <PRIVATE0000000000134a87+0x5178>
  1c0738:	mov    0x1f50(%rsp),%rax
  1c0740:	mov    %r15,%rdi
  1c0743:	lea    0x0(,%rax,8),%rsi
  1c074b:	call   8d1880 <PRIVATE0000000000ac645d>
  1c0750:	mov    %rax,%r12
  1c0753:	test   %r12,%r12
  1c0756:	je     1c51c1 <PRIVATE0000000000134a87+0x5151>
  1c075c:	mov    0x1f50(%rsp),%rsi
  1c0764:	mov    %r15,%rdi
  1c0767:	shl    $0x4,%rsi
  1c076b:	call   8d1880 <PRIVATE0000000000ac645d>
  1c0770:	mov    %rax,%rbp
  1c0773:	test   %rbp,%rbp
  1c0776:	je     1c51c1 <PRIVATE0000000000134a87+0x5151>
  1c077c:	cmpl   $0x0,0x2050(%rsp)
  1c0784:	je     1c0b5e <PRIVATE0000000000134a87+0xaee>
  1c078a:	mov    0x20f8(%rsp),%rax
  1c0792:	mov    0x2100(%rsp),%rsi
  1c079a:	movl   $0xa,0x294(%r14)
  1c07a5:	lea    0x40(%rax),%rdi
  1c07a9:	mov    %rdi,0x1e70(%rsp)
  1c07b1:	call   8ead90 <PRIVATE0000000000ade8ca>
  1c07b6:	mov    %r14,%rdi
  1c07b9:	mov    0x2100(%rsp),%rsi
  1c07c1:	call   f15c0 <PRIVATE000000000005b257>
  1c07c6:	mov    %eax,(%rsp)
  1c07c9:	cmpl   $0x1,(%rsp)
  1c07cd:	jne    1c07fa <PRIVATE0000000000134a87+0x78a>
  1c07cf:	mov    %r15,0x1f30(%rsp)
  1c07d7:	mov    0x2100(%rsp),%r15
  1c07df:	mov    %r14,%rdi
  1c07e2:	mov    %r15,%rsi
  1c07e5:	call   f15c0 <PRIVATE000000000005b257>
  1c07ea:	cmp    $0x1,%eax
  1c07ed:	je     1c07df <PRIVATE0000000000134a87+0x76f>
  1c07ef:	mov    %eax,(%rsp)
  1c07f2:	mov    0x1f30(%rsp),%r15
  1c07fa:	mov    0x1e70(%rsp),%rdi
  1c0802:	mov    0x2100(%rsp),%rsi
  1c080a:	call   8eacf0 <PRIVATE0000000000ade770>
  1c080f:	cmpl   $0xc,(%rsp)
  1c0813:	je     1c08b0 <PRIVATE0000000000134a87+0x840>
  1c0819:	cmpl   $0x1,(%rsp)
  1c081d:	jle    1c0845 <PRIVATE0000000000134a87+0x7d5>
  1c081f:	mov    0x20f8(%rsp),%rdx
  1c0827:	mov    (%rsp),%eax
  1c082a:	mov    %eax,0x1f70(%rsp)
  1c0831:	lea    0xc0(%rdx),%rcx
  1c0838:	mov    %rcx,0x1f80(%rsp)
  1c0840:	jmp    1c08d5 <PRIVATE0000000000134a87+0x865>
  1c0845:	cmpl   $0xfffffc18,0xf0(%r14)
  1c0850:	jg     1c0881 <PRIVATE0000000000134a87+0x811>
  1c0852:	mov    0x20f8(%rsp),%rcx
  1c085a:	mov    $0x1,%eax
  1c085f:	mov    0x20e8(%rsp),%rdx
  1c0867:	mov    %eax,0xf0(%r14)
  1c086e:	lea    0xc0(%rcx),%r8
  1c0875:	mov    %r8,0x1f80(%rsp)
  1c087d:	mov    %eax,(%rdx)
  1c087f:	jmp    1c08d5 <PRIVATE0000000000134a87+0x865>
  1c0881:	cmpl   $0x0,0x1fa0(%rsp)
  1c0889:	jne    1c0b3f <PRIVATE0000000000134a87+0xacf>
  1c088f:	cmpl   $0x0,0x184(%r14)
  1c0897:	jne    1c08b0 <PRIVATE0000000000134a87+0x840>
  1c0899:	mov    0x1fe0(%rsp),%eax
  1c08a0:	lea    0xa(%rax),%edx
  1c08a3:	cmp    0x178(%r14),%edx
  1c08aa:	jg     1c0b3f <PRIVATE0000000000134a87+0xacf>
  1c08b0:	mov    0x20f8(%rsp),%rdx
  1c08b8:	mov    0x20e8(%rsp),%rax
  1c08c0:	lea    0xc0(%rdx),%rcx
  1c08c7:	mov    %rcx,0x1f80(%rsp)
  1c08cf:	movl   $0x1,(%rax)
  1c08d5:	mov    0x1f80(%rsp),%rdi
  1c08dd:	mov    0x2100(%rsp),%rsi
  1c08e5:	call   8ead40 <PRIVATE0000000000ade86f>
  1c08ea:	mov    0x20f0(%rsp),%rdx
  1c08f2:	mov    0x1fb0(%rsp),%eax
  1c08f9:	cmpq   $0x0,0x1f78(%rsp)
  1c0902:	mov    %eax,(%rdx)
  1c0904:	je     1c0916 <PRIVATE0000000000134a87+0x8a6>
  1c0906:	mov    %r15,%rdi
  1c0909:	mov    0x1f78(%rsp),%rsi
  1c0911:	call   8d2460 <PRIVATE0000000000ac661f>
  1c0916:	test   %r13,%r13
  1c0919:	je     1c0926 <PRIVATE0000000000134a87+0x8b6>
  1c091b:	mov    %r15,%rdi
  1c091e:	mov    %r13,%rsi
  1c0921:	call   8d2460 <PRIVATE0000000000ac661f>
  1c0926:	test   %rbx,%rbx
  1c0929:	je     1c0936 <PRIVATE0000000000134a87+0x8c6>
  1c092b:	mov    %r15,%rdi
  1c092e:	mov    %rbx,%rsi
  1c0931:	call   8d2460 <PRIVATE0000000000ac661f>
  1c0936:	test   %r12,%r12
  1c0939:	je     1c0946 <PRIVATE0000000000134a87+0x8d6>
  1c093b:	mov    %r15,%rdi
  1c093e:	mov    %r12,%rsi
  1c0941:	call   8d2460 <PRIVATE0000000000ac661f>
  1c0946:	test   %rbp,%rbp
  1c0949:	je     1c0956 <PRIVATE0000000000134a87+0x8e6>
  1c094b:	mov    %r15,%rdi
  1c094e:	mov    %rbp,%rsi
  1c0951:	call   8d2460 <PRIVATE0000000000ac661f>
  1c0956:	mov    0x4b8(%r14),%rbx
  1c095d:	xor    %r12d,%r12d
  1c0960:	xor    %ebp,%ebp
  1c0962:	cmpl   $0x0,0x1848(%rsp)
  1c096a:	mov    0x100(%rbx),%rbx
  1c0971:	jle    1c0a55 <PRIVATE0000000000134a87+0x9e5>
  1c0977:	cmpq   $0x0,0x50(%rsp,%rbp,1)
  1c097d:	je     1c0995 <PRIVATE0000000000134a87+0x925>
  1c097f:	mov    %rbx,%rdi
  1c0982:	mov    0x50(%rsp,%rbp,1),%rsi
  1c0987:	call   8d2460 <PRIVATE0000000000ac661f>
  1c098c:	movq   $0x0,0x50(%rsp,%rbp,1)
  1c0995:	cmpq   $0x0,0x58(%rsp,%rbp,1)
  1c099b:	je     1c09b3 <PRIVATE0000000000134a87+0x943>
  1c099d:	mov    %rbx,%rdi
  1c09a0:	mov    0x58(%rsp,%rbp,1),%rsi
  1c09a5:	call   8d2460 <PRIVATE0000000000ac661f>
  1c09aa:	movq   $0x0,0x58(%rsp,%rbp,1)
  1c09b3:	cmpq   $0x0,0x68(%rsp,%rbp,1)
  1c09b9:	je     1c09d1 <PRIVATE0000000000134a87+0x961>
  1c09bb:	mov    %rbx,%rdi
  1c09be:	mov    0x68(%rsp,%rbp,1),%rsi
  1c09c3:	call   8d2460 <PRIVATE0000000000ac661f>
  1c09c8:	movq   $0x0,0x68(%rsp,%rbp,1)
  1c09d1:	cmpq   $0x0,0x70(%rsp,%rbp,1)
  1c09d7:	je     1c09ef <PRIVATE0000000000134a87+0x97f>
  1c09d9:	mov    %rbx,%rdi
  1c09dc:	mov    0x70(%rsp,%rbp,1),%rsi
  1c09e1:	call   8d2460 <PRIVATE0000000000ac661f>
  1c09e6:	movq   $0x0,0x70(%rsp,%rbp,1)
  1c09ef:	cmpq   $0x0,0x80(%rsp,%rbp,1)
  1c09f8:	je     1c0a16 <PRIVATE0000000000134a87+0x9a6>
  1c09fa:	mov    %rbx,%rdi
  1c09fd:	mov    0x80(%rsp,%rbp,1),%rsi
  1c0a05:	call   8d2460 <PRIVATE0000000000ac661f>
  1c0a0a:	movq   $0x0,0x80(%rsp,%rbp,1)
  1c0a16:	cmpq   $0x0,0x88(%rsp,%rbp,1)
  1c0a1f:	je     1c0a3d <PRIVATE0000000000134a87+0x9cd>
  1c0a21:	mov    %rbx,%rdi
  1c0a24:	mov    0x88(%rsp,%rbp,1),%rsi
  1c0a2c:	call   8d2460 <PRIVATE0000000000ac661f>
  1c0a31:	movq   $0x0,0x88(%rsp,%rbp,1)
  1c0a3d:	inc    %r12
  1c0a40:	add    $0x60,%rbp
  1c0a44:	movslq 0x1848(%rsp),%rax
  1c0a4c:	cmp    %rax,%r12
  1c0a4f:	jl     1c0977 <PRIVATE0000000000134a87+0x907>
  1c0a55:	mov    0x1858(%rsp),%rsi
  1c0a5d:	test   %rsi,%rsi
  1c0a60:	je     1c0a76 <PRIVATE0000000000134a87+0xa06>
  1c0a62:	mov    %rbx,%rdi
  1c0a65:	call   8d2460 <PRIVATE0000000000ac661f>
  1c0a6a:	movq   $0x0,0x1858(%rsp)
  1c0a76:	movslq 0x1864(%rsp),%rbp
  1c0a7e:	lea    -0x3(%rbp,%rbp,2),%r12
  1c0a83:	shl    $0x3,%r12
  1c0a87:	dec    %rbp
  1c0a8a:	jl     1c0b0a <PRIVATE0000000000134a87+0xa9a>
  1c0a8c:	cmpq   $0x0,0x1880(%rsp,%r12,1)
  1c0a95:	je     1c0ab3 <PRIVATE0000000000134a87+0xa43>
  1c0a97:	mov    %rbx,%rdi
  1c0a9a:	mov    0x1880(%rsp,%r12,1),%rsi
  1c0aa2:	call   8d2460 <PRIVATE0000000000ac661f>
  1c0aa7:	movq   $0x0,0x1880(%rsp,%r12,1)
  1c0ab3:	cmpq   $0x0,0x1878(%rsp,%r12,1)
  1c0abc:	je     1c0ada <PRIVATE0000000000134a87+0xa6a>
  1c0abe:	mov    %rbx,%rdi
  1c0ac1:	mov    0x1878(%rsp,%r12,1),%rsi
  1c0ac9:	call   8d2460 <PRIVATE0000000000ac661f>
  1c0ace:	movq   $0x0,0x1878(%rsp,%r12,1)
  1c0ada:	cmpq   $0x0,0x1870(%rsp,%r12,1)
  1c0ae3:	je     1c0b01 <PRIVATE0000000000134a87+0xa91>
  1c0ae5:	mov    %rbx,%rdi
  1c0ae8:	mov    0x1870(%rsp,%r12,1),%rsi
  1c0af0:	call   8d2460 <PRIVATE0000000000ac661f>
  1c0af5:	movq   $0x0,0x1870(%rsp,%r12,1)
  1c0b01:	add    $0xffffffffffffffe8,%r12
  1c0b05:	dec    %rbp
  1c0b08:	jns    1c0a8c <PRIVATE0000000000134a87+0xa1c>
  1c0b0a:	mov    %rbx,%rdi
  1c0b0d:	mov    0x1868(%rsp),%rsi
  1c0b15:	call   8ecae0 <PRIVATE0000000000ae1940>
  1c0b1a:	movq   $0x0,0x1868(%rsp)
  1c0b26:	mov    0x1f70(%rsp),%eax
  1c0b2d:	add    $0x20a8,%rsp
  1c0b34:	pop    %rbp
  1c0b35:	pop    %rbx
  1c0b36:	pop    %r15
  1c0b38:	pop    %r14
  1c0b3a:	pop    %r13
  1c0b3c:	pop    %r12
  1c0b3e:	ret
  1c0b3f:	mov    0x2048(%rsp),%rax
  1c0b47:	movl   $0x0,0x294(%r14)
  1c0b52:	mov    0x18(%rax),%rdx
  1c0b56:	mov    %rdx,0x1fe8(%rsp)
  1c0b5e:	xor    %eax,%eax
  1c0b60:	orl    $0x1,0x1d8(%r14)
  1c0b68:	pxor   %xmm0,%xmm0
  1c0b6c:	mov    %eax,0x1e80(%rsp)
  1c0b73:	mov    %eax,0x1e78(%rsp)
  1c0b7a:	mov    %eax,0x1e88(%rsp)
  1c0b81:	cmpl   $0x0,0x2068(%rsp)
  1c0b89:	jle    1c0d66 <PRIVATE0000000000134a87+0xcf6>
  1c0b8f:	mov    0x2000(%rsp),%rax
  1c0b97:	cmpl   $0x6,0x2068(%rsp)
  1c0b9f:	mov    0x10(%rax),%rdx
  1c0ba3:	mov    %rdx,(%rsp)
  1c0ba7:	jle    1c0cde <PRIVATE0000000000134a87+0xc6e>
  1c0bad:	mov    0x1f50(%rsp),%rax
  1c0bb5:	lea    0x0(,%rax,8),%rdx
  1c0bbd:	mov    (%rsp),%rax
  1c0bc1:	sub    0x1ed0(%rsp),%rax
  1c0bc9:	cmp    %rdx,%rax
  1c0bcc:	jge    1c0bda <PRIVATE0000000000134a87+0xb6a>
  1c0bce:	neg    %rax
  1c0bd1:	cmp    %rdx,%rax
  1c0bd4:	jl     1c0cde <PRIVATE0000000000134a87+0xc6e>
  1c0bda:	cmpq   $0x2,0x1f50(%rsp)
  1c0be3:	jl     1c0f4c <PRIVATE0000000000134a87+0xedc>
  1c0be9:	mov    (%rsp),%r8
  1c0bed:	and    $0xf,%r8
  1c0bf1:	test   %r8d,%r8d
  1c0bf4:	je     1c0c09 <PRIVATE0000000000134a87+0xb99>
  1c0bf6:	test   $0x7,%r8d
  1c0bfd:	jne    1c0f4c <PRIVATE0000000000134a87+0xedc>
  1c0c03:	mov    $0x1,%r8d
  1c0c09:	mov    %r8d,%edx
  1c0c0c:	lea    0x2(%rdx),%rax
  1c0c10:	cmp    0x1f50(%rsp),%rax
  1c0c18:	jg     1c0f4c <PRIVATE0000000000134a87+0xedc>
  1c0c1e:	mov    0x2068(%rsp),%ecx
  1c0c25:	mov    %ecx,%eax
  1c0c27:	sub    %r8d,%eax
  1c0c2a:	and    $0x1,%eax
  1c0c2d:	neg    %eax
  1c0c2f:	add    %ecx,%eax
  1c0c31:	xor    %ecx,%ecx
  1c0c33:	movslq %eax,%rax
  1c0c36:	test   %r8d,%r8d
  1c0c39:	jbe    1c0c57 <PRIVATE0000000000134a87+0xbe7>
  1c0c3b:	mov    (%rsp),%r9
  1c0c3f:	mov    0x1ed0(%rsp),%r10
  1c0c47:	mov    (%r10,%rcx,8),%r8
  1c0c4b:	mov    %r8,(%r9,%rcx,8)
  1c0c4f:	inc    %rcx
  1c0c52:	cmp    %rdx,%rcx
  1c0c55:	jb     1c0c47 <PRIVATE0000000000134a87+0xbd7>
  1c0c57:	mov    0x1ed0(%rsp),%rcx
  1c0c5f:	lea    (%rcx,%rdx,8),%r8
  1c0c63:	test   $0xf,%r8
  1c0c6a:	je     1c0c8c <PRIVATE0000000000134a87+0xc1c>
  1c0c6c:	mov    (%rsp),%rcx
  1c0c70:	mov    0x1ed0(%rsp),%r8
  1c0c78:	movups (%r8,%rdx,8),%xmm1
  1c0c7d:	movups %xmm1,(%rcx,%rdx,8)
  1c0c81:	add    $0x2,%rdx
  1c0c85:	cmp    %rax,%rdx
  1c0c88:	jb     1c0c78 <PRIVATE0000000000134a87+0xc08>
  1c0c8a:	jmp    1c0caa <PRIVATE0000000000134a87+0xc3a>
  1c0c8c:	mov    (%rsp),%rcx
  1c0c90:	mov    0x1ed0(%rsp),%r8
  1c0c98:	movups (%r8,%rdx,8),%xmm1
  1c0c9d:	movups %xmm1,(%rcx,%rdx,8)
  1c0ca1:	add    $0x2,%rdx
  1c0ca5:	cmp    %rax,%rdx
  1c0ca8:	jb     1c0c98 <PRIVATE0000000000134a87+0xc28>
  1c0caa:	cmp    0x1f50(%rsp),%rax
  1c0cb2:	jae    1c0d48 <PRIVATE0000000000134a87+0xcd8>
  1c0cb8:	mov    (%rsp),%rcx
  1c0cbc:	mov    0x1f50(%rsp),%r8
  1c0cc4:	mov    0x1ed0(%rsp),%r9
  1c0ccc:	mov    (%r9,%rax,8),%rdx
  1c0cd0:	mov    %rdx,(%rcx,%rax,8)
  1c0cd4:	inc    %rax
  1c0cd7:	cmp    %r8,%rax
  1c0cda:	jb     1c0ccc <PRIVATE0000000000134a87+0xc5c>
  1c0cdc:	jmp    1c0d48 <PRIVATE0000000000134a87+0xcd8>
  1c0cde:	xor    %ecx,%ecx
  1c0ce0:	mov    $0x1,%r8d
  1c0ce6:	mov    0x2068(%rsp),%eax
  1c0ced:	xor    %edx,%edx
  1c0cef:	shr    %eax
  1c0cf1:	je     1c0d22 <PRIVATE0000000000134a87+0xcb2>
  1c0cf3:	mov    (%rsp),%r10
  1c0cf7:	mov    0x1ed0(%rsp),%r11
  1c0cff:	mov    (%rdx,%r11,1),%r8
  1c0d03:	inc    %rcx
  1c0d06:	mov    %r8,(%rdx,%r10,1)
  1c0d0a:	mov    0x8(%rdx,%r11,1),%r9
  1c0d0f:	mov    %r9,0x8(%rdx,%r10,1)
  1c0d14:	add    $0x10,%rdx
  1c0d18:	cmp    %rax,%rcx
  1c0d1b:	jb     1c0cff <PRIVATE0000000000134a87+0xc8f>
  1c0d1d:	lea    0x1(%rcx,%rcx,1),%r8d
  1c0d22:	lea    -0x1(%r8),%eax
  1c0d26:	cmp    0x2068(%rsp),%eax
  1c0d2d:	jae    1c0d48 <PRIVATE0000000000134a87+0xcd8>
  1c0d2f:	movslq %r8d,%r8
  1c0d32:	mov    0x1ed0(%rsp),%rax
  1c0d3a:	mov    (%rsp),%rcx
  1c0d3e:	mov    -0x8(%rax,%r8,8),%rdx
  1c0d43:	mov    %rdx,-0x8(%rcx,%r8,8)
  1c0d48:	cmpl   $0x0,0x2060(%rsp)
  1c0d50:	jle    1c0f53 <PRIVATE0000000000134a87+0xee3>
  1c0d56:	mov    0x2068(%rsp),%eax
  1c0d5d:	mov    %eax,0x2098(%rsp)
  1c0d64:	jmp    1c0d7f <PRIVATE0000000000134a87+0xd0f>
  1c0d66:	cmpl   $0x0,0x2060(%rsp)
  1c0d6e:	jle    1c52f2 <PRIVATE0000000000134a87+0x5282>
  1c0d74:	movl   $0x0,0x2098(%rsp)
  1c0d7f:	movslq 0x2060(%rsp),%rdx
  1c0d87:	xor    %eax,%eax
  1c0d89:	mov    %rdx,0x1f20(%rsp)
  1c0d91:	pxor   %xmm2,%xmm2
  1c0d95:	mov    %rbp,0x1ee8(%rsp)
  1c0d9d:	mov    %r12,0x1ef0(%rsp)
  1c0da5:	mov    %r13,0x1f60(%rsp)
  1c0dad:	mov    %r15,0x1f30(%rsp)
  1c0db5:	mov    %r14,0x1f68(%rsp)
  1c0dbd:	mov    %rbx,0x1f18(%rsp)
  1c0dc5:	movsd  0x1ee0(%rsp),%xmm1
  1c0dce:	mov    0x1e80(%rsp),%ebp
  1c0dd5:	mov    0x1e88(%rsp),%r14d
  1c0ddd:	mov    %rdx,%r13
  1c0de0:	mov    0x1fb0(%rsp),%edi
  1c0de7:	mov    0x2000(%rsp),%r11
  1c0def:	mov    0x1fc0(%rsp),%r15
  1c0df7:	mov    0x1f88(%rsp),%r10
  1c0dff:	mov    0x1f90(%rsp),%rsi
  1c0e07:	mov    0x2038(%rsp),%r12
  1c0e0f:	mov    0x2040(%rsp),%r8
  1c0e17:	mov    0x1fa8(%rsp),%r9
  1c0e1f:	mov    (%r15,%rax,4),%edx
  1c0e23:	test   %edx,%edx
  1c0e25:	jl     1c4fd7 <PRIVATE0000000000134a87+0x4f67>
  1c0e2b:	inc    %rax
  1c0e2e:	cmp    %r13,%rax
  1c0e31:	jl     1c0e1f <PRIVATE0000000000134a87+0xdaf>
  1c0e33:	mov    %ebp,0x1e80(%rsp)
  1c0e3a:	mov    %r14d,0x1e88(%rsp)
  1c0e42:	mov    %edi,0x1fb0(%rsp)
  1c0e49:	mov    0x1ee8(%rsp),%rbp
  1c0e51:	mov    0x1ef0(%rsp),%r12
  1c0e59:	mov    0x1f18(%rsp),%rbx
  1c0e61:	mov    0x1f60(%rsp),%r13
  1c0e69:	mov    0x1f30(%rsp),%r15
  1c0e71:	mov    0x1f68(%rsp),%r14
  1c0e79:	cmpl   $0x0,0x2068(%rsp)
  1c0e81:	jle    1c0fc3 <PRIVATE0000000000134a87+0xf53>
  1c0e87:	movl   $0x0,0x2098(%rsp)
  1c0e92:	xor    %eax,%eax
  1c0e94:	mov    %rbp,0x1ee8(%rsp)
  1c0e9c:	pxor   %xmm4,%xmm4
  1c0ea0:	mov    %r12,0x1ef0(%rsp)
  1c0ea8:	mov    %rbx,0x1f18(%rsp)
  1c0eb0:	mov    %r13,0x1f60(%rsp)
  1c0eb8:	mov    %r15,0x1f30(%rsp)
  1c0ec0:	mov    %r14,0x1f68(%rsp)
  1c0ec8:	movsd  0x1ee0(%rsp),%xmm5
  1c0ed1:	mov    0x1e78(%rsp),%r11d
  1c0ed9:	mov    0x1e80(%rsp),%r10d
  1c0ee1:	mov    0x1e88(%rsp),%r9d
  1c0ee9:	mov    0x1fb0(%rsp),%r12d
  1c0ef1:	mov    0x1ed8(%rsp),%r13
  1c0ef9:	mov    0x1fb8(%rsp),%rdi
  1c0f01:	mov    0x2000(%rsp),%rdx
  1c0f09:	mov    0x1fc0(%rsp),%rcx
  1c0f11:	mov    0x2060(%rsp),%ebx
  1c0f18:	mov    0x2068(%rsp),%ebp
  1c0f1f:	mov    0x1fa8(%rsp),%rsi
  1c0f27:	jmp    1c0f30 <PRIVATE0000000000134a87+0xec0>
  1c0f29:	mov    %eax,0x2098(%rsp)
  1c0f30:	lea    (%rbx,%rax,1),%r8d
  1c0f34:	movslq %r8d,%r15
  1c0f37:	mov    (%rcx,%r15,4),%r14d
  1c0f3b:	test   %r14d,%r14d
  1c0f3e:	jl     1c4f29 <PRIVATE0000000000134a87+0x4eb9>
  1c0f44:	inc    %eax
  1c0f46:	cmp    %ebp,%eax
  1c0f48:	jl     1c0f29 <PRIVATE0000000000134a87+0xeb9>
  1c0f4a:	jmp    1c0f73 <PRIVATE0000000000134a87+0xf03>
  1c0f4c:	xor    %eax,%eax
  1c0f4e:	jmp    1c0caa <PRIVATE0000000000134a87+0xc3a>
  1c0f53:	movslq 0x2060(%rsp),%rax
  1c0f5b:	movl   $0x0,0x2098(%rsp)
  1c0f66:	mov    %rax,0x1f20(%rsp)
  1c0f6e:	jmp    1c0e92 <PRIVATE0000000000134a87+0xe22>
  1c0f73:	mov    %r12d,0x1fb0(%rsp)
  1c0f7b:	mov    %r11d,0x1e78(%rsp)
  1c0f83:	mov    %r10d,0x1e80(%rsp)
  1c0f8b:	mov    %r9d,0x1e88(%rsp)
  1c0f93:	mov    0x1ee8(%rsp),%rbp
  1c0f9b:	mov    0x1ef0(%rsp),%r12
  1c0fa3:	mov    0x1f18(%rsp),%rbx
  1c0fab:	mov    0x1f60(%rsp),%r13
  1c0fb3:	mov    0x1f30(%rsp),%r15
  1c0fbb:	mov    0x1f68(%rsp),%r14
  1c0fc3:	mov    0x1f50(%rsp),%rax
  1c0fcb:	mov    0x1ec8(%rsp),%rcx
  1c0fd3:	and    $0xf,%rcx
  1c0fd7:	movsd  0x1fc8(%rsp),%xmm0
  1c0fe0:	lea    0x0(,%rax,8),%rdx
  1c0fe8:	movsd  0x2070(%rsp),%xmm1
  1c0ff1:	mov    %rdx,0x1f10(%rsp)
  1c0ff9:	mov    %ecx,%edx
  1c0ffb:	mov    0x2068(%rsp),%eax
  1c1002:	and    $0x7,%edx
  1c1005:	xorps  0x28a0284(%rip),%xmm0        # 2a61290 <__STRLITPACK_7+0x964>
  1c100c:	xorps  0x28a027d(%rip),%xmm1        # 2a61290 <__STRLITPACK_7+0x964>
  1c1013:	shr    %eax
  1c1015:	movsd  %xmm0,0x1f00(%rsp)
  1c101e:	mov    %eax,%r8d
  1c1021:	movsd  %xmm1,0x1f08(%rsp)
  1c102a:	mov    %rbp,0x1ee8(%rsp)
  1c1032:	mov    %r12,0x1ef0(%rsp)
  1c103a:	mov    %r13,0x1f60(%rsp)
  1c1042:	mov    %edx,0x1e90(%rsp)
  1c1049:	mov    %ecx,0x1e98(%rsp)
  1c1050:	mov    %r8,0x1ea0(%rsp)
  1c1058:	mov    %rbx,0x1f18(%rsp)
  1c1060:	mov    %eax,%ebx
  1c1062:	mov    %r15,0x1f30(%rsp)
  1c106a:	xor    %r15d,%r15d
  1c106d:	mov    %r14,0x1f68(%rsp)
  1c1075:	xor    %r14d,%r14d
  1c1078:	mov    0x1f50(%rsp),%r13
  1c1080:	mov    0x1fd0(%rsp),%rbp
  1c1088:	mov    0x2068(%rsp),%r12d
  1c1090:	mov    0x1fb0(%rsp),%edi
  1c1097:	mov    %r14,%rax
  1c109a:	movslq %edi,%rdi
  1c109d:	movl   $0x0,0x2098(%rsp)
  1c10a8:	test   %edi,%edi
  1c10aa:	jle    1c113a <PRIVATE0000000000134a87+0x10ca>
  1c10b0:	movsd  0x28a0330(%rip),%xmm2        # 2a613e8 <__STRLITPACK_7+0xabc>
  1c10b8:	mov    0x1f18(%rsp),%rcx
  1c10c0:	mov    0x1ed8(%rsp),%rsi
  1c10c8:	mov    0x1fb8(%rsp),%r8
  1c10d0:	mov    0x2038(%rsp),%r9
  1c10d8:	mov    0x2060(%rsp),%r10d
  1c10e0:	mov    (%r8,%rax,4),%edx
  1c10e4:	cmp    %r10d,%edx
  1c10e7:	jge    1c1111 <PRIVATE0000000000134a87+0x10a1>
  1c10e9:	movslq %edx,%rdx
  1c10ec:	pxor   %xmm1,%xmm1
  1c10f0:	movsd  (%rsi,%rax,8),%xmm0
  1c10f5:	minsd  %xmm2,%xmm0
  1c10f9:	cvtsi2sdl (%r9,%rdx,4),%xmm1
  1c10ff:	addsd  %xmm0,%xmm1
  1c1103:	xorps  0x28a0186(%rip),%xmm1        # 2a61290 <__STRLITPACK_7+0x964>
  1c110a:	movsd  %xmm1,(%rcx,%rax,8)
  1c110f:	jmp    1c112b <PRIVATE0000000000134a87+0x10bb>
  1c1111:	movsd  (%rsi,%rax,8),%xmm0
  1c1116:	movsd  0x28a02c2(%rip),%xmm1        # 2a613e0 <__STRLITPACK_7+0xab4>
  1c111e:	minsd  %xmm2,%xmm0
  1c1122:	subsd  %xmm0,%xmm1
  1c1126:	movsd  %xmm1,(%rcx,%rax,8)
  1c112b:	inc    %rax
  1c112e:	mov    %eax,0x2098(%rsp)
  1c1135:	cmp    %rdi,%rax
  1c1138:	jl     1c10e0 <PRIVATE0000000000134a87+0x1070>
  1c113a:	mov    0x1f18(%rsp),%rsi
  1c1142:	mov    0x1fb8(%rsp),%rdx
  1c114a:	call   8bccb0 <PRIVATE0000000000aabbe3>
  1c114f:	test   %r12d,%r12d
  1c1152:	jle    1c1175 <PRIVATE0000000000134a87+0x1105>
  1c1154:	mov    0x2000(%rsp),%rdx
  1c115c:	mov    0x10(%rdx),%rsi
  1c1160:	cmp    %rsi,%rbp
  1c1163:	je     1c1175 <PRIVATE0000000000134a87+0x1105>
  1c1165:	mov    %rbp,%rdi
  1c1168:	mov    0x1f10(%rsp),%rdx
  1c1170:	call   bcab0 <memcpy@plt>
  1c1175:	mov    0x2008(%rsp),%r15
  1c117d:	mov    0x1f68(%rsp),%r14
  1c1185:	jmp    1c11a9 <PRIVATE0000000000134a87+0x1139>
  1c1187:	xor    %edi,%edi
  1c1189:	mov    0x1f18(%rsp),%rsi
  1c1191:	mov    0x1fb8(%rsp),%rdx
  1c1199:	movl   $0x0,0x2098(%rsp)
  1c11a4:	call   8bccb0 <PRIVATE0000000000aabbe3>
  1c11a9:	mov    0x2000(%rsp),%rsi
  1c11b1:	mov    %r15,%rdx
  1c11b4:	xor    %ecx,%ecx
  1c11b6:	mov    0x2048(%rsp),%rdi
  1c11be:	mov    0x2100(%rsp),%r8
  1c11c6:	movl   $0xffffffff,(%rsi)
  1c11cc:	call   1205d0 <PRIVATE000000000009cf9b>
  1c11d1:	pxor   %xmm3,%xmm3
  1c11d5:	movaps %xmm3,%xmm2
  1c11d8:	movaps %xmm2,%xmm1
  1c11db:	test   %r12d,%r12d
  1c11de:	jle    1c1468 <PRIVATE0000000000134a87+0x13f8>
  1c11e4:	xor    %r8d,%r8d
  1c11e7:	mov    $0x1,%r9d
  1c11ed:	xor    %ecx,%ecx
  1c11ef:	test   %ebx,%ebx
  1c11f1:	je     1c1232 <PRIVATE0000000000134a87+0x11c2>
  1c11f3:	mov    0x1ea0(%rsp),%r9
  1c11fb:	movsd  (%rcx,%rbp,1),%xmm0
  1c1200:	inc    %r8
  1c1203:	andps  0x28a0096(%rip),%xmm0        # 2a612a0 <__STRLITPACK_7+0x974>
  1c120a:	movsd  0x8(%rcx,%rbp,1),%xmm4
  1c1210:	add    $0x10,%rcx
  1c1214:	andps  0x28a0085(%rip),%xmm4        # 2a612a0 <__STRLITPACK_7+0x974>
  1c121b:	maxsd  %xmm0,%xmm1
  1c121f:	comisd %xmm4,%xmm1
  1c1223:	ja     1c1228 <PRIVATE0000000000134a87+0x11b8>
  1c1225:	movaps %xmm4,%xmm1
  1c1228:	cmp    %r9,%r8
  1c122b:	jb     1c11fb <PRIVATE0000000000134a87+0x118b>
  1c122d:	lea    0x1(%r8,%r8,1),%r9d
  1c1232:	lea    -0x1(%r9),%ecx
  1c1236:	cmp    %r12d,%ecx
  1c1239:	jae    1c1250 <PRIVATE0000000000134a87+0x11e0>
  1c123b:	movslq %r9d,%r9
  1c123e:	movsd  -0x8(%rbp,%r9,8),%xmm0
  1c1245:	andps  0x28a0054(%rip),%xmm0        # 2a612a0 <__STRLITPACK_7+0x974>
  1c124c:	maxsd  %xmm0,%xmm1
  1c1250:	mov    %r14,%rdi
  1c1253:	mov    0x10(%r15),%rsi
  1c1257:	mov    0x1ec8(%rsp),%rdx
  1c125f:	mov    %r12d,0x2098(%rsp)
  1c1267:	movsd  %xmm1,0x1ea8(%rsp)
  1c1270:	movsd  %xmm2,0x1eb0(%rsp)
  1c1279:	movsd  %xmm3,0x1eb8(%rsp)
  1c1282:	call   115d60 <PRIVATE0000000000089d8f>
  1c1287:	xor    %r8d,%r8d
  1c128a:	mov    $0x1,%r10d
  1c1290:	movsd  0x1eb8(%rsp),%xmm3
  1c1299:	xor    %ecx,%ecx
  1c129b:	movsd  0x1eb0(%rsp),%xmm2
  1c12a4:	movsd  0x1ea8(%rsp),%xmm1
  1c12ad:	mov    0x10(%r15),%r9
  1c12b1:	test   %ebx,%ebx
  1c12b3:	je     1c12f6 <PRIVATE0000000000134a87+0x1286>
  1c12b5:	mov    0x1ea0(%rsp),%r10
  1c12bd:	movsd  (%rcx,%r9,1),%xmm0
  1c12c3:	inc    %r8
  1c12c6:	andps  0x289ffd3(%rip),%xmm0        # 2a612a0 <__STRLITPACK_7+0x974>
  1c12cd:	movsd  0x8(%rcx,%r9,1),%xmm4
  1c12d4:	add    $0x10,%rcx
  1c12d8:	andps  0x289ffc1(%rip),%xmm4        # 2a612a0 <__STRLITPACK_7+0x974>
  1c12df:	maxsd  %xmm0,%xmm2
  1c12e3:	comisd %xmm4,%xmm2
  1c12e7:	ja     1c12ec <PRIVATE0000000000134a87+0x127c>
  1c12e9:	movaps %xmm4,%xmm2
  1c12ec:	cmp    %r10,%r8
  1c12ef:	jb     1c12bd <PRIVATE0000000000134a87+0x124d>
  1c12f1:	lea    0x1(%r8,%r8,1),%r10d
  1c12f6:	lea    -0x1(%r10),%ecx
  1c12fa:	cmp    %r12d,%ecx
  1c12fd:	jae    1c1314 <PRIVATE0000000000134a87+0x12a4>
  1c12ff:	movslq %r10d,%r10
  1c1302:	movsd  -0x8(%r9,%r10,8),%xmm0
  1c1309:	andps  0x289ff90(%rip),%xmm0        # 2a612a0 <__STRLITPACK_7+0x974>
  1c1310:	maxsd  %xmm0,%xmm2
  1c1314:	xor    %r8d,%r8d
  1c1317:	mov    $0x1,%r9d
  1c131d:	xor    %ecx,%ecx
  1c131f:	test   %ebx,%ebx
  1c1321:	je     1c1377 <PRIVATE0000000000134a87+0x1307>
  1c1323:	mov    0x1ea0(%rsp),%r9
  1c132b:	mov    0x1ec8(%rsp),%r10
  1c1333:	movsd  (%rcx,%rbp,1),%xmm0
  1c1338:	inc    %r8
  1c133b:	movsd  0x8(%rcx,%rbp,1),%xmm4
  1c1341:	subsd  (%rcx,%r10,1),%xmm0
  1c1347:	subsd  0x8(%rcx,%r10,1),%xmm4
  1c134e:	andps  0x289ff4b(%rip),%xmm0        # 2a612a0 <__STRLITPACK_7+0x974>
  1c1355:	add    $0x10,%rcx
  1c1359:	andps  0x289ff40(%rip),%xmm4        # 2a612a0 <__STRLITPACK_7+0x974>
  1c1360:	maxsd  %xmm0,%xmm3
  1c1364:	comisd %xmm4,%xmm3
  1c1368:	ja     1c136d <PRIVATE0000000000134a87+0x12fd>
  1c136a:	movaps %xmm4,%xmm3
  1c136d:	cmp    %r9,%r8
  1c1370:	jb     1c1333 <PRIVATE0000000000134a87+0x12c3>
  1c1372:	lea    0x1(%r8,%r8,1),%r9d
  1c1377:	lea    -0x1(%r9),%ecx
  1c137b:	cmp    %r12d,%ecx
  1c137e:	jae    1c13a4 <PRIVATE0000000000134a87+0x1334>
  1c1380:	movslq %r9d,%r9
  1c1383:	mov    0x1ec8(%rsp),%rcx
  1c138b:	movsd  -0x8(%rbp,%r9,8),%xmm0
  1c1392:	subsd  -0x8(%rcx,%r9,8),%xmm0
  1c1399:	andps  0x289ff00(%rip),%xmm0        # 2a612a0 <__STRLITPACK_7+0x974>
  1c13a0:	maxsd  %xmm0,%xmm3
  1c13a4:	cmp    $0x8,%r13
  1c13a8:	jl     1c162e <PRIVATE0000000000134a87+0x15be>
  1c13ae:	mov    0x1e98(%rsp),%r10d
  1c13b6:	test   %r10d,%r10d
  1c13b9:	je     1c13cf <PRIVATE0000000000134a87+0x135f>
  1c13bb:	cmpl   $0x0,0x1e90(%rsp)
  1c13c3:	jne    1c162e <PRIVATE0000000000134a87+0x15be>
  1c13c9:	mov    $0x1,%r10d
  1c13cf:	mov    %r10d,%r8d
  1c13d2:	lea    0x8(%r8),%rcx
  1c13d6:	cmp    %rcx,%r13
  1c13d9:	jl     1c162e <PRIVATE0000000000134a87+0x15be>
  1c13df:	mov    %r12d,%r9d
  1c13e2:	xor    %ecx,%ecx
  1c13e4:	sub    %r10d,%r9d
  1c13e7:	and    $0x7,%r9d
  1c13eb:	neg    %r9d
  1c13ee:	add    %r12d,%r9d
  1c13f1:	movslq %r9d,%r9
  1c13f4:	test   %r10d,%r10d
  1c13f7:	jbe    1c1413 <PRIVATE0000000000134a87+0x13a3>
  1c13f9:	mov    0x1ec8(%rsp),%r10
  1c1401:	pxor   %xmm0,%xmm0
  1c1405:	movsd  %xmm0,(%r10,%rcx,8)
  1c140b:	inc    %rcx
  1c140e:	cmp    %r8,%rcx
  1c1411:	jb     1c1405 <PRIVATE0000000000134a87+0x1395>
  1c1413:	mov    0x1ec8(%rsp),%rcx
  1c141b:	pxor   %xmm0,%xmm0
  1c141f:	movups %xmm0,(%rcx,%r8,8)
  1c1424:	movups %xmm0,0x10(%rcx,%r8,8)
  1c142a:	movups %xmm0,0x20(%rcx,%r8,8)
  1c1430:	movups %xmm0,0x30(%rcx,%r8,8)
  1c1436:	add    $0x8,%r8
  1c143a:	cmp    %r9,%r8
  1c143d:	jb     1c141f <PRIVATE0000000000134a87+0x13af>
  1c143f:	cmp    %r13,%r9
  1c1442:	jae    1c145e <PRIVATE0000000000134a87+0x13ee>
  1c1444:	mov    0x1ec8(%rsp),%rcx
  1c144c:	pxor   %xmm0,%xmm0
  1c1450:	movsd  %xmm0,(%rcx,%r9,8)
  1c1456:	inc    %r9
  1c1459:	cmp    %r13,%r9
  1c145c:	jb     1c1450 <PRIVATE0000000000134a87+0x13e0>
  1c145e:	mov    %r12d,0x2098(%rsp)
  1c1466:	jmp    1c14c8 <PRIVATE0000000000134a87+0x1458>
  1c1468:	mov    %r14,%rdi
  1c146b:	mov    0x10(%r15),%rsi
  1c146f:	mov    0x1ec8(%rsp),%rdx
  1c1477:	movl   $0x0,0x2098(%rsp)
  1c1482:	movsd  %xmm1,0x1ea8(%rsp)
  1c148b:	movsd  %xmm2,0x1eb0(%rsp)
  1c1494:	movsd  %xmm3,0x1eb8(%rsp)
  1c149d:	call   115d60 <PRIVATE0000000000089d8f>
  1c14a2:	movsd  0x1eb8(%rsp),%xmm3
  1c14ab:	movsd  0x1eb0(%rsp),%xmm2
  1c14b4:	movsd  0x1ea8(%rsp),%xmm1
  1c14bd:	movl   $0x0,0x2098(%rsp)
  1c14c8:	movsd  0x289fed8(%rip),%xmm4        # 2a613a8 <__STRLITPACK_7+0xa7c>
  1c14d0:	mulsd  %xmm1,%xmm4
  1c14d4:	movsd  0x289fe1c(%rip),%xmm0        # 2a612f8 <__STRLITPACK_7+0x9cc>
  1c14dc:	comisd %xmm4,%xmm2
  1c14e0:	ja     1c14f4 <PRIVATE0000000000134a87+0x1484>
  1c14e2:	movsd  0x289fec6(%rip),%xmm4        # 2a613b0 <__STRLITPACK_7+0xa84>
  1c14ea:	comisd %xmm4,%xmm3
  1c14ee:	jbe    1c4ef8 <PRIVATE0000000000134a87+0x4e88>
  1c14f4:	movsd  0x289fde4(%rip),%xmm0        # 2a612e0 <__STRLITPACK_7+0x9b4>
  1c14fc:	comisd 0x390(%r14),%xmm0
  1c1505:	jbe    1c1555 <PRIVATE0000000000134a87+0x14e5>
  1c1507:	mov    %r14,%rdi
  1c150a:	movsd  %xmm0,0x1e70(%rsp)
  1c1513:	movsd  %xmm3,0x1eb8(%rsp)
  1c151c:	call   1bc0e0 <PRIVATE000000000012f5d1>
  1c1521:	movsd  0x1e70(%rsp),%xmm0
  1c152a:	lea    0x289bc7f(%rip),%rsi        # 2a5d1b0 <_fini+0x5ac>
  1c1531:	mov    $0x1,%eax
  1c1536:	mov    0x1f30(%rsp),%rdi
  1c153e:	movsd  %xmm0,0x390(%r14)
  1c1547:	call   889be0 <PRIVATE0000000000a7a18c>
  1c154c:	movsd  0x1eb8(%rsp),%xmm3
  1c1555:	comisd 0x289fda3(%rip),%xmm3        # 2a61300 <__STRLITPACK_7+0x9d4>
  1c155d:	jbe    1c1566 <PRIVATE0000000000134a87+0x14f6>
  1c155f:	incl   0x178(%r14)
  1c1566:	test   %r12d,%r12d
  1c1569:	jle    1c4ee2 <PRIVATE0000000000134a87+0x4e72>
  1c156f:	mov    0x10(%r15),%rsi
  1c1573:	cmp    0x2010(%rsp),%rsi
  1c157b:	je     1c1592 <PRIVATE0000000000134a87+0x1522>
  1c157d:	mov    0x2010(%rsp),%rdi
  1c1585:	mov    0x1f10(%rsp),%rdx
  1c158d:	call   bcab0 <memcpy@plt>
  1c1592:	xor    %eax,%eax
  1c1594:	xor    %ecx,%ecx
  1c1596:	movl   $0x0,0x2098(%rsp)
  1c15a1:	movsd  0x1f00(%rsp),%xmm2
  1c15aa:	movsd  0x1fc8(%rsp),%xmm3
  1c15b3:	mov    0x1fe8(%rsp),%r9
  1c15bb:	mov    0x1ff0(%rsp),%r10
  1c15c3:	mov    0x2010(%rsp),%r11
  1c15cb:	mov    0x1f88(%rsp),%rdx
  1c15d3:	mov    0x1f90(%rsp),%rsi
  1c15db:	jmp    1c15e4 <PRIVATE0000000000134a87+0x1574>
  1c15dd:	mov    %ecx,0x2098(%rsp)
  1c15e4:	movslq (%r9,%rcx,4),%r8
  1c15e8:	movsd  (%rsi,%r8,8),%xmm0
  1c15ee:	comisd %xmm2,%xmm0
  1c15f2:	jbe    1c161e <PRIVATE0000000000134a87+0x15ae>
  1c15f4:	movsd  (%r11,%rcx,8),%xmm1
  1c15fa:	subsd  %xmm0,%xmm1
  1c15fe:	movsd  %xmm1,(%r11,%rcx,8)
  1c1604:	movsd  (%rdx,%r8,8),%xmm0
  1c160a:	comisd %xmm0,%xmm3
  1c160e:	jbe    1c161e <PRIVATE0000000000134a87+0x15ae>
  1c1610:	subsd  (%rsi,%r8,8),%xmm0
  1c1616:	movsd  %xmm0,(%r10,%rcx,8)
  1c161c:	jmp    1c1624 <PRIVATE0000000000134a87+0x15b4>
  1c161e:	movsd  %xmm3,(%r10,%rcx,8)
  1c1624:	inc    %rcx
  1c1627:	cmp    %r13,%rcx
  1c162a:	jl     1c15dd <PRIVATE0000000000134a87+0x156d>
  1c162c:	jmp    1c1636 <PRIVATE0000000000134a87+0x15c6>
  1c162e:	xor    %r9d,%r9d
  1c1631:	jmp    1c143f <PRIVATE0000000000134a87+0x13cf>
  1c1636:	movl   $0x0,0x2098(%rsp)
  1c1641:	xor    %ecx,%ecx
  1c1643:	movsd  0x1f08(%rsp),%xmm4
  1c164c:	pxor   %xmm2,%xmm2
  1c1650:	movsd  0x1f00(%rsp),%xmm5
  1c1659:	pxor   %xmm6,%xmm6
  1c165d:	movsd  0x2070(%rsp),%xmm7
  1c1666:	movsd  0x1fc8(%rsp),%xmm8
  1c1670:	mov    0x1fe8(%rsp),%r9
  1c1678:	mov    0x1ff0(%rsp),%r10
  1c1680:	mov    0x2010(%rsp),%r11
  1c1688:	mov    0x2018(%rsp),%rdx
  1c1690:	mov    0x1f88(%rsp),%rsi
  1c1698:	mov    0x1f90(%rsp),%rdi
  1c16a0:	movslq (%r9,%rcx,4),%r8
  1c16a4:	comisd (%rdi,%r8,8),%xmm5
  1c16aa:	jb     1c16ca <PRIVATE0000000000134a87+0x165a>
  1c16ac:	movsd  (%rsi,%r8,8),%xmm0
  1c16b2:	comisd %xmm8,%xmm0
  1c16b7:	jb     1c16ca <PRIVATE0000000000134a87+0x165a>
  1c16b9:	movb   $0x46,(%rcx,%rdx,1)
  1c16bd:	movslq 0x2098(%rsp),%rcx
  1c16c5:	jmp    1c1773 <PRIVATE0000000000134a87+0x1703>
  1c16ca:	movsd  (%r11,%rcx,8),%xmm3
  1c16d0:	comisd %xmm3,%xmm4
  1c16d4:	jbe    1c16ef <PRIVATE0000000000134a87+0x167f>
  1c16d6:	movb   $0x42,(%rcx,%rdx,1)
  1c16da:	inc    %eax
  1c16dc:	movslq 0x2098(%rsp),%rcx
  1c16e4:	subsd  (%r11,%rcx,8),%xmm2
  1c16ea:	jmp    1c1773 <PRIVATE0000000000134a87+0x1703>
  1c16ef:	comisd %xmm3,%xmm7
  1c16f3:	jbe    1c1715 <PRIVATE0000000000134a87+0x16a5>
  1c16f5:	movsd  (%r10,%rcx,8),%xmm0
  1c16fb:	ucomisd %xmm6,%xmm0
  1c16ff:	jp     1c1707 <PRIVATE0000000000134a87+0x1697>
  1c1701:	je     1c1c99 <PRIVATE0000000000134a87+0x1c29>
  1c1707:	movb   $0x4c,(%rcx,%rdx,1)
  1c170b:	movslq 0x2098(%rsp),%rcx
  1c1713:	jmp    1c1773 <PRIVATE0000000000134a87+0x1703>
  1c1715:	movsd  (%r10,%rcx,8),%xmm1
  1c171b:	movaps %xmm1,%xmm0
  1c171e:	subsd  %xmm7,%xmm0
  1c1722:	comisd %xmm3,%xmm0
  1c1726:	ja     1c172f <PRIVATE0000000000134a87+0x16bf>
  1c1728:	comisd %xmm8,%xmm1
  1c172d:	jb     1c173d <PRIVATE0000000000134a87+0x16cd>
  1c172f:	movb   $0x4e,(%rcx,%rdx,1)
  1c1733:	movslq 0x2098(%rsp),%rcx
  1c173b:	jmp    1c1773 <PRIVATE0000000000134a87+0x1703>
  1c173d:	addsd  %xmm7,%xmm1
  1c1741:	comisd %xmm3,%xmm1
  1c1745:	jbe    1c1755 <PRIVATE0000000000134a87+0x16e5>
  1c1747:	movb   $0x55,(%rcx,%rdx,1)
  1c174b:	movslq 0x2098(%rsp),%rcx
  1c1753:	jmp    1c1773 <PRIVATE0000000000134a87+0x1703>
  1c1755:	movb   $0x41,(%rcx,%rdx,1)
  1c1759:	inc    %eax
  1c175b:	movslq 0x2098(%rsp),%rcx
  1c1763:	movsd  (%r11,%rcx,8),%xmm0
  1c1769:	subsd  (%r10,%rcx,8),%xmm0
  1c176f:	addsd  %xmm0,%xmm2
  1c1773:	inc    %rcx
  1c1776:	mov    %ecx,0x2098(%rsp)
  1c177d:	cmp    %r13,%rcx
  1c1780:	jl     1c16a0 <PRIVATE0000000000134a87+0x1630>
  1c1786:	cmpl   $0x0,0x2050(%rsp)
  1c178e:	je     1c17c4 <PRIVATE0000000000134a87+0x1754>
  1c1790:	mov    0x4b8(%r14),%rdi
  1c1797:	mov    0x20e0(%rsp),%rsi
  1c179f:	mov    0x2100(%rsp),%rdx
  1c17a7:	movsd  %xmm2,(%rsp)
  1c17ac:	mov    %eax,0x1f28(%rsp)
  1c17b3:	call   1bbf80 <PRIVATE000000000012f282>
  1c17b8:	mov    0x1f28(%rsp),%eax
  1c17bf:	movsd  (%rsp),%xmm2
  1c17c4:	mov    0x20e0(%rsp),%rcx
  1c17cc:	cmpl   $0x2,0x1f98(%rsp)
  1c17d4:	movsd  0x289faf4(%rip),%xmm0        # 2a612d0 <__STRLITPACK_7+0x9a4>
  1c17dc:	jne    1c17e1 <PRIVATE0000000000134a87+0x1771>
  1c17de:	movaps %xmm2,%xmm0
  1c17e1:	movsd  (%rcx),%xmm3
  1c17e5:	cmpl   $0x0,0x1fa0(%rsp)
  1c17ed:	jne    1c1829 <PRIVATE0000000000134a87+0x17b9>
  1c17ef:	movsd  0x289fb99(%rip),%xmm1        # 2a61390 <__STRLITPACK_7+0xa64>
  1c17f7:	comisd %xmm1,%xmm3
  1c17fb:	ja     1c1c64 <PRIVATE0000000000134a87+0x1bf4>
  1c1801:	movsd  0x289fb9f(%rip),%xmm1        # 2a613a8 <__STRLITPACK_7+0xa7c>
  1c1809:	comisd %xmm1,%xmm0
  1c180d:	ja     1c1c64 <PRIVATE0000000000134a87+0x1bf4>
  1c1813:	mulsd  %xmm3,%xmm0
  1c1817:	movsd  0x289fbb9(%rip),%xmm1        # 2a613d8 <__STRLITPACK_7+0xaac>
  1c181f:	comisd %xmm1,%xmm0
  1c1823:	ja     1c1c64 <PRIVATE0000000000134a87+0x1bf4>
  1c1829:	movsd  0x289fb97(%rip),%xmm0        # 2a613c8 <__STRLITPACK_7+0xa9c>
  1c1831:	comisd %xmm0,%xmm3
  1c1835:	ja     1c4e90 <PRIVATE0000000000134a87+0x4e20>
  1c183b:	movsd  0x289fb8d(%rip),%xmm0        # 2a613d0 <__STRLITPACK_7+0xaa4>
  1c1843:	comisd %xmm0,%xmm2
  1c1847:	ja     1c4e37 <PRIVATE0000000000134a87+0x4dc7>
  1c184d:	mov    0x1ec0(%rsp),%cl
  1c1854:	test   %cl,%cl
  1c1856:	jne    1c1dd7 <PRIVATE0000000000134a87+0x1d67>
  1c185c:	comisd 0x289fa6c(%rip),%xmm2        # 2a612d0 <__STRLITPACK_7+0x9a4>
  1c1864:	jbe    1c1dd7 <PRIVATE0000000000134a87+0x1d67>
  1c186a:	movsd  0x2070(%rsp),%xmm0
  1c1873:	comisd 0x120(%r14),%xmm0
  1c187c:	jbe    1c1dd7 <PRIVATE0000000000134a87+0x1d67>
  1c1882:	mov    0x1fb0(%rsp),%ecx
  1c1889:	cmp    0x1e88(%rsp),%ecx
  1c1890:	jle    1c1dd7 <PRIVATE0000000000134a87+0x1d67>
  1c1896:	movb   $0x1,0x1ec0(%rsp)
  1c189e:	cmp    0x1e80(%rsp),%ecx
  1c18a5:	jle    1c18c3 <PRIVATE0000000000134a87+0x1853>
  1c18a7:	movsd  0x1ee0(%rsp),%xmm0
  1c18b0:	mulsd  0x289fa38(%rip),%xmm0        # 2a612f0 <__STRLITPACK_7+0x9c4>
  1c18b8:	movsd  %xmm0,0x1ee0(%rsp)
  1c18c1:	jmp    1c1902 <PRIVATE0000000000134a87+0x1892>
  1c18c3:	cmp    0x1e78(%rsp),%ecx
  1c18ca:	jle    1c18e8 <PRIVATE0000000000134a87+0x1878>
  1c18cc:	movsd  0x1ee0(%rsp),%xmm0
  1c18d5:	mulsd  0x289fac3(%rip),%xmm0        # 2a613a0 <__STRLITPACK_7+0xa74>
  1c18dd:	movsd  %xmm0,0x1ee0(%rsp)
  1c18e6:	jmp    1c1902 <PRIVATE0000000000134a87+0x1892>
  1c18e8:	movsd  0x1ee0(%rsp),%xmm0
  1c18f1:	mulsd  0x289f9df(%rip),%xmm0        # 2a612d8 <__STRLITPACK_7+0x9ac>
  1c18f9:	movsd  %xmm0,0x1ee0(%rsp)
  1c1902:	movl   $0x0,0x1fb0(%rsp)
  1c190d:	test   %r12d,%r12d
  1c1910:	jle    1c1aa4 <PRIVATE0000000000134a87+0x1a34>
  1c1916:	mov    0x2000(%rsp),%rdx
  1c191e:	xor    %r15d,%r15d
  1c1921:	mov    %r14,0x1f68(%rsp)
  1c1929:	xor    %r14d,%r14d
  1c192c:	mov    0x10(%rdx),%rdx
  1c1930:	cmp    $0x6,%r12d
  1c1934:	jle    1c1a26 <PRIVATE0000000000134a87+0x19b6>
  1c193a:	mov    %rdx,%rcx
  1c193d:	sub    0x1ed0(%rsp),%rcx
  1c1945:	cmp    0x1f10(%rsp),%rcx
  1c194d:	jge    1c1960 <PRIVATE0000000000134a87+0x18f0>
  1c194f:	neg    %rcx
  1c1952:	cmp    0x1f10(%rsp),%rcx
  1c195a:	jl     1c1a26 <PRIVATE0000000000134a87+0x19b6>
  1c1960:	cmp    $0x2,%r13
  1c1964:	jl     1c1c91 <PRIVATE0000000000134a87+0x1c21>
  1c196a:	mov    %rdx,%r8
  1c196d:	and    $0xf,%r8
  1c1971:	test   %r8d,%r8d
  1c1974:	je     1c1989 <PRIVATE0000000000134a87+0x1919>
  1c1976:	test   $0x7,%r8d
  1c197d:	jne    1c1c91 <PRIVATE0000000000134a87+0x1c21>
  1c1983:	mov    $0x1,%r8d
  1c1989:	mov    %r8d,%ecx
  1c198c:	lea    0x2(%rcx),%rsi
  1c1990:	cmp    %rsi,%r13
  1c1993:	jl     1c1c91 <PRIVATE0000000000134a87+0x1c21>
  1c1999:	mov    %r12d,%eax
  1c199c:	mov    %r14,%rsi
  1c199f:	sub    %r8d,%eax
  1c19a2:	and    $0x1,%eax
  1c19a5:	neg    %eax
  1c19a7:	add    %r12d,%eax
  1c19aa:	movslq %eax,%rax
  1c19ad:	test   %r8d,%r8d
  1c19b0:	jbe    1c19ca <PRIVATE0000000000134a87+0x195a>
  1c19b2:	mov    0x1ed0(%rsp),%r9
  1c19ba:	mov    (%r9,%rsi,8),%r8
  1c19be:	mov    %r8,(%rdx,%rsi,8)
  1c19c2:	inc    %rsi
  1c19c5:	cmp    %rcx,%rsi
  1c19c8:	jb     1c19ba <PRIVATE0000000000134a87+0x194a>
  1c19ca:	mov    0x1ed0(%rsp),%rsi
  1c19d2:	lea    (%rsi,%rcx,8),%r8
  1c19d6:	test   $0xf,%r8
  1c19dd:	je     1c19f2 <PRIVATE0000000000134a87+0x1982>
  1c19df:	movups (%rsi,%rcx,8),%xmm0
  1c19e3:	movups %xmm0,(%rdx,%rcx,8)
  1c19e7:	add    $0x2,%rcx
  1c19eb:	cmp    %rax,%rcx
  1c19ee:	jb     1c19df <PRIVATE0000000000134a87+0x196f>
  1c19f0:	jmp    1c1a03 <PRIVATE0000000000134a87+0x1993>
  1c19f2:	movups (%rsi,%rcx,8),%xmm0
  1c19f6:	movups %xmm0,(%rdx,%rcx,8)
  1c19fa:	add    $0x2,%rcx
  1c19fe:	cmp    %rax,%rcx
  1c1a01:	jb     1c19f2 <PRIVATE0000000000134a87+0x1982>
  1c1a03:	cmp    %r13,%rax
  1c1a06:	jae    1c1a8c <PRIVATE0000000000134a87+0x1a1c>
  1c1a0c:	mov    0x1ed0(%rsp),%rsi
  1c1a14:	mov    (%rsi,%rax,8),%rcx
  1c1a18:	mov    %rcx,(%rdx,%rax,8)
  1c1a1c:	inc    %rax
  1c1a1f:	cmp    %r13,%rax
  1c1a22:	jb     1c1a14 <PRIVATE0000000000134a87+0x19a4>
  1c1a24:	jmp    1c1a8c <PRIVATE0000000000134a87+0x1a1c>
  1c1a26:	mov    %r14,%rcx
  1c1a29:	mov    $0x1,%esi
  1c1a2e:	mov    %rcx,%rax
  1c1a31:	test   %ebx,%ebx
  1c1a33:	je     1c1a6f <PRIVATE0000000000134a87+0x19ff>
  1c1a35:	mov    0x1ea0(%rsp),%r9
  1c1a3d:	mov    0x1ed0(%rsp),%r10
  1c1a45:	mov    (%rax,%r10,1),%rsi
  1c1a49:	inc    %rcx
  1c1a4c:	mov    %rsi,(%rax,%rdx,1)
  1c1a50:	mov    0x8(%rax,%r10,1),%r8
  1c1a55:	mov    %r8,0x8(%rax,%rdx,1)
  1c1a5a:	add    $0x10,%rax
  1c1a5e:	cmp    %r9,%rcx
  1c1a61:	jb     1c1a45 <PRIVATE0000000000134a87+0x19d5>
  1c1a63:	mov    %r9,0x1ea0(%rsp)
  1c1a6b:	lea    0x1(%rcx,%rcx,1),%esi
  1c1a6f:	lea    -0x1(%rsi),%ecx
  1c1a72:	cmp    %r12d,%ecx
  1c1a75:	jae    1c1a8c <PRIVATE0000000000134a87+0x1a1c>
  1c1a77:	movslq %esi,%rsi
  1c1a7a:	mov    0x1ed0(%rsp),%rcx
  1c1a82:	mov    -0x8(%rcx,%rsi,8),%r8
  1c1a87:	mov    %r8,-0x8(%rdx,%rsi,8)
  1c1a8c:	cmpl   $0x0,0x2060(%rsp)
  1c1a94:	jle    1c1b7b <PRIVATE0000000000134a87+0x1b0b>
  1c1a9a:	mov    %r12d,0x2098(%rsp)
  1c1aa2:	jmp    1c1ad7 <PRIVATE0000000000134a87+0x1a67>
  1c1aa4:	cmpl   $0x0,0x2060(%rsp)
  1c1aac:	jle    1c1187 <PRIVATE0000000000134a87+0x1117>
  1c1ab2:	mov    0x2000(%rsp),%rdx
  1c1aba:	xor    %r15d,%r15d
  1c1abd:	mov    %r14,0x1f68(%rsp)
  1c1ac5:	xor    %r14d,%r14d
  1c1ac8:	movl   $0x0,0x2098(%rsp)
  1c1ad3:	mov    0x10(%rdx),%rdx
  1c1ad7:	movsd  0x1ee0(%rsp),%xmm2
  1c1ae0:	mov    %r14,%rax
  1c1ae3:	mov    0x1f20(%rsp),%r10
  1c1aeb:	pxor   %xmm1,%xmm1
  1c1aef:	mov    0x1fb0(%rsp),%r12d
  1c1af7:	mov    0x1ed8(%rsp),%rcx
  1c1aff:	mov    0x1fb8(%rsp),%rbp
  1c1b07:	mov    0x1fc0(%rsp),%rsi
  1c1b0f:	mov    0x1f88(%rsp),%rdi
  1c1b17:	mov    0x1f90(%rsp),%r13
  1c1b1f:	mov    0x2038(%rsp),%r8
  1c1b27:	mov    0x2040(%rsp),%r14
  1c1b2f:	mov    0x1fa8(%rsp),%r15
  1c1b37:	mov    (%rsi,%rax,4),%r9d
  1c1b3b:	test   %r9d,%r9d
  1c1b3e:	jl     1c1caa <PRIVATE0000000000134a87+0x1c3a>
  1c1b44:	inc    %rax
  1c1b47:	cmp    %r10,%rax
  1c1b4a:	jl     1c1b37 <PRIVATE0000000000134a87+0x1ac7>
  1c1b4c:	mov    %r12d,0x1fb0(%rsp)
  1c1b54:	xor    %r15d,%r15d
  1c1b57:	mov    0x2068(%rsp),%r12d
  1c1b5f:	xor    %r14d,%r14d
  1c1b62:	mov    0x1f50(%rsp),%r13
  1c1b6a:	mov    0x1fd0(%rsp),%rbp
  1c1b72:	test   %r12d,%r12d
  1c1b75:	jle    1c1090 <PRIVATE0000000000134a87+0x1020>
  1c1b7b:	movl   $0x0,0x2098(%rsp)
  1c1b86:	movsd  0x1ee0(%rsp),%xmm3
  1c1b8f:	mov    %r15d,%eax
  1c1b92:	mov    0x2058(%rsp),%r9
  1c1b9a:	pxor   %xmm2,%xmm2
  1c1b9e:	mov    0x1fb0(%rsp),%r10d
  1c1ba6:	mov    0x1ed8(%rsp),%rdi
  1c1bae:	mov    0x1fb8(%rsp),%r11
  1c1bb6:	mov    0x1fc0(%rsp),%rbp
  1c1bbe:	mov    0x2060(%rsp),%esi
  1c1bc5:	jmp    1c1bce <PRIVATE0000000000134a87+0x1b5e>
  1c1bc7:	mov    %eax,0x2098(%rsp)
  1c1bce:	lea    (%rsi,%rax,1),%r8d
  1c1bd2:	movslq %r8d,%rcx
  1c1bd5:	mov    0x0(%rbp,%rcx,4),%r14d
  1c1bda:	test   %r14d,%r14d
  1c1bdd:	jl     1c1bfe <PRIVATE0000000000134a87+0x1b8e>
  1c1bdf:	inc    %eax
  1c1be1:	cmp    %r12d,%eax
  1c1be4:	jl     1c1bc7 <PRIVATE0000000000134a87+0x1b57>
  1c1be6:	mov    %r10d,0x1fb0(%rsp)
  1c1bee:	xor    %r14d,%r14d
  1c1bf1:	mov    0x1fd0(%rsp),%rbp
  1c1bf9:	jmp    1c1090 <PRIVATE0000000000134a87+0x1020>
  1c1bfe:	cmp    $0xfffffffd,%r14d
  1c1c02:	jl     1c1bdf <PRIVATE0000000000134a87+0x1b6f>
  1c1c04:	cmp    $0xffffffff,%r14d
  1c1c08:	je     1c1c50 <PRIVATE0000000000134a87+0x1be0>
  1c1c0a:	movslq %r10d,%r10
  1c1c0d:	movl   $0xfffffffd,0x0(%rbp,%rcx,4)
  1c1c15:	movslq 0x2098(%rsp),%rax
  1c1c1d:	mov    %r8d,(%r11,%r10,4)
  1c1c21:	movsd  (%r9,%rax,8),%xmm0
  1c1c27:	mov    0x2098(%rsp),%eax
  1c1c2e:	movslq %eax,%r14
  1c1c31:	movsd  %xmm0,(%rdi,%r10,8)
  1c1c37:	inc    %r10d
  1c1c3a:	shl    $0x3,%r14
  1c1c3e:	movsd  (%r14,%rdx,1),%xmm1
  1c1c44:	subsd  %xmm0,%xmm1
  1c1c48:	movsd  %xmm1,(%r14,%rdx,1)
  1c1c4e:	jmp    1c1bdf <PRIVATE0000000000134a87+0x1b6f>
  1c1c50:	movslq %eax,%r14
  1c1c53:	shl    $0x3,%r14
  1c1c57:	comisd (%r9,%r14,1),%xmm3
  1c1c5d:	jbe    1c1c0a <PRIVATE0000000000134a87+0x1b9a>
  1c1c5f:	movaps %xmm2,%xmm0
  1c1c62:	jmp    1c1c3e <PRIVATE0000000000134a87+0x1bce>
  1c1c64:	mov    0x1ee8(%rsp),%rbp
  1c1c6c:	mov    0x1ef0(%rsp),%r12
  1c1c74:	mov    0x1f18(%rsp),%rbx
  1c1c7c:	mov    0x1f60(%rsp),%r13
  1c1c84:	mov    0x1f30(%rsp),%r15
  1c1c8c:	jmp    1c08b0 <PRIVATE0000000000134a87+0x840>
  1c1c91:	mov    %r14,%rax
  1c1c94:	jmp    1c1a03 <PRIVATE0000000000134a87+0x1993>
  1c1c99:	movb   $0x58,(%rcx,%rdx,1)
  1c1c9d:	movslq 0x2098(%rsp),%rcx
  1c1ca5:	jmp    1c1773 <PRIVATE0000000000134a87+0x1703>
  1c1caa:	cmp    $0xfffffffd,%r9d
  1c1cae:	jl     1c1b44 <PRIVATE0000000000134a87+0x1ad4>
  1c1cb4:	movsd  (%r15,%rax,8),%xmm0
  1c1cba:	cmp    $0xffffffff,%r9d
  1c1cbe:	je     1c1db7 <PRIVATE0000000000134a87+0x1d47>
  1c1cc4:	cmp    $0xfffffffe,%r9d
  1c1cc8:	je     1c1d99 <PRIVATE0000000000134a87+0x1d29>
  1c1cce:	movsd  (%rdi,%rax,8),%xmm5
  1c1cd3:	movsd  0x0(%r13,%rax,8),%xmm6
  1c1cda:	movaps %xmm0,%xmm4
  1c1cdd:	movaps %xmm5,%xmm3
  1c1ce0:	addsd  %xmm0,%xmm4
  1c1ce4:	addsd  %xmm6,%xmm3
  1c1ce8:	movl   $0xfffffffd,(%rsi,%rax,4)
  1c1cef:	movslq %r12d,%r9
  1c1cf2:	comisd %xmm4,%xmm3
  1c1cf6:	jbe    1c1d07 <PRIVATE0000000000134a87+0x1c97>
  1c1cf8:	movaps %xmm0,%xmm3
  1c1cfb:	subsd  %xmm6,%xmm3
  1c1cff:	movsd  %xmm3,(%rcx,%r9,8)
  1c1d05:	jmp    1c1d11 <PRIVATE0000000000134a87+0x1ca1>
  1c1d07:	subsd  %xmm0,%xmm5
  1c1d0b:	movsd  %xmm5,(%rcx,%r9,8)
  1c1d11:	mov    %eax,0x0(%rbp,%r9,4)
  1c1d16:	inc    %r12d
  1c1d19:	ucomisd %xmm1,%xmm0
  1c1d1d:	jp     1c1d25 <PRIVATE0000000000134a87+0x1cb5>
  1c1d1f:	je     1c1b44 <PRIVATE0000000000134a87+0x1ad4>
  1c1d25:	mov    (%r14,%rax,8),%r11
  1c1d29:	mov    %r11,%r9
  1c1d2c:	cmpl   $0x0,(%r8,%rax,4)
  1c1d31:	jle    1c1b44 <PRIVATE0000000000134a87+0x1ad4>
  1c1d37:	mov    0x2028(%rsp),%r13
  1c1d3f:	mov    0x2030(%rsp),%r14
  1c1d47:	movaps %xmm0,%xmm3
  1c1d4a:	mulsd  0x0(%r13,%r9,8),%xmm3
  1c1d51:	movslq (%r14,%r9,4),%r15
  1c1d55:	inc    %r9
  1c1d58:	mov    %r15d,0x2098(%rsp)
  1c1d60:	movsd  (%rdx,%r15,8),%xmm4
  1c1d66:	subsd  %xmm3,%xmm4
  1c1d6a:	movsd  %xmm4,(%rdx,%r15,8)
  1c1d70:	movslq (%r8,%rax,4),%r15
  1c1d74:	add    %r11,%r15
  1c1d77:	cmp    %r15,%r9
  1c1d7a:	jl     1c1d47 <PRIVATE0000000000134a87+0x1cd7>
  1c1d7c:	mov    0x1f90(%rsp),%r13
  1c1d84:	mov    0x2040(%rsp),%r14
  1c1d8c:	mov    0x1fa8(%rsp),%r15
  1c1d94:	jmp    1c1b44 <PRIVATE0000000000134a87+0x1ad4>
  1c1d99:	movsd  (%rdi,%rax,8),%xmm5
  1c1d9e:	movaps %xmm5,%xmm3
  1c1da1:	subsd  %xmm2,%xmm3
  1c1da5:	comisd %xmm3,%xmm0
  1c1da9:	jbe    1c1cd3 <PRIVATE0000000000134a87+0x1c63>
  1c1daf:	movaps %xmm5,%xmm0
  1c1db2:	jmp    1c1d19 <PRIVATE0000000000134a87+0x1ca9>
  1c1db7:	movsd  0x0(%r13,%rax,8),%xmm4
  1c1dbe:	movaps %xmm2,%xmm3
  1c1dc1:	addsd  %xmm4,%xmm3
  1c1dc5:	comisd %xmm0,%xmm3
  1c1dc9:	jbe    1c1cce <PRIVATE0000000000134a87+0x1c5e>
  1c1dcf:	movaps %xmm4,%xmm0
  1c1dd2:	jmp    1c1d19 <PRIVATE0000000000134a87+0x1ca9>
  1c1dd7:	mov    %eax,0x1f28(%rsp)
  1c1dde:	mov    0x1ee8(%rsp),%rbp
  1c1de6:	mov    0x1ef0(%rsp),%r12
  1c1dee:	mov    0x1f18(%rsp),%rbx
  1c1df6:	mov    0x1f60(%rsp),%r13
  1c1dfe:	mov    0x1f30(%rsp),%r15
  1c1e06:	pxor   %xmm3,%xmm3
  1c1e0a:	mov    %r14,%rdi
  1c1e0d:	cvtsi2sdl 0x1fb0(%rsp),%xmm3
  1c1e16:	movsd  0x2a0(%r14),%xmm0
  1c1e1f:	pxor   %xmm1,%xmm1
  1c1e23:	mov    $0x3,%esi
  1c1e28:	mov    $0x1,%edx
  1c1e2d:	mov    0x2100(%rsp),%rcx
  1c1e35:	movsd  %xmm2,0x120(%r14)
  1c1e3e:	movsd  %xmm3,(%rsp)
  1c1e43:	addsd  0x289f485(%rip),%xmm0        # 2a612d0 <__STRLITPACK_7+0x9a4>
  1c1e4b:	movsd  %xmm0,0x2a0(%r14)
  1c1e54:	call   164c60 <PRIVATE00000000000d07d1>
  1c1e59:	mov    0x20f8(%rsp),%rax
  1c1e61:	mov    0x2100(%rsp),%rsi
  1c1e69:	lea    0xc0(%rax),%rdi
  1c1e70:	mov    %rdi,0x1f80(%rsp)
  1c1e78:	call   8ead90 <PRIVATE0000000000ade8ca>
  1c1e7d:	pxor   %xmm0,%xmm0
  1c1e81:	cvtsi2sdl 0x2068(%rsp),%xmm0
  1c1e8a:	movsd  %xmm0,0x1e98(%rsp)
  1c1e93:	call   bca60 <sqrt@plt>
  1c1e98:	movsd  (%rsp),%xmm1
  1c1e9d:	mov    0x1fb0(%rsp),%r8d
  1c1ea5:	cmpnltsd %xmm0,%xmm1
  1c1eaa:	movd   %xmm1,%eax
  1c1eae:	mov    0x1fb8(%rsp),%rdx
  1c1eb6:	neg    %eax
  1c1eb8:	mov    0x1f78(%rsp),%rcx
  1c1ec0:	mov    %eax,0x209c(%rsp)
  1c1ec7:	mov    %r8d,0x1e90(%rsp)
  1c1ecf:	mov    %r8d,0xc(%rsp)
  1c1ed4:	mov    %rdx,0x10(%rsp)
  1c1ed9:	mov    %r13,0x18(%rsp)
  1c1ede:	mov    %rcx,0x20(%rsp)
  1c1ee3:	test   %r8d,%r8d
  1c1ee6:	je     1c4050 <PRIVATE0000000000134a87+0x3fe0>
  1c1eec:	movsd  0x1e98(%rsp),%xmm0
  1c1ef5:	movsd  0x289f50b(%rip),%xmm2        # 2a61408 <__STRLITPACK_7+0xadc>
  1c1efd:	mulsd  %xmm0,%xmm2
  1c1f01:	mov    0x1f20(%rsp),%rax
  1c1f09:	movsd  0x2070(%rsp),%xmm1
  1c1f12:	movsd  0x289f4de(%rip),%xmm5        # 2a613f8 <__STRLITPACK_7+0xacc>
  1c1f1a:	movsd  0x289f436(%rip),%xmm4        # 2a61358 <__STRLITPACK_7+0xa2c>
  1c1f22:	shl    $0x3,%rax
  1c1f26:	movsd  0x289f4d2(%rip),%xmm3        # 2a61400 <__STRLITPACK_7+0xad4>
  1c1f2e:	mulsd  %xmm1,%xmm5
  1c1f32:	mulsd  %xmm1,%xmm4
  1c1f36:	addsd  %xmm1,%xmm1
  1c1f3a:	mulsd  %xmm0,%xmm3
  1c1f3e:	mulsd  %xmm0,%xmm2
  1c1f42:	mov    0x20f8(%rsp),%rdx
  1c1f4a:	mov    0x1fd0(%rsp),%rcx
  1c1f52:	sub    %rax,%rcx
  1c1f55:	neg    %rax
  1c1f58:	add    0x2058(%rsp),%rax
  1c1f60:	lea    0x100(%rdx),%rdi
  1c1f67:	mov    %r15,0x1f30(%rsp)
  1c1f6f:	addl   $0xa,0x1fe0(%rsp)
  1c1f77:	mov    %r8d,%edx
  1c1f7a:	movsd  0x289f386(%rip),%xmm7        # 2a61308 <__STRLITPACK_7+0x9dc>
  1c1f82:	movsd  0x289f466(%rip),%xmm6        # 2a613f0 <__STRLITPACK_7+0xac4>
  1c1f8a:	movsd  %xmm1,0x1e88(%rsp)
  1c1f93:	movsd  %xmm2,0x1e80(%rsp)
  1c1f9c:	movsd  %xmm3,0x1e78(%rsp)
  1c1fa5:	movsd  %xmm4,0x1ec8(%rsp)
  1c1fae:	movsd  %xmm5,0x1eb0(%rsp)
  1c1fb7:	mov    %rax,0x1ec0(%rsp)
  1c1fbf:	mov    %rcx,0x1e70(%rsp)
  1c1fc7:	mov    %rbp,0x1ee8(%rsp)
  1c1fcf:	mov    %r12,0x1ef0(%rsp)
  1c1fd7:	xor    %r12d,%r12d
  1c1fda:	mov    %rbx,0x1f18(%rsp)
  1c1fe2:	mov    %rdi,%rbx
  1c1fe5:	mov    %r13,0x1f60(%rsp)
  1c1fed:	xor    %r13d,%r13d
  1c1ff0:	mov    0x2100(%rsp),%r15
  1c1ff8:	cmpl   $0x0,0x209c(%rsp)
  1c2000:	je     1c2029 <PRIVATE0000000000134a87+0x1fb9>
  1c2002:	mov    %r13,%rbp
  1c2005:	cmpl   $0x0,0x8(%rsp)
  1c200a:	jle    1c2029 <PRIVATE0000000000134a87+0x1fb9>
  1c200c:	mov    0x20(%rsp),%rax
  1c2011:	movl   $0x0,(%rax,%rbp,4)
  1c2018:	inc    %rbp
  1c201b:	movslq 0x8(%rsp),%rdx
  1c2020:	cmp    %rdx,%rbp
  1c2023:	jl     1c2011 <PRIVATE0000000000134a87+0x1fa1>
  1c2025:	mov    0xc(%rsp),%edx
  1c2029:	mov    0x1850(%rsp),%r10d
  1c2031:	movslq %r10d,%r10
  1c2034:	movslq 0x2c(%rsp),%rax
  1c2039:	lea    (%r10,%r10,2),%rbp
  1c203d:	shl    $0x5,%rbp
  1c2041:	cmp    0x184c(%rsp),%r10d
  1c2049:	jge    1c2074 <PRIVATE0000000000134a87+0x2004>
  1c204b:	mov    0x10(%rsp),%rdx
  1c2050:	mov    0x90(%rsp,%rbp,1),%r11d
  1c2058:	inc    %r10d
  1c205b:	mov    %r11d,(%rdx,%rax,4)
  1c205f:	add    $0x60,%rbp
  1c2063:	inc    %rax
  1c2066:	cmp    0x184c(%rsp),%r10d
  1c206e:	jl     1c2050 <PRIVATE0000000000134a87+0x1fe0>
  1c2070:	mov    0xc(%rsp),%edx
  1c2074:	movl   $0x0,0x1850(%rsp)
  1c207f:	cmpl   $0x0,0x1848(%rsp)
  1c2087:	jle    1c2094 <PRIVATE0000000000134a87+0x2024>
  1c2089:	movl   $0x0,0x184c(%rsp)
  1c2094:	mov    0x28(%rsp),%r10d
  1c2099:	movslq %r10d,%rbp
  1c209c:	cmp    %edx,%r10d
  1c209f:	jge    1c20be <PRIVATE0000000000134a87+0x204e>
  1c20a1:	mov    0x10(%rsp),%rdx
  1c20a6:	mov    (%rdx,%rbp,4),%r11d
  1c20aa:	inc    %r10d
  1c20ad:	mov    %r11d,(%rdx,%rax,4)
  1c20b1:	inc    %rbp
  1c20b4:	inc    %rax
  1c20b7:	cmp    0xc(%rsp),%r10d
  1c20bc:	jl     1c20a6 <PRIVATE0000000000134a87+0x2036>
  1c20be:	mov    %r14,%rsi
  1c20c1:	lea    0x8(%rsp),%rdi
  1c20c6:	mov    %r15,%r8
  1c20c9:	lea    0x2090(%rsp),%rdx
  1c20d1:	mov    %eax,0x4(%rdi)
  1c20d4:	lea    0x209c(%rsp),%rcx
  1c20dc:	mov    %r12d,0x24(%rdi)
  1c20e0:	xor    %ebp,%ebp
  1c20e2:	mov    %r12d,0x20(%rdi)
  1c20e6:	call   1c5310 <PRIVATE000000000025615c>
  1c20eb:	mov    %eax,%edx
  1c20ed:	movslq %edx,%rax
  1c20f0:	test   %edx,%edx
  1c20f2:	jl     1c4e2a <PRIVATE0000000000134a87+0x4dba>
  1c20f8:	mov    %ebp,0x1ea0(%rsp)
  1c20ff:	mov    %rbx,0x1f40(%rsp)
  1c2107:	mov    %rax,%rbx
  1c210a:	mov    0x1f30(%rsp),%rdx
  1c2112:	cmpq   $0x0,0x2e58(%rdx)
  1c211a:	je     1c212a <PRIVATE0000000000134a87+0x20ba>
  1c211c:	cmpq   $0x0,0x2e80(%rdx)
  1c2124:	je     1c4dcc <PRIVATE0000000000134a87+0x4d5c>
  1c212a:	mov    0x4b8(%r14),%rdx
  1c2131:	mov    %r15,%rsi
  1c2134:	mov    0x100(%rdx),%rdi
  1c213b:	call   88eed0 <PRIVATE0000000000a83b2f>
  1c2140:	test   %eax,%eax
  1c2142:	jne    1c4dbe <PRIVATE0000000000134a87+0x4d4e>
  1c2148:	mov    %r14,%rdi
  1c214b:	mov    %r15,%rsi
  1c214e:	call   11f430 <PRIVATE000000000008b129>
  1c2153:	test   %eax,%eax
  1c2155:	je     1c215e <PRIVATE0000000000134a87+0x20ee>
  1c2157:	mov    %eax,0xf0(%r14)
  1c215e:	jne    1c4dfd <PRIVATE0000000000134a87+0x4d8d>
  1c2164:	mov    %ebx,0x2fc(%r14)
  1c216b:	movl   $0x0,0x328(%r14)
  1c2176:	cmp    0x1f20(%rsp),%rbx
  1c217e:	jge    1c21bb <PRIVATE0000000000134a87+0x214b>
  1c2180:	mov    0x1fa8(%rsp),%rdx
  1c2188:	mov    0x1f88(%rsp),%r10
  1c2190:	mov    0x1f90(%rsp),%rbp
  1c2198:	movsd  (%rdx,%rbx,8),%xmm0
  1c219d:	movsd  (%r10,%rbx,8),%xmm1
  1c21a3:	movaps %xmm0,%xmm5
  1c21a6:	subsd  %xmm0,%xmm1
  1c21aa:	subsd  0x0(%rbp,%rbx,8),%xmm5
  1c21b0:	movsd  %xmm1,0x1ee0(%rsp)
  1c21b9:	jmp    1c21da <PRIVATE0000000000134a87+0x216a>
  1c21bb:	mov    0x1ec0(%rsp),%rdx
  1c21c3:	movsd  0x1fc8(%rsp),%xmm0
  1c21cc:	movsd  %xmm0,0x1ee0(%rsp)
  1c21d5:	movsd  (%rdx,%rbx,8),%xmm5
  1c21da:	mov    0x2020(%rsp),%rdx
  1c21e2:	xor    %eax,%eax
  1c21e4:	mov    0x2090(%rsp),%r9
  1c21ec:	pxor   %xmm1,%xmm1
  1c21f0:	cmpl   $0x0,0x1f28(%rsp)
  1c21f8:	movsd  (%rdx,%rbx,8),%xmm4
  1c21fd:	movslq (%r9),%r10
  1c2200:	je     1c266a <PRIVATE0000000000134a87+0x25fa>
  1c2206:	test   %r10,%r10
  1c2209:	jle    1c227c <PRIVATE0000000000134a87+0x220c>
  1c220b:	mov    0x8(%r9),%r11
  1c220f:	mov    0x10(%r9),%rbp
  1c2213:	mov    0x1fe8(%rsp),%r13
  1c221b:	mov    0x2018(%rsp),%rdx
  1c2223:	mov    0x2020(%rsp),%rcx
  1c222b:	movslq (%r11,%rax,4),%r12
  1c222f:	mov    %r12d,0x2098(%rsp)
  1c2237:	movsd  0x0(%rbp,%rax,8),%xmm2
  1c223d:	movslq 0x0(%r13,%r12,4),%r10
  1c2242:	movsd  (%rcx,%r10,8),%xmm0
  1c2248:	mulsd  %xmm2,%xmm0
  1c224c:	movsbl (%r12,%rdx,1),%r10d
  1c2251:	subsd  %xmm0,%xmm4
  1c2255:	cmp    $0x42,%r10d
  1c2259:	je     1c226d <PRIVATE0000000000134a87+0x21fd>
  1c225b:	movaps %xmm1,%xmm0
  1c225e:	cmp    $0x41,%r10d
  1c2262:	subsd  %xmm2,%xmm0
  1c2266:	jne    1c2271 <PRIVATE0000000000134a87+0x2201>
  1c2268:	movaps %xmm0,%xmm1
  1c226b:	jmp    1c2271 <PRIVATE0000000000134a87+0x2201>
  1c226d:	addsd  %xmm2,%xmm1
  1c2271:	inc    %rax
  1c2274:	movslq (%r9),%r10
  1c2277:	cmp    %r10,%rax
  1c227a:	jl     1c222b <PRIVATE0000000000134a87+0x21bb>
  1c227c:	comisd 0x1ec8(%rsp),%xmm4
  1c2285:	jbe    1c229a <PRIVATE0000000000134a87+0x222a>
  1c2287:	movsd  0x1ec8(%rsp),%xmm0
  1c2290:	comisd %xmm5,%xmm0
  1c2294:	ja     1c23d7 <PRIVATE0000000000134a87+0x2367>
  1c229a:	movsd  0x1eb0(%rsp),%xmm0
  1c22a3:	comisd %xmm4,%xmm0
  1c22a7:	jbe    1c22c1 <PRIVATE0000000000134a87+0x2251>
  1c22a9:	movsd  0x1ec8(%rsp),%xmm0
  1c22b2:	comisd 0x1ee0(%rsp),%xmm0
  1c22bb:	ja     1c24ee <PRIVATE0000000000134a87+0x247e>
  1c22c1:	movsd  0x289f07f(%rip),%xmm0        # 2a61348 <__STRLITPACK_7+0xa1c>
  1c22c9:	mulsd  %xmm5,%xmm0
  1c22cd:	movsd  0x1ee0(%rsp),%xmm2
  1c22d6:	comisd %xmm0,%xmm2
  1c22da:	jbe    1c2311 <PRIVATE0000000000134a87+0x22a1>
  1c22dc:	movaps %xmm5,%xmm3
  1c22df:	mulsd  %xmm1,%xmm3
  1c22e3:	xorps  0x289efa6(%rip),%xmm3        # 2a61290 <__STRLITPACK_7+0x964>
  1c22ea:	movsd  0x2080(%rsp),%xmm2
  1c22f3:	comisd %xmm3,%xmm2
  1c22f7:	jbe    1c2311 <PRIVATE0000000000134a87+0x22a1>
  1c22f9:	movaps %xmm4,%xmm3
  1c22fc:	mulsd  %xmm5,%xmm3
  1c2300:	xorps  0x289ef89(%rip),%xmm3        # 2a61290 <__STRLITPACK_7+0x964>
  1c2307:	comisd %xmm3,%xmm2
  1c230b:	ja     1c23d7 <PRIVATE0000000000134a87+0x2367>
  1c2311:	movsd  0x289f02f(%rip),%xmm6        # 2a61348 <__STRLITPACK_7+0xa1c>
  1c2319:	mulsd  0x1ee0(%rsp),%xmm6
  1c2322:	comisd %xmm6,%xmm5
  1c2326:	jbe    1c235b <PRIVATE0000000000134a87+0x22eb>
  1c2328:	movsd  0x1ee0(%rsp),%xmm2
  1c2331:	mulsd  %xmm1,%xmm2
  1c2335:	movsd  0x2080(%rsp),%xmm3
  1c233e:	comisd %xmm2,%xmm3
  1c2342:	jbe    1c235b <PRIVATE0000000000134a87+0x22eb>
  1c2344:	movsd  0x1ee0(%rsp),%xmm2
  1c234d:	mulsd  %xmm4,%xmm2
  1c2351:	comisd %xmm2,%xmm3
  1c2355:	ja     1c24ee <PRIVATE0000000000134a87+0x247e>
  1c235b:	movaps %xmm1,%xmm3
  1c235e:	andps  0x289ef3b(%rip),%xmm3        # 2a612a0 <__STRLITPACK_7+0x974>
  1c2365:	movsd  0x2080(%rsp),%xmm2
  1c236e:	comisd %xmm3,%xmm2
  1c2372:	jbe    1c23c9 <PRIVATE0000000000134a87+0x2359>
  1c2374:	movsd  0x1ee0(%rsp),%xmm1
  1c237d:	comisd %xmm0,%xmm1
  1c2381:	jbe    1c239a <PRIVATE0000000000134a87+0x232a>
  1c2383:	movaps %xmm4,%xmm1
  1c2386:	mulsd  %xmm5,%xmm1
  1c238a:	xorps  0x289eeff(%rip),%xmm1        # 2a61290 <__STRLITPACK_7+0x964>
  1c2391:	movaps %xmm2,%xmm0
  1c2394:	comisd %xmm1,%xmm0
  1c2398:	ja     1c23d7 <PRIVATE0000000000134a87+0x2367>
  1c239a:	comisd %xmm6,%xmm5
  1c239e:	jbe    1c23ba <PRIVATE0000000000134a87+0x234a>
  1c23a0:	movsd  0x1ee0(%rsp),%xmm0
  1c23a9:	mulsd  %xmm4,%xmm0
  1c23ad:	movaps %xmm2,%xmm1
  1c23b0:	comisd %xmm0,%xmm1
  1c23b4:	ja     1c24ee <PRIVATE0000000000134a87+0x247e>
  1c23ba:	pxor   %xmm0,%xmm0
  1c23be:	comisd %xmm0,%xmm4
  1c23c2:	ja     1c23d7 <PRIVATE0000000000134a87+0x2367>
  1c23c4:	jmp    1c24ee <PRIVATE0000000000134a87+0x247e>
  1c23c9:	pxor   %xmm0,%xmm0
  1c23cd:	comisd %xmm0,%xmm1
  1c23d1:	jbe    1c24ee <PRIVATE0000000000134a87+0x247e>
  1c23d7:	movl   $0x1,0x328(%r14)
  1c23e2:	xor    %r13b,%r13b
  1c23e5:	test   %r10,%r10
  1c23e8:	jle    1c24f1 <PRIVATE0000000000134a87+0x2481>
  1c23ee:	mov    0x10(%r9),%rbp
  1c23f2:	cmp    $0x8,%r10
  1c23f6:	jl     1c449f <PRIVATE0000000000134a87+0x442f>
  1c23fc:	mov    %rbp,%rax
  1c23ff:	and    $0xf,%rax
  1c2403:	mov    %eax,%edx
  1c2405:	test   %edx,%edx
  1c2407:	je     1c2417 <PRIVATE0000000000134a87+0x23a7>
  1c2409:	test   $0x7,%dl
  1c240c:	jne    1c449f <PRIVATE0000000000134a87+0x442f>
  1c2412:	mov    $0x1,%edx
  1c2417:	lea    0x8(%rdx),%r11d
  1c241b:	cmp    %r11,%r10
  1c241e:	jl     1c449f <PRIVATE0000000000134a87+0x442f>
  1c2424:	mov    %r10d,%eax
  1c2427:	mov    %eax,%r11d
  1c242a:	sub    %edx,%r11d
  1c242d:	and    $0x7,%r11d
  1c2431:	sub    %r11d,%eax
  1c2434:	xor    %r11d,%r11d
  1c2437:	mov    %edx,%r12d
  1c243a:	test   %edx,%edx
  1c243c:	jbe    1c245b <PRIVATE0000000000134a87+0x23eb>
  1c243e:	movsd  0x0(%rbp,%r11,8),%xmm0
  1c2445:	xorps  0x289ee44(%rip),%xmm0        # 2a61290 <__STRLITPACK_7+0x964>
  1c244c:	movsd  %xmm0,0x0(%rbp,%r11,8)
  1c2453:	inc    %r11
  1c2456:	cmp    %r12,%r11
  1c2459:	jb     1c243e <PRIVATE0000000000134a87+0x23ce>
  1c245b:	movslq %eax,%r11
  1c245e:	xchg   %ax,%ax
  1c2460:	movups 0x0(%rbp,%r12,8),%xmm0
  1c2466:	movups 0x10(%rbp,%r12,8),%xmm1
  1c246c:	movups 0x20(%rbp,%r12,8),%xmm2
  1c2472:	movups 0x30(%rbp,%r12,8),%xmm3
  1c2478:	xorps  0x289ee31(%rip),%xmm0        # 2a612b0 <__STRLITPACK_7+0x984>
  1c247f:	xorps  0x289ee2a(%rip),%xmm1        # 2a612b0 <__STRLITPACK_7+0x984>
  1c2486:	xorps  0x289ee23(%rip),%xmm2        # 2a612b0 <__STRLITPACK_7+0x984>
  1c248d:	xorps  0x289ee1c(%rip),%xmm3        # 2a612b0 <__STRLITPACK_7+0x984>
  1c2494:	movups %xmm0,0x0(%rbp,%r12,8)
  1c249a:	movups %xmm1,0x10(%rbp,%r12,8)
  1c24a0:	movups %xmm2,0x20(%rbp,%r12,8)
  1c24a6:	movups %xmm3,0x30(%rbp,%r12,8)
  1c24ac:	add    $0x8,%r12
  1c24b0:	cmp    %r11,%r12
  1c24b3:	jb     1c2460 <PRIVATE0000000000134a87+0x23f0>
  1c24b5:	xor    %r11d,%r11d
  1c24b8:	lea    0x1(%rax),%r12d
  1c24bc:	movslq %r12d,%r12
  1c24bf:	cmp    %r10,%r12
  1c24c2:	ja     1c24f1 <PRIVATE0000000000134a87+0x2481>
  1c24c4:	movslq %eax,%rax
  1c24c7:	sub    %rax,%r10
  1c24ca:	lea    0x0(%rbp,%rax,8),%rbp
  1c24cf:	movsd  0x0(%rbp),%xmm0
  1c24d4:	inc    %r11
  1c24d7:	xorps  0x289edb2(%rip),%xmm0        # 2a61290 <__STRLITPACK_7+0x964>
  1c24de:	movsd  %xmm0,0x0(%rbp)
  1c24e3:	add    $0x8,%rbp
  1c24e7:	cmp    %r10,%r11
  1c24ea:	jb     1c24cf <PRIVATE0000000000134a87+0x245f>
  1c24ec:	jmp    1c24f1 <PRIVATE0000000000134a87+0x2481>
  1c24ee:	mov    $0x1,%r13b
  1c24f1:	add    $0xffffffffffffffd0,%rsp
  1c24f5:	mov    %r14,%rdi
  1c24f8:	mov    0x2038(%rsp),%r10
  1c2500:	lea    0x1f28(%rsp),%r12
  1c2508:	mov    %r9,%rdx
  1c250b:	mov    0x8(%r10),%rbp
  1c250f:	mov    0x10(%r10),%r11
  1c2513:	mov    %rbp,(%rsp)
  1c2517:	lea    0x1ee8(%rsp),%rbp
  1c251f:	mov    %r11,0x8(%rsp)
  1c2524:	mov    %rbp,0x10(%rsp)
  1c2529:	mov    %r12,0x18(%rsp)
  1c252e:	mov    %r15,0x20(%rsp)
  1c2533:	movsd  0x110(%rbp),%xmm2
  1c253b:	movsd  0x1b8(%rbp),%xmm0
  1c2543:	movaps %xmm2,%xmm3
  1c2546:	movsd  0x1c0(%rbp),%xmm1
  1c254e:	mov    0x1b0(%rbp),%esi
  1c2554:	mov    0x158(%rbp),%rcx
  1c255b:	mov    0x138(%rbp),%r8
  1c2562:	mov    0x160(%rbp),%r9
  1c2569:	movsd  %xmm4,0x18(%rbp)
  1c256e:	movsd  %xmm5,0x20(%rbp)
  1c2573:	call   eefa0 <PRIVATE0000000000054eef>
  1c2578:	movsd  0x1f08(%rsp),%xmm5
  1c2581:	movsd  0x1f00(%rsp),%xmm4
  1c258a:	add    $0x30,%rsp
  1c258e:	test   %eax,%eax
  1c2590:	jl     1c25b4 <PRIVATE0000000000134a87+0x2544>
  1c2592:	movsd  0x1ef8(%rsp),%xmm1
  1c259b:	andps  0x289ecfe(%rip),%xmm1        # 2a612a0 <__STRLITPACK_7+0x974>
  1c25a2:	movsd  0x289ed46(%rip),%xmm0        # 2a612f0 <__STRLITPACK_7+0x9c4>
  1c25aa:	comisd %xmm1,%xmm0
  1c25ae:	jbe    1c2655 <PRIVATE0000000000134a87+0x25e5>
  1c25b4:	add    $0xffffffffffffffd0,%rsp
  1c25b8:	mov    %r14,%rdi
  1c25bb:	mov    0x20c0(%rsp),%rdx
  1c25c3:	mov    %rbp,(%rsp)
  1c25c7:	mov    %r12,0x8(%rsp)
  1c25cc:	mov    0x1f20(%rsp),%r10
  1c25d4:	mov    %r10,0x10(%rsp)
  1c25d9:	mov    0x1f18(%rsp),%r11
  1c25e1:	mov    %r11,0x18(%rsp)
  1c25e6:	mov    %r15,0x20(%rsp)
  1c25eb:	movsd  0x1ff8(%rsp),%xmm2
  1c25f4:	movsd  0x20a0(%rsp),%xmm0
  1c25fd:	movaps %xmm2,%xmm3
  1c2600:	movsd  0x20a8(%rsp),%xmm1
  1c2609:	mov    0x2098(%rsp),%esi
  1c2610:	mov    0x2040(%rsp),%rcx
  1c2618:	mov    0x2020(%rsp),%r8
  1c2620:	mov    0x2048(%rsp),%r9
  1c2628:	movsd  %xmm4,0x1f00(%rsp)
  1c2631:	movsd  %xmm5,0x1f08(%rsp)
  1c263a:	call   f0120 <PRIVATE0000000000056973>
  1c263f:	movsd  0x1f08(%rsp),%xmm5
  1c2648:	movsd  0x1f00(%rsp),%xmm4
  1c2651:	add    $0x30,%rsp
  1c2655:	mov    %eax,0x2f8(%r14)
  1c265c:	movsd  0x1eb8(%rsp),%xmm0
  1c2665:	jmp    1c28ee <PRIVATE0000000000134a87+0x287e>
  1c266a:	test   %r10,%r10
  1c266d:	jle    1c26ab <PRIVATE0000000000134a87+0x263b>
  1c266f:	mov    0x8(%r9),%r11
  1c2673:	mov    0x10(%r9),%rbp
  1c2677:	mov    0x1fe8(%rsp),%r13
  1c267f:	movslq (%r11,%rax,4),%r10
  1c2683:	mov    %r10d,0x2098(%rsp)
  1c268b:	movsd  0x0(%rbp,%rax,8),%xmm0
  1c2691:	inc    %rax
  1c2694:	movslq 0x0(%r13,%r10,4),%r12
  1c2699:	movslq (%r9),%r10
  1c269c:	mulsd  (%rdx,%r12,8),%xmm0
  1c26a2:	subsd  %xmm0,%xmm4
  1c26a6:	cmp    %r10,%rax
  1c26a9:	jl     1c267f <PRIVATE0000000000134a87+0x260f>
  1c26ab:	movsd  0x289ec95(%rip),%xmm1        # 2a61348 <__STRLITPACK_7+0xa1c>
  1c26b3:	mulsd  %xmm5,%xmm1
  1c26b7:	movsd  0x1ee0(%rsp),%xmm0
  1c26c0:	comisd %xmm1,%xmm0
  1c26c4:	jbe    1c26e3 <PRIVATE0000000000134a87+0x2673>
  1c26c6:	movaps %xmm4,%xmm1
  1c26c9:	mulsd  %xmm5,%xmm1
  1c26cd:	xorps  0x289ebbc(%rip),%xmm1        # 2a61290 <__STRLITPACK_7+0x964>
  1c26d4:	movsd  0x2080(%rsp),%xmm0
  1c26dd:	comisd %xmm1,%xmm0
  1c26e1:	ja     1c2728 <PRIVATE0000000000134a87+0x26b8>
  1c26e3:	movsd  0x289ec5d(%rip),%xmm0        # 2a61348 <__STRLITPACK_7+0xa1c>
  1c26eb:	mulsd  0x1ee0(%rsp),%xmm0
  1c26f4:	comisd %xmm0,%xmm5
  1c26f8:	jbe    1c271a <PRIVATE0000000000134a87+0x26aa>
  1c26fa:	movsd  0x1ee0(%rsp),%xmm0
  1c2703:	mulsd  %xmm4,%xmm0
  1c2707:	movsd  0x2080(%rsp),%xmm1
  1c2710:	comisd %xmm0,%xmm1
  1c2714:	ja     1c283e <PRIVATE0000000000134a87+0x27ce>
  1c271a:	pxor   %xmm0,%xmm0
  1c271e:	comisd %xmm0,%xmm4
  1c2722:	jbe    1c283e <PRIVATE0000000000134a87+0x27ce>
  1c2728:	movl   $0x1,0x328(%r14)
  1c2733:	xor    %r13b,%r13b
  1c2736:	test   %r10,%r10
  1c2739:	jle    1c2841 <PRIVATE0000000000134a87+0x27d1>
  1c273f:	mov    0x10(%r9),%rbp
  1c2743:	cmp    $0x8,%r10
  1c2747:	jl     1c44a6 <PRIVATE0000000000134a87+0x4436>
  1c274d:	mov    %rbp,%rax
  1c2750:	and    $0xf,%rax
  1c2754:	mov    %eax,%edx
  1c2756:	test   %edx,%edx
  1c2758:	je     1c2768 <PRIVATE0000000000134a87+0x26f8>
  1c275a:	test   $0x7,%dl
  1c275d:	jne    1c44a6 <PRIVATE0000000000134a87+0x4436>
  1c2763:	mov    $0x1,%edx
  1c2768:	lea    0x8(%rdx),%r11d
  1c276c:	cmp    %r11,%r10
  1c276f:	jl     1c44a6 <PRIVATE0000000000134a87+0x4436>
  1c2775:	mov    %r10d,%eax
  1c2778:	mov    %eax,%r11d
  1c277b:	sub    %edx,%r11d
  1c277e:	and    $0x7,%r11d
  1c2782:	sub    %r11d,%eax
  1c2785:	xor    %r11d,%r11d
  1c2788:	mov    %edx,%r12d
  1c278b:	test   %edx,%edx
  1c278d:	jbe    1c27ac <PRIVATE0000000000134a87+0x273c>
  1c278f:	movsd  0x0(%rbp,%r11,8),%xmm0
  1c2796:	xorps  0x289eaf3(%rip),%xmm0        # 2a61290 <__STRLITPACK_7+0x964>
  1c279d:	movsd  %xmm0,0x0(%rbp,%r11,8)
  1c27a4:	inc    %r11
  1c27a7:	cmp    %r12,%r11
  1c27aa:	jb     1c278f <PRIVATE0000000000134a87+0x271f>
  1c27ac:	movslq %eax,%r11
  1c27af:	nop
  1c27b0:	movups 0x0(%rbp,%r12,8),%xmm0
  1c27b6:	movups 0x10(%rbp,%r12,8),%xmm1
  1c27bc:	movups 0x20(%rbp,%r12,8),%xmm2
  1c27c2:	movups 0x30(%rbp,%r12,8),%xmm3
  1c27c8:	xorps  0x289eae1(%rip),%xmm0        # 2a612b0 <__STRLITPACK_7+0x984>
  1c27cf:	xorps  0x289eada(%rip),%xmm1        # 2a612b0 <__STRLITPACK_7+0x984>
  1c27d6:	xorps  0x289ead3(%rip),%xmm2        # 2a612b0 <__STRLITPACK_7+0x984>
  1c27dd:	xorps  0x289eacc(%rip),%xmm3        # 2a612b0 <__STRLITPACK_7+0x984>
  1c27e4:	movups %xmm0,0x0(%rbp,%r12,8)
  1c27ea:	movups %xmm1,0x10(%rbp,%r12,8)
  1c27f0:	movups %xmm2,0x20(%rbp,%r12,8)
  1c27f6:	movups %xmm3,0x30(%rbp,%r12,8)
  1c27fc:	add    $0x8,%r12
  1c2800:	cmp    %r11,%r12
  1c2803:	jb     1c27b0 <PRIVATE0000000000134a87+0x2740>
  1c2805:	xor    %r11d,%r11d
  1c2808:	lea    0x1(%rax),%r12d
  1c280c:	movslq %r12d,%r12
  1c280f:	cmp    %r10,%r12
  1c2812:	ja     1c2841 <PRIVATE0000000000134a87+0x27d1>
  1c2814:	movslq %eax,%rax
  1c2817:	sub    %rax,%r10
  1c281a:	lea    0x0(%rbp,%rax,8),%rbp
  1c281f:	movsd  0x0(%rbp),%xmm0
  1c2824:	inc    %r11
  1c2827:	xorps  0x289ea62(%rip),%xmm0        # 2a61290 <__STRLITPACK_7+0x964>
  1c282e:	movsd  %xmm0,0x0(%rbp)
  1c2833:	add    $0x8,%rbp
  1c2837:	cmp    %r10,%r11
  1c283a:	jb     1c281f <PRIVATE0000000000134a87+0x27af>
  1c283c:	jmp    1c2841 <PRIVATE0000000000134a87+0x27d1>
  1c283e:	mov    $0x1,%r13b
  1c2841:	mov    %r14,%rdi
  1c2844:	lea    0x1ef8(%rsp),%r12
  1c284c:	xor    %edx,%edx
  1c284e:	lea    0x1eb8(%rsp),%rbp
  1c2856:	push   %r15
  1c2858:	push   %r12
  1c285a:	push   %rbp
  1c285b:	mov    0x150(%rbp),%r10
  1c2862:	push   0x10(%r10)
  1c2866:	push   0x8(%r10)
  1c286a:	push   0x2040(%rsp)
  1c2871:	push   0x2020(%rsp)
  1c2878:	push   0x2048(%rsp)
  1c287f:	movsd  %xmm4,0x18(%rbp)
  1c2884:	movsd  0x110(%rbp),%xmm2
  1c288c:	movsd  0x1b8(%rbp),%xmm0
  1c2894:	movaps %xmm2,%xmm3
  1c2897:	movsd  0x1c0(%rbp),%xmm1
  1c289f:	mov    0x1b0(%rbp),%esi
  1c28a5:	mov    0x180(%rbp),%rcx
  1c28ac:	mov    0x130(%rbp),%r8
  1c28b3:	movsd  %xmm5,0x20(%rbp)
  1c28b8:	call   ef960 <PRIVATE0000000000055f68>
  1c28bd:	movsd  0x1f18(%rsp),%xmm5
  1c28c6:	movsd  0x1f10(%rsp),%xmm4
  1c28cf:	add    $0x40,%rsp
  1c28d3:	cmp    $0xffffffff,%eax
  1c28d6:	jne    1c2655 <PRIVATE0000000000134a87+0x25e5>
  1c28dc:	movsd  0x1fc8(%rsp),%xmm0
  1c28e5:	movsd  %xmm0,0x1eb8(%rsp)
  1c28ee:	ucomisd 0x1fc8(%rsp),%xmm0
  1c28f7:	jp     1c28ff <PRIVATE0000000000134a87+0x288f>
  1c28f9:	je     1c497e <PRIVATE0000000000134a87+0x490e>
  1c28ff:	mov    0x328(%r14),%ecx
  1c2906:	mov    0x2f8(%r14),%esi
  1c290d:	test   %ecx,%ecx
  1c290f:	je     1c2924 <PRIVATE0000000000134a87+0x28b4>
  1c2911:	movaps %xmm5,%xmm1
  1c2914:	subsd  0x289e9ec(%rip),%xmm1        # 2a61308 <__STRLITPACK_7+0x9dc>
  1c291c:	comisd %xmm1,%xmm0
  1c2920:	jae    1c293b <PRIVATE0000000000134a87+0x28cb>
  1c2922:	jmp    1c2948 <PRIVATE0000000000134a87+0x28d8>
  1c2924:	movsd  0x1ee0(%rsp),%xmm1
  1c292d:	subsd  0x289e9d3(%rip),%xmm1        # 2a61308 <__STRLITPACK_7+0x9dc>
  1c2935:	comisd %xmm1,%xmm0
  1c2939:	jb     1c2948 <PRIVATE0000000000134a87+0x28d8>
  1c293b:	movl   $0xffffffff,0x2f8(%r14)
  1c2946:	jmp    1c2994 <PRIVATE0000000000134a87+0x2924>
  1c2948:	test   %esi,%esi
  1c294a:	jl     1c2994 <PRIVATE0000000000134a87+0x2924>
  1c294c:	test   %r13b,%r13b
  1c294f:	jne    1c2a72 <PRIVATE0000000000134a87+0x2a02>
  1c2955:	comisd 0x1ec8(%rsp),%xmm4
  1c295e:	jbe    1c2a72 <PRIVATE0000000000134a87+0x2a02>
  1c2964:	movsd  0x1ec8(%rsp),%xmm1
  1c296d:	comisd %xmm5,%xmm1
  1c2971:	jbe    1c2a72 <PRIVATE0000000000134a87+0x2a02>
  1c2977:	movsd  0x1fc8(%rsp),%xmm0
  1c2980:	movsd  %xmm0,0x1eb8(%rsp)
  1c2989:	movl   $0xffffffff,0x2f8(%r14)
  1c2994:	movsd  0x2a0(%r14),%xmm0
  1c299d:	movslq 0x2fc(%r14),%r10
  1c29a4:	mov    0x1f90(%rsp),%rdx
  1c29ac:	movsd  0x1f00(%rsp),%xmm1
  1c29b5:	incl   0x1ea0(%rsp)
  1c29bc:	addsd  0x289e90c(%rip),%xmm0        # 2a612d0 <__STRLITPACK_7+0x9a4>
  1c29c4:	movsd  %xmm0,0x2a0(%r14)
  1c29cd:	comisd (%rdx,%r10,8),%xmm1
  1c29d3:	jb     1c2a2b <PRIVATE0000000000134a87+0x29bb>
  1c29d5:	mov    0x1f88(%rsp),%rdx
  1c29dd:	movsd  (%rdx,%r10,8),%xmm0
  1c29e3:	comisd 0x1fc8(%rsp),%xmm0
  1c29ec:	jb     1c2a2b <PRIVATE0000000000134a87+0x29bb>
  1c29ee:	mov    0x1fa8(%rsp),%rdx
  1c29f6:	test   %ecx,%ecx
  1c29f8:	mov    0x1fc0(%rsp),%rbp
  1c2a00:	mov    0x2090(%rsp),%rax
  1c2a08:	movsd  (%rdx,%r10,8),%xmm5
  1c2a0e:	movaps %xmm5,%xmm0
  1c2a11:	xorps  0x289e878(%rip),%xmm0        # 2a61290 <__STRLITPACK_7+0x964>
  1c2a18:	jne    1c2a1d <PRIVATE0000000000134a87+0x29ad>
  1c2a1a:	movaps %xmm0,%xmm5
  1c2a1d:	movl   $0xfffffffd,0x0(%rbp,%r10,4)
  1c2a26:	jmp    1c3004 <PRIVATE0000000000134a87+0x2f94>
  1c2a2b:	test   %ecx,%ecx
  1c2a2d:	je     1c2a4c <PRIVATE0000000000134a87+0x29dc>
  1c2a2f:	mov    0x1fc0(%rsp),%rdx
  1c2a37:	mov    0x2090(%rsp),%rax
  1c2a3f:	movl   $0xffffffff,(%rdx,%r10,4)
  1c2a47:	jmp    1c3004 <PRIVATE0000000000134a87+0x2f94>
  1c2a4c:	mov    0x1fc0(%rsp),%rdx
  1c2a54:	movsd  0x1ee0(%rsp),%xmm5
  1c2a5d:	mov    0x2090(%rsp),%rax
  1c2a65:	movl   $0xfffffffe,(%rdx,%r10,4)
  1c2a6d:	jmp    1c3004 <PRIVATE0000000000134a87+0x2f94>
  1c2a72:	cmp    $0x1,%r13b
  1c2a76:	je     1c494e <PRIVATE0000000000134a87+0x48de>
  1c2a7c:	test   %esi,%esi
  1c2a7e:	jl     1c2994 <PRIVATE0000000000134a87+0x2924>
  1c2a84:	movsd  0x1ef8(%rsp),%xmm2
  1c2a8d:	movsd  0x289e86b(%rip),%xmm1        # 2a61300 <__STRLITPACK_7+0x9d4>
  1c2a95:	comisd %xmm2,%xmm1
  1c2a99:	ja     1c3b7b <PRIVATE0000000000134a87+0x3b0b>
  1c2a9f:	mulsd  %xmm2,%xmm0
  1c2aa3:	movsd  0x289e945(%rip),%xmm1        # 2a613f0 <__STRLITPACK_7+0xac4>
  1c2aab:	comisd %xmm0,%xmm1
  1c2aaf:	ja     1c3b7b <PRIVATE0000000000134a87+0x3b0b>
  1c2ab5:	movsd  0x2a0(%r14),%xmm0
  1c2abe:	mov    0x2090(%rsp),%rax
  1c2ac6:	movslq %esi,%r10
  1c2ac9:	addsd  0x289e7ff(%rip),%xmm0        # 2a612d0 <__STRLITPACK_7+0x9a4>
  1c2ad1:	mov    0x10(%rax),%rbp
  1c2ad5:	mov    0x8(%rax),%r11
  1c2ad9:	movsd  %xmm0,0x2a0(%r14)
  1c2ae2:	mov    %esi,0x2098(%rsp)
  1c2ae9:	movslq 0x1850(%rsp),%r13
  1c2af1:	mov    %r13,%r12
  1c2af4:	mov    0x0(%rbp,%r10,8),%rdi
  1c2af9:	mov    (%r11,%r10,4),%edx
  1c2afd:	incl   0x1ea0(%rsp)
  1c2b04:	mov    %rdi,0x1ef8(%rsp)
  1c2b0c:	mov    %edx,0x2f8(%r14)
  1c2b13:	dec    %r12
  1c2b16:	jl     1c2b39 <PRIVATE0000000000134a87+0x2ac9>
  1c2b18:	lea    -0x3(%r13,%r13,2),%r12
  1c2b1d:	shl    $0x5,%r12
  1c2b21:	mov    %edx,0x94(%rsp,%r12,1)
  1c2b29:	mov    %rdi,0x98(%rsp,%r12,1)
  1c2b31:	mov    %ecx,0xa0(%rsp,%r12,1)
  1c2b39:	mov    (%rax),%r13d
  1c2b3c:	lea    -0x1(%r13),%r12d
  1c2b40:	cmp    %r12d,%esi
  1c2b43:	je     1c2b6c <PRIVATE0000000000134a87+0x2afc>
  1c2b45:	movslq %r12d,%r12
  1c2b48:	mov    (%r11,%r12,4),%edx
  1c2b4c:	mov    %edx,(%r11,%r10,4)
  1c2b50:	movslq 0x2098(%rsp),%r11
  1c2b58:	mov    0x0(%rbp,%r12,8),%r10
  1c2b5d:	mov    (%rax),%r13d
  1c2b60:	mov    %r10,0x0(%rbp,%r11,8)
  1c2b65:	mov    0x2f8(%r14),%edx
  1c2b6c:	movslq 0x2fc(%r14),%r10
  1c2b73:	dec    %r13d
  1c2b76:	mov    0x1fc0(%rsp),%rbp
  1c2b7e:	mov    %r13d,(%rax)
  1c2b81:	mov    %edx,0x2098(%rsp)
  1c2b88:	mov    %edx,0x0(%rbp,%r10,4)
  1c2b8d:	movslq 0x2098(%rsp),%r13
  1c2b95:	mov    0x1fe8(%rsp),%r12
  1c2b9d:	mov    0x2fc(%r14),%r11d
  1c2ba4:	mov    0x1f88(%rsp),%rbp
  1c2bac:	mov    (%r12,%r13,4),%edx
  1c2bb0:	mov    %edx,0x32c(%r14)
  1c2bb7:	mov    %r11d,(%r12,%r13,4)
  1c2bbb:	movslq 0x32c(%r14),%rdx
  1c2bc2:	movsd  0x2088(%rsp),%xmm0
  1c2bcb:	movsd  0x0(%rbp,%rdx,8),%xmm1
  1c2bd1:	mov    0x1f90(%rsp),%rbp
  1c2bd9:	subsd  0x0(%rbp,%rdx,8),%xmm1
  1c2bdf:	comisd %xmm1,%xmm0
  1c2be3:	jae    1c2d77 <PRIVATE0000000000134a87+0x2d07>
  1c2be9:	movsd  0x1ef8(%rsp),%xmm2
  1c2bf2:	pxor   %xmm0,%xmm0
  1c2bf6:	comisd %xmm2,%xmm0
  1c2bfa:	jbe    1c2cbe <PRIVATE0000000000134a87+0x2c4e>
  1c2c00:	movslq 0x2f8(%r14),%r10
  1c2c07:	mov    0x2018(%rsp),%rbp
  1c2c0f:	mov    (%r10,%rbp,1),%bpl
  1c2c13:	cmp    $0x4e,%bpl
  1c2c17:	je     1c2c69 <PRIVATE0000000000134a87+0x2bf9>
  1c2c19:	cmp    $0x55,%bpl
  1c2c1d:	je     1c2c69 <PRIVATE0000000000134a87+0x2bf9>
  1c2c1f:	cmp    $0x42,%bpl
  1c2c23:	je     1c2c7e <PRIVATE0000000000134a87+0x2c0e>
  1c2c25:	cmp    $0x58,%bpl
  1c2c29:	je     1c2d77 <PRIVATE0000000000134a87+0x2d07>
  1c2c2f:	cmp    $0x4c,%bpl
  1c2c33:	jne    1c2c54 <PRIVATE0000000000134a87+0x2be4>
  1c2c35:	andps  0x289e664(%rip),%xmm2        # 2a612a0 <__STRLITPACK_7+0x974>
  1c2c3c:	mulsd  0x1eb8(%rsp),%xmm2
  1c2c45:	movsd  0x1e88(%rsp),%xmm0
  1c2c4e:	comisd %xmm2,%xmm0
  1c2c52:	jbe    1c2c69 <PRIVATE0000000000134a87+0x2bf9>
  1c2c54:	mov    0x1fc0(%rsp),%rbp
  1c2c5c:	movl   $0xffffffff,0x0(%rbp,%rdx,4)
  1c2c64:	jmp    1c2d87 <PRIVATE0000000000134a87+0x2d17>
  1c2c69:	mov    0x1fc0(%rsp),%rbp
  1c2c71:	movl   $0xfffffffe,0x0(%rbp,%rdx,4)
  1c2c79:	jmp    1c2d87 <PRIVATE0000000000134a87+0x2d17>
  1c2c7e:	mov    0x2010(%rsp),%rbp
  1c2c86:	movsd  0x1eb8(%rsp),%xmm0
  1c2c8f:	movsd  0x0(%rbp,%r10,8),%xmm1
  1c2c96:	divsd  %xmm2,%xmm1
  1c2c9a:	subsd  %xmm0,%xmm1
  1c2c9e:	addsd  0x289e62a(%rip),%xmm0        # 2a612d0 <__STRLITPACK_7+0x9a4>
  1c2ca6:	mulsd  0x1e88(%rsp),%xmm0
  1c2caf:	andps  0x289e5ea(%rip),%xmm1        # 2a612a0 <__STRLITPACK_7+0x974>
  1c2cb6:	comisd %xmm1,%xmm0
  1c2cba:	ja     1c2c54 <PRIVATE0000000000134a87+0x2be4>
  1c2cbc:	jmp    1c2c69 <PRIVATE0000000000134a87+0x2bf9>
  1c2cbe:	movslq 0x2f8(%r14),%r11
  1c2cc5:	mov    0x2018(%rsp),%rbp
  1c2ccd:	movsbl (%r11,%rbp,1),%ebp
  1c2cd2:	mov    %ebp,%r10d
  1c2cd5:	and    $0xfffffffd,%r10d
  1c2cd9:	cmp    $0x4c,%r10d
  1c2cdd:	je     1c2c54 <PRIVATE0000000000134a87+0x2be4>
  1c2ce3:	cmp    $0x58,%ebp
  1c2ce6:	je     1c2d77 <PRIVATE0000000000134a87+0x2d07>
  1c2cec:	cmp    $0x41,%ebp
  1c2cef:	jne    1c2d46 <PRIVATE0000000000134a87+0x2cd6>
  1c2cf1:	mov    0x2010(%rsp),%rbp
  1c2cf9:	mov    0x1ff0(%rsp),%r10
  1c2d01:	movsd  0x1eb8(%rsp),%xmm0
  1c2d0a:	movsd  0x0(%rbp,%r11,8),%xmm1
  1c2d11:	subsd  (%r10,%r11,8),%xmm1
  1c2d17:	divsd  %xmm2,%xmm1
  1c2d1b:	subsd  %xmm0,%xmm1
  1c2d1f:	addsd  0x289e5a9(%rip),%xmm0        # 2a612d0 <__STRLITPACK_7+0x9a4>
  1c2d27:	mulsd  0x1e88(%rsp),%xmm0
  1c2d30:	andps  0x289e569(%rip),%xmm1        # 2a612a0 <__STRLITPACK_7+0x974>
  1c2d37:	comisd %xmm1,%xmm0
  1c2d3b:	ja     1c2c69 <PRIVATE0000000000134a87+0x2bf9>
  1c2d41:	jmp    1c2c54 <PRIVATE0000000000134a87+0x2be4>
  1c2d46:	cmp    $0x55,%ebp
  1c2d49:	jne    1c2c54 <PRIVATE0000000000134a87+0x2be4>
  1c2d4f:	andps  0x289e54a(%rip),%xmm2        # 2a612a0 <__STRLITPACK_7+0x974>
  1c2d56:	mulsd  0x1eb8(%rsp),%xmm2
  1c2d5f:	movsd  0x1e88(%rsp),%xmm0
  1c2d68:	comisd %xmm2,%xmm0
  1c2d6c:	ja     1c2c69 <PRIVATE0000000000134a87+0x2bf9>
  1c2d72:	jmp    1c2c54 <PRIVATE0000000000134a87+0x2be4>
  1c2d77:	mov    0x1fc0(%rsp),%rbp
  1c2d7f:	movl   $0xfffffffc,0x0(%rbp,%rdx,4)
  1c2d87:	movslq 0x2fc(%r14),%r10
  1c2d8e:	mov    0x1f88(%rsp),%rdx
  1c2d96:	movslq 0x2f8(%r14),%r11
  1c2d9d:	movsd  (%rdx,%r10,8),%xmm0
  1c2da3:	comisd 0x1fc8(%rsp),%xmm0
  1c2dac:	jb     1c2dc7 <PRIVATE0000000000134a87+0x2d57>
  1c2dae:	mov    0x1ff0(%rsp),%rdx
  1c2db6:	movsd  0x1fc8(%rsp),%xmm0
  1c2dbf:	movsd  %xmm0,(%rdx,%r11,8)
  1c2dc5:	jmp    1c2de4 <PRIVATE0000000000134a87+0x2d74>
  1c2dc7:	mov    0x1f90(%rsp),%rdx
  1c2dcf:	mov    0x1ff0(%rsp),%rbp
  1c2dd7:	subsd  (%rdx,%r10,8),%xmm0
  1c2ddd:	movsd  %xmm0,0x0(%rbp,%r11,8)
  1c2de4:	mov    0x328(%r14),%ebp
  1c2deb:	test   %ebp,%ebp
  1c2ded:	je     1c2e0a <PRIVATE0000000000134a87+0x2d9a>
  1c2def:	movsd  0x1ef8(%rsp),%xmm0
  1c2df8:	xorps  0x289e491(%rip),%xmm0        # 2a61290 <__STRLITPACK_7+0x964>
  1c2dff:	movsd  %xmm0,0x1ef8(%rsp)
  1c2e08:	jmp    1c2e13 <PRIVATE0000000000134a87+0x2da3>
  1c2e0a:	movsd  0x1ef8(%rsp),%xmm0
  1c2e13:	movslq 0x2098(%rsp),%rdx
  1c2e1b:	mov    0x2018(%rsp),%r11
  1c2e23:	mov    (%rdx,%r11,1),%r11b
  1c2e27:	cmp    $0x42,%r11b
  1c2e2b:	je     1c2e33 <PRIVATE0000000000134a87+0x2dc3>
  1c2e2d:	cmp    $0x41,%r11b
  1c2e31:	jne    1c2e3a <PRIVATE0000000000134a87+0x2dca>
  1c2e33:	decl   0x1f28(%rsp)
  1c2e3a:	movslq 0x32c(%r14),%rcx
  1c2e41:	mov    0x1fc0(%rsp),%r11
  1c2e49:	cmpl   $0xffffffff,(%r11,%rcx,4)
  1c2e4e:	je     1c2e81 <PRIVATE0000000000134a87+0x2e11>
  1c2e50:	mov    0x2010(%rsp),%r11
  1c2e58:	mov    0x1f88(%rsp),%r12
  1c2e60:	mov    0x1f90(%rsp),%r13
  1c2e68:	movsd  (%r11,%rdx,8),%xmm2
  1c2e6e:	subsd  (%r12,%rcx,8),%xmm2
  1c2e74:	addsd  0x0(%r13,%rcx,8),%xmm2
  1c2e7b:	divsd  %xmm0,%xmm2
  1c2e7f:	jmp    1c2e93 <PRIVATE0000000000134a87+0x2e23>
  1c2e81:	mov    0x2010(%rsp),%r11
  1c2e89:	movsd  (%r11,%rdx,8),%xmm2
  1c2e8f:	divsd  %xmm0,%xmm2
  1c2e93:	movaps %xmm2,%xmm5
  1c2e96:	test   %ebp,%ebp
  1c2e98:	xorps  0x289e3f1(%rip),%xmm5        # 2a61290 <__STRLITPACK_7+0x964>
  1c2e9f:	movsd  %xmm2,(%r11,%rdx,8)
  1c2ea5:	jne    1c2eaa <PRIVATE0000000000134a87+0x2e3a>
  1c2ea7:	movaps %xmm2,%xmm5
  1c2eaa:	cmp    0x1f20(%rsp),%rbx
  1c2eb2:	jl     1c2eca <PRIVATE0000000000134a87+0x2e5a>
  1c2eb4:	mov    0x1ec0(%rsp),%rbp
  1c2ebc:	addsd  0x0(%rbp,%rbx,8),%xmm2
  1c2ec2:	movsd  %xmm2,(%r11,%rdx,8)
  1c2ec8:	jmp    1c2f04 <PRIVATE0000000000134a87+0x2e94>
  1c2eca:	mov    0x1fa8(%rsp),%rbp
  1c2ed2:	mov    0x1f90(%rsp),%r12
  1c2eda:	addsd  0x0(%rbp,%rbx,8),%xmm2
  1c2ee0:	movsd  %xmm2,(%r11,%rdx,8)
  1c2ee6:	movsd  (%r12,%rbx,8),%xmm0
  1c2eec:	comisd 0x1f00(%rsp),%xmm0
  1c2ef5:	jbe    1c2f04 <PRIVATE0000000000134a87+0x2e94>
  1c2ef7:	mov    %r11,%rbp
  1c2efa:	subsd  %xmm0,%xmm2
  1c2efe:	movsd  %xmm2,0x0(%rbp,%rdx,8)
  1c2f04:	mov    0x1f90(%rsp),%rbp
  1c2f0c:	movsd  0x1f00(%rsp),%xmm0
  1c2f15:	comisd 0x0(%rbp,%r10,8),%xmm0
  1c2f1c:	jb     1c2f2f <PRIVATE0000000000134a87+0x2ebf>
  1c2f1e:	mov    0x2018(%rsp),%rbp
  1c2f26:	movb   $0x46,(%rdx,%rbp,1)
  1c2f2a:	jmp    1c3004 <PRIVATE0000000000134a87+0x2f94>
  1c2f2f:	movsd  0x1f08(%rsp),%xmm0
  1c2f38:	comisd %xmm2,%xmm0
  1c2f3c:	jbe    1c2f56 <PRIVATE0000000000134a87+0x2ee6>
  1c2f3e:	mov    0x2018(%rsp),%rbp
  1c2f46:	incl   0x1f28(%rsp)
  1c2f4d:	movb   $0x42,(%rdx,%rbp,1)
  1c2f51:	jmp    1c3004 <PRIVATE0000000000134a87+0x2f94>
  1c2f56:	movsd  0x2070(%rsp),%xmm0
  1c2f5f:	comisd %xmm2,%xmm0
  1c2f63:	jbe    1c2f9b <PRIVATE0000000000134a87+0x2f2b>
  1c2f65:	mov    0x1ff0(%rsp),%rbp
  1c2f6d:	pxor   %xmm1,%xmm1
  1c2f71:	movsd  0x0(%rbp,%rdx,8),%xmm0
  1c2f77:	ucomisd %xmm1,%xmm0
  1c2f7b:	jp     1c2f7f <PRIVATE0000000000134a87+0x2f0f>
  1c2f7d:	je     1c2f8d <PRIVATE0000000000134a87+0x2f1d>
  1c2f7f:	mov    0x2018(%rsp),%rbp
  1c2f87:	movb   $0x4c,(%rdx,%rbp,1)
  1c2f8b:	jmp    1c3004 <PRIVATE0000000000134a87+0x2f94>
  1c2f8d:	mov    0x2018(%rsp),%rbp
  1c2f95:	movb   $0x58,(%rdx,%rbp,1)
  1c2f99:	jmp    1c3004 <PRIVATE0000000000134a87+0x2f94>
  1c2f9b:	mov    0x1ff0(%rsp),%rbp
  1c2fa3:	movsd  0x0(%rbp,%rdx,8),%xmm1
  1c2fa9:	movaps %xmm1,%xmm0
  1c2fac:	subsd  0x2070(%rsp),%xmm0
  1c2fb5:	comisd %xmm2,%xmm0
  1c2fb9:	ja     1c2fc6 <PRIVATE0000000000134a87+0x2f56>
  1c2fbb:	comisd 0x1fc8(%rsp),%xmm1
  1c2fc4:	jb     1c2fd4 <PRIVATE0000000000134a87+0x2f64>
  1c2fc6:	mov    0x2018(%rsp),%rbp
  1c2fce:	movb   $0x4e,(%rdx,%rbp,1)
  1c2fd2:	jmp    1c3004 <PRIVATE0000000000134a87+0x2f94>
  1c2fd4:	addsd  0x2070(%rsp),%xmm1
  1c2fdd:	comisd %xmm2,%xmm1
  1c2fe1:	jbe    1c2ff1 <PRIVATE0000000000134a87+0x2f81>
  1c2fe3:	mov    0x2018(%rsp),%rbp
  1c2feb:	movb   $0x55,(%rdx,%rbp,1)
  1c2fef:	jmp    1c3004 <PRIVATE0000000000134a87+0x2f94>
  1c2ff1:	mov    0x2018(%rsp),%rbp
  1c2ff9:	incl   0x1f28(%rsp)
  1c3000:	movb   $0x41,(%rdx,%rbp,1)
  1c3004:	cmp    0x1f20(%rsp),%rbx
  1c300c:	jl     1c303b <PRIVATE0000000000134a87+0x2fcb>
  1c300e:	mov    0x1e70(%rsp),%rbp
  1c3016:	mov    0x1ec0(%rsp),%rdx
  1c301e:	movsd  0x0(%rbp,%rbx,8),%xmm0
  1c3024:	addsd  (%rdx,%rbx,8),%xmm0
  1c3029:	movsd  %xmm0,0x0(%rbp,%rbx,8)
  1c302f:	mov    0x2f8(%r14),%ebp
  1c3036:	jmp    1c30d5 <PRIVATE0000000000134a87+0x3065>
  1c303b:	mov    0x1fa8(%rsp),%rdx
  1c3043:	mov    0x2f8(%r14),%ebp
  1c304a:	movsd  (%rdx,%rbx,8),%xmm0
  1c304f:	test   %ebp,%ebp
  1c3051:	jl     1c46e5 <PRIVATE0000000000134a87+0x4675>
  1c3057:	pxor   %xmm1,%xmm1
  1c305b:	ucomisd %xmm1,%xmm0
  1c305f:	jp     1c3063 <PRIVATE0000000000134a87+0x2ff3>
  1c3061:	je     1c30dd <PRIVATE0000000000134a87+0x306d>
  1c3063:	lea    0x0(,%rbx,4),%rdx
  1c306b:	mov    0x2040(%rsp),%r10
  1c3073:	mov    0x2038(%rsp),%r11
  1c307b:	mov    (%r10,%rbx,8),%r10
  1c307f:	mov    %r10,%rbx
  1c3082:	cmpl   $0x0,(%rdx,%r11,1)
  1c3087:	jle    1c30d5 <PRIVATE0000000000134a87+0x3065>
  1c3089:	mov    0x1fd0(%rsp),%r13
  1c3091:	mov    0x2028(%rsp),%rcx
  1c3099:	mov    0x2030(%rsp),%rsi
  1c30a1:	mov    %r11,%rdi
  1c30a4:	movaps %xmm0,%xmm1
  1c30a7:	mulsd  (%rcx,%rbx,8),%xmm1
  1c30ac:	movslq (%rsi,%rbx,4),%r11
  1c30b0:	inc    %rbx
  1c30b3:	mov    %r11d,0x2098(%rsp)
  1c30bb:	movslq (%rdx,%rdi,1),%r12
  1c30bf:	add    %r10,%r12
  1c30c2:	addsd  0x0(%r13,%r11,8),%xmm1
  1c30c9:	movsd  %xmm1,0x0(%r13,%r11,8)
  1c30d0:	cmp    %r12,%rbx
  1c30d3:	jl     1c30a4 <PRIVATE0000000000134a87+0x3034>
  1c30d5:	test   %ebp,%ebp
  1c30d7:	jl     1c46d7 <PRIVATE0000000000134a87+0x4667>
  1c30dd:	mov    0x32c(%r14),%edx
  1c30e4:	cmp    0x2060(%rsp),%edx
  1c30eb:	jge    1c3190 <PRIVATE0000000000134a87+0x3120>
  1c30f1:	movslq %edx,%rdx
  1c30f4:	mov    0x1fc0(%rsp),%rbx
  1c30fc:	cmpl   $0xffffffff,(%rbx,%rdx,4)
  1c3100:	je     1c46be <PRIVATE0000000000134a87+0x464e>
  1c3106:	mov    0x1f88(%rsp),%rbx
  1c310e:	movsd  (%rbx,%rdx,8),%xmm0
  1c3113:	xorps  0x289e176(%rip),%xmm0        # 2a61290 <__STRLITPACK_7+0x964>
  1c311a:	pxor   %xmm1,%xmm1
  1c311e:	ucomisd %xmm1,%xmm0
  1c3122:	jp     1c3126 <PRIVATE0000000000134a87+0x30b6>
  1c3124:	je     1c3190 <PRIVATE0000000000134a87+0x3120>
  1c3126:	mov    0x2040(%rsp),%rbx
  1c312e:	mov    0x2038(%rsp),%r11
  1c3136:	mov    (%rbx,%rdx,8),%r10
  1c313a:	mov    %r10,%rbx
  1c313d:	cmpl   $0x0,(%r11,%rdx,4)
  1c3142:	jle    1c3190 <PRIVATE0000000000134a87+0x3120>
  1c3144:	mov    0x1fd0(%rsp),%r13
  1c314c:	mov    0x2028(%rsp),%rcx
  1c3154:	mov    0x2030(%rsp),%rsi
  1c315c:	mov    %r11,%rdi
  1c315f:	movaps %xmm0,%xmm1
  1c3162:	mulsd  (%rcx,%rbx,8),%xmm1
  1c3167:	movslq (%rsi,%rbx,4),%r11
  1c316b:	inc    %rbx
  1c316e:	mov    %r11d,0x2098(%rsp)
  1c3176:	movslq (%rdi,%rdx,4),%r12
  1c317a:	add    %r10,%r12
  1c317d:	addsd  0x0(%r13,%r11,8),%xmm1
  1c3184:	movsd  %xmm1,0x0(%r13,%r11,8)
  1c318b:	cmp    %r12,%rbx
  1c318e:	jl     1c315f <PRIVATE0000000000134a87+0x30ef>
  1c3190:	cmpl   $0x0,(%rax)
  1c3193:	jle    1c32b5 <PRIVATE0000000000134a87+0x3245>
  1c3199:	movsd  0x1f08(%rsp),%xmm3
  1c31a2:	xor    %edx,%edx
  1c31a4:	movsd  0x2070(%rsp),%xmm6
  1c31ad:	pxor   %xmm4,%xmm4
  1c31b1:	movsd  0x1fc8(%rsp),%xmm7
  1c31ba:	mov    0x1f28(%rsp),%r11d
  1c31c2:	mov    0x1ff0(%rsp),%r12
  1c31ca:	mov    0x2010(%rsp),%r13
  1c31d2:	mov    0x2018(%rsp),%rcx
  1c31da:	mov    0x10(%rax),%rbp
  1c31de:	movaps %xmm5,%xmm0
  1c31e1:	mov    0x8(%rax),%rbx
  1c31e5:	mulsd  0x0(%rbp,%rdx,8),%xmm0
  1c31eb:	movslq (%rbx,%rdx,4),%r10
  1c31ef:	mov    %r10d,0x2098(%rsp)
  1c31f7:	movsd  0x0(%r13,%r10,8),%xmm2
  1c31fe:	subsd  %xmm0,%xmm2
  1c3202:	movsd  %xmm2,0x0(%r13,%r10,8)
  1c3209:	mov    (%r10,%rcx,1),%bl
  1c320d:	cmp    $0x42,%bl
  1c3210:	je     1c46b6 <PRIVATE0000000000134a87+0x4646>
  1c3216:	cmp    $0x41,%bl
  1c3219:	je     1c46b6 <PRIVATE0000000000134a87+0x4646>
  1c321f:	cmp    $0x46,%bl
  1c3222:	je     1c328f <PRIVATE0000000000134a87+0x321f>
  1c3224:	comisd %xmm2,%xmm3
  1c3228:	jbe    1c3234 <PRIVATE0000000000134a87+0x31c4>
  1c322a:	movb   $0x42,(%r10,%rcx,1)
  1c322f:	inc    %r11d
  1c3232:	jmp    1c328f <PRIVATE0000000000134a87+0x321f>
  1c3234:	comisd %xmm2,%xmm6
  1c3238:	jbe    1c3256 <PRIVATE0000000000134a87+0x31e6>
  1c323a:	movsd  (%r12,%r10,8),%xmm0
  1c3240:	ucomisd %xmm4,%xmm0
  1c3244:	jp     1c3248 <PRIVATE0000000000134a87+0x31d8>
  1c3246:	je     1c324f <PRIVATE0000000000134a87+0x31df>
  1c3248:	movb   $0x4c,(%r10,%rcx,1)
  1c324d:	jmp    1c328f <PRIVATE0000000000134a87+0x321f>
  1c324f:	movb   $0x58,(%r10,%rcx,1)
  1c3254:	jmp    1c328f <PRIVATE0000000000134a87+0x321f>
  1c3256:	movsd  (%r12,%r10,8),%xmm1
  1c325c:	movaps %xmm1,%xmm0
  1c325f:	subsd  %xmm6,%xmm0
  1c3263:	comisd %xmm2,%xmm0
  1c3267:	ja     1c326f <PRIVATE0000000000134a87+0x31ff>
  1c3269:	comisd %xmm7,%xmm1
  1c326d:	jb     1c3276 <PRIVATE0000000000134a87+0x3206>
  1c326f:	movb   $0x4e,(%r10,%rcx,1)
  1c3274:	jmp    1c328f <PRIVATE0000000000134a87+0x321f>
  1c3276:	addsd  %xmm6,%xmm1
  1c327a:	comisd %xmm2,%xmm1
  1c327e:	jbe    1c3287 <PRIVATE0000000000134a87+0x3217>
  1c3280:	movb   $0x55,(%r10,%rcx,1)
  1c3285:	jmp    1c328f <PRIVATE0000000000134a87+0x321f>
  1c3287:	movb   $0x41,(%r10,%rcx,1)
  1c328c:	inc    %r11d
  1c328f:	inc    %rdx
  1c3292:	movslq (%rax),%rbx
  1c3295:	cmp    %rbx,%rdx
  1c3298:	jl     1c31da <PRIVATE0000000000134a87+0x316a>
  1c329e:	mov    0x2f8(%r14),%ebp
  1c32a5:	mov    %r11d,0x1f28(%rsp)
  1c32ad:	test   %ebp,%ebp
  1c32af:	jl     1c3b9f <PRIVATE0000000000134a87+0x3b2f>
  1c32b5:	cmpl   $0x0,0x209c(%rsp)
  1c32bd:	je     1c336d <PRIVATE0000000000134a87+0x32fd>
  1c32c3:	mov    0x1ff8(%rsp),%rsi
  1c32cb:	mov    %r15,%rcx
  1c32ce:	movsd  0x289dffa(%rip),%xmm0        # 2a612d0 <__STRLITPACK_7+0x9a4>
  1c32d6:	mov    0x2048(%rsp),%rdi
  1c32de:	mov    0x8(%rsi),%rax
  1c32e2:	movl   $0x1,(%rsi)
  1c32e8:	mov    0x2000(%rsp),%rdx
  1c32f0:	mov    %ebp,(%rax)
  1c32f2:	mov    0x10(%rsi),%rbp
  1c32f6:	movsd  %xmm0,0x0(%rbp)
  1c32fb:	call   131710 <PRIVATE000000000009e376>
  1c3300:	mov    0x2000(%rsp),%rax
  1c3308:	mov    (%rax),%eax
  1c330a:	test   %eax,%eax
  1c330c:	jl     1c45c7 <PRIVATE0000000000134a87+0x4557>
  1c3312:	mov    0x2000(%rsp),%rdx
  1c331a:	mov    0x8(%rdx),%rdx
  1c331e:	jle    1c336d <PRIVATE0000000000134a87+0x32fd>
  1c3320:	mov    %eax,%ebp
  1c3322:	mov    $0x1,%r12d
  1c3328:	mov    0x20(%rsp),%rbx
  1c332d:	mov    %r12d,%r11d
  1c3330:	xor    %r10d,%r10d
  1c3333:	shr    %ebp
  1c3335:	je     1c3355 <PRIVATE0000000000134a87+0x32e5>
  1c3337:	movslq (%rdx,%r10,8),%r12
  1c333b:	mov    %r11d,(%rbx,%r12,4)
  1c333f:	movslq 0x4(%rdx,%r10,8),%r13
  1c3344:	inc    %r10
  1c3347:	mov    %r11d,(%rbx,%r13,4)
  1c334b:	cmp    %rbp,%r10
  1c334e:	jb     1c3337 <PRIVATE0000000000134a87+0x32c7>
  1c3350:	lea    0x1(%r10,%r10,1),%r12d
  1c3355:	lea    -0x1(%r12),%ebp
  1c335a:	cmp    %eax,%ebp
  1c335c:	jae    1c336d <PRIVATE0000000000134a87+0x32fd>
  1c335e:	movslq %r12d,%r12
  1c3361:	movslq -0x4(%rdx,%r12,4),%rax
  1c3366:	movl   $0x1,(%rbx,%rax,4)
  1c336d:	cmpl   $0x0,0x294(%r14)
  1c3375:	jne    1c3465 <PRIVATE0000000000134a87+0x33f5>
  1c337b:	mov    0x2048(%rsp),%rax
  1c3383:	mov    0x18c(%rax),%ebx
  1c3389:	cmp    0x188(%rax),%ebx
  1c338f:	je     1c344d <PRIVATE0000000000134a87+0x33dd>
  1c3395:	pxor   %xmm1,%xmm1
  1c3399:	mov    0x1e8(%rax),%rdx
  1c33a0:	add    0x1e0(%rax),%rdx
  1c33a7:	cvtsi2sd %rdx,%xmm1
  1c33ac:	mov    0x46c(%r14),%edx
  1c33b3:	subsd  0x1e78(%rsp),%xmm1
  1c33bc:	cmp    %edx,%ebx
  1c33be:	jge    1c33c6 <PRIVATE0000000000134a87+0x3356>
  1c33c0:	pxor   %xmm3,%xmm3
  1c33c4:	jmp    1c3411 <PRIVATE0000000000134a87+0x33a1>
  1c33c6:	pxor   %xmm3,%xmm3
  1c33ca:	pxor   %xmm0,%xmm0
  1c33ce:	pxor   %xmm2,%xmm2
  1c33d2:	cvtsi2sd %edx,%xmm2
  1c33d6:	cvtsi2sdq 0x190(%rax),%xmm3
  1c33df:	cvtsi2sdl 0x184(%rax),%xmm0
  1c33e7:	mulsd  0x1e80(%rsp),%xmm2
  1c33f0:	mulsd  %xmm1,%xmm1
  1c33f4:	addsd  %xmm0,%xmm3
  1c33f8:	minsd  %xmm1,%xmm2
  1c33fc:	mulsd  0x289df4c(%rip),%xmm3        # 2a61350 <__STRLITPACK_7+0xa24>
  1c3404:	mulsd  0x1e98(%rsp),%xmm3
  1c340d:	subsd  %xmm2,%xmm3
  1c3411:	mov    0x178(%r14),%eax
  1c3418:	pxor   %xmm0,%xmm0
  1c341c:	comisd %xmm0,%xmm3
  1c3420:	jbe    1c3435 <PRIVATE0000000000134a87+0x33c5>
  1c3422:	test   %eax,%eax
  1c3424:	je     1c3428 <PRIVATE0000000000134a87+0x33b8>
  1c3426:	cmp    %edx,%ebx
  1c3428:	movl   $0xa,0x294(%r14)
  1c3433:	jmp    1c3465 <PRIVATE0000000000134a87+0x33f5>
  1c3435:	test   %eax,%eax
  1c3437:	je     1c343d <PRIVATE0000000000134a87+0x33cd>
  1c3439:	cmp    %edx,%ebx
  1c343b:	jg     1c3428 <PRIVATE0000000000134a87+0x33b8>
  1c343d:	cmpl   $0x0,0x209c(%rsp)
  1c3445:	jne    1c3b6e <PRIVATE0000000000134a87+0x3afe>
  1c344b:	jmp    1c3465 <PRIVATE0000000000134a87+0x33f5>
  1c344d:	movl   $0x2,0x294(%r14)
  1c3458:	jmp    1c3465 <PRIVATE0000000000134a87+0x33f5>
  1c345a:	movl   $0x14,0x1fd8(%rsp)
  1c3465:	mov    %r15,%rsi
  1c3468:	mov    0x1f80(%rsp),%rdi
  1c3470:	call   8eacf0 <PRIVATE0000000000ade770>
  1c3475:	mov    %r15,%rsi
  1c3478:	mov    0x1f40(%rsp),%rdi
  1c3480:	call   8ead90 <PRIVATE0000000000ade8ca>
  1c3485:	mov    0x209c(%rsp),%ebp
  1c348c:	mov    %ebp,0x1f48(%rsp)
  1c3493:	mov    0x470(%r14),%rbp
  1c349a:	mov    0x2090(%rsp),%rax
  1c34a2:	mov    0x210(%r14),%r13
  1c34a9:	mov    %rax,0x210(%r14)
  1c34b0:	mov    0x1850(%rsp),%eax
  1c34b7:	mov    0x1c8(%rbp),%r12
  1c34be:	test   %eax,%eax
  1c34c0:	jle    1c34d9 <PRIVATE0000000000134a87+0x3469>
  1c34c2:	movslq %eax,%rax
  1c34c5:	lea    (%rax,%rax,2),%rdx
  1c34c9:	shl    $0x5,%rdx
  1c34cd:	lea    0x18(%rsp,%rdx,1),%rbx
  1c34d2:	mov    %rbx,0x1c8(%rbp)
  1c34d9:	mov    0x184c(%rsp),%edx
  1c34e0:	cmp    $0x1,%edx
  1c34e3:	jle    1c352d <PRIVATE0000000000134a87+0x34bd>
  1c34e5:	cmp    %edx,%eax
  1c34e7:	jge    1c352d <PRIVATE0000000000134a87+0x34bd>
  1c34e9:	mov    0x294(%r14),%eax
  1c34f0:	test   %eax,%eax
  1c34f2:	je     1c34f8 <PRIVATE0000000000134a87+0x3488>
  1c34f4:	mov    %eax,0x30(%rsp)
  1c34f8:	mov    %r14,%rdi
  1c34fb:	mov    %r15,%rsi
  1c34fe:	call   f15c0 <PRIVATE000000000005b257>
  1c3503:	mov    %eax,%ebx
  1c3505:	test   %ebx,%ebx
  1c3507:	je     1c353a <PRIVATE0000000000134a87+0x34ca>
  1c3509:	mov    0x30(%rsp),%edx
  1c350d:	test   %edx,%edx
  1c350f:	je     1c353a <PRIVATE0000000000134a87+0x34ca>
  1c3511:	mov    %r14,%rdi
  1c3514:	mov    %r15,%rsi
  1c3517:	mov    %edx,0x294(%r14)
  1c351e:	movl   $0x0,0x30(%rsp)
  1c3526:	call   f15c0 <PRIVATE000000000005b257>
  1c352b:	jmp    1c3538 <PRIVATE0000000000134a87+0x34c8>
  1c352d:	mov    %r14,%rdi
  1c3530:	mov    %r15,%rsi
  1c3533:	call   f15c0 <PRIVATE000000000005b257>
  1c3538:	mov    %eax,%ebx
  1c353a:	cmpl   $0x0,0x1850(%rsp)
  1c3542:	jle    1c362d <PRIVATE0000000000134a87+0x35bd>
  1c3548:	mov    %r14,%rdi
  1c354b:	mov    %r12,0x1c8(%rbp)
  1c3552:	call   1bb6f0 <PRIVATE000000000012dd49>
  1c3557:	test   %eax,%eax
  1c3559:	jne    1c362d <PRIVATE0000000000134a87+0x35bd>
  1c355f:	cmpl   $0x0,0x1f48(%rsp)
  1c3567:	je     1c358b <PRIVATE0000000000134a87+0x351b>
  1c3569:	xor    %ebp,%ebp
  1c356b:	cmpl   $0x0,0x8(%rsp)
  1c3570:	jle    1c358b <PRIVATE0000000000134a87+0x351b>
  1c3572:	mov    0x20(%rsp),%rdx
  1c3577:	movl   $0x0,(%rdx,%rbp,4)
  1c357e:	inc    %rbp
  1c3581:	movslq 0x8(%rsp),%r10
  1c3586:	cmp    %r10,%rbp
  1c3589:	jl     1c3577 <PRIVATE0000000000134a87+0x3507>
  1c358b:	mov    0x1850(%rsp),%r10d
  1c3593:	movslq %r10d,%r10
  1c3596:	movslq 0x2c(%rsp),%r11
  1c359b:	lea    (%r10,%r10,2),%rbp
  1c359f:	shl    $0x5,%rbp
  1c35a3:	cmp    0x184c(%rsp),%r10d
  1c35ab:	jge    1c35d2 <PRIVATE0000000000134a87+0x3562>
  1c35ad:	mov    0x10(%rsp),%rdx
  1c35b2:	mov    0x90(%rsp,%rbp,1),%r12d
  1c35ba:	inc    %r10d
  1c35bd:	mov    %r12d,(%rdx,%r11,4)
  1c35c1:	add    $0x60,%rbp
  1c35c5:	inc    %r11
  1c35c8:	cmp    0x184c(%rsp),%r10d
  1c35d0:	jl     1c35b2 <PRIVATE0000000000134a87+0x3542>
  1c35d2:	movl   $0x0,0x1850(%rsp)
  1c35dd:	cmpl   $0x0,0x1848(%rsp)
  1c35e5:	jle    1c35f2 <PRIVATE0000000000134a87+0x3582>
  1c35e7:	movl   $0x0,0x184c(%rsp)
  1c35f2:	mov    0x28(%rsp),%r10d
  1c35f7:	movslq %r10d,%rbp
  1c35fa:	cmp    0xc(%rsp),%r10d
  1c35ff:	jge    1c361e <PRIVATE0000000000134a87+0x35ae>
  1c3601:	mov    0x10(%rsp),%rdx
  1c3606:	mov    (%rdx,%rbp,4),%r12d
  1c360a:	inc    %r10d
  1c360d:	mov    %r12d,(%rdx,%r11,4)
  1c3611:	inc    %rbp
  1c3614:	inc    %r11
  1c3617:	cmp    0xc(%rsp),%r10d
  1c361c:	jl     1c3606 <PRIVATE0000000000134a87+0x3596>
  1c361e:	xor    %edx,%edx
  1c3620:	mov    %r11d,0xc(%rsp)
  1c3625:	mov    %edx,0x2c(%rsp)
  1c3629:	mov    %edx,0x28(%rsp)
  1c362d:	cmp    $0x1,%ebx
  1c3630:	je     1c363e <PRIVATE0000000000134a87+0x35ce>
  1c3632:	mov    %r13,0x210(%r14)
  1c3639:	jmp    1c37f5 <PRIVATE0000000000134a87+0x3785>
  1c363e:	mov    %r13,0x1f38(%rsp)
  1c3646:	mov    0x470(%r14),%r12
  1c364d:	mov    0x2090(%rsp),%rax
  1c3655:	mov    %rax,0x210(%r14)
  1c365c:	mov    0x1850(%rsp),%eax
  1c3663:	mov    0x209c(%rsp),%ebp
  1c366a:	mov    0x1c8(%r12),%r13
  1c3672:	test   %eax,%eax
  1c3674:	jle    1c368e <PRIVATE0000000000134a87+0x361e>
  1c3676:	movslq %eax,%rax
  1c3679:	lea    (%rax,%rax,2),%rdx
  1c367d:	shl    $0x5,%rdx
  1c3681:	lea    0x18(%rsp,%rdx,1),%rbx
  1c3686:	mov    %rbx,0x1c8(%r12)
  1c368e:	mov    0x184c(%rsp),%edx
  1c3695:	cmp    $0x1,%edx
  1c3698:	jle    1c36e2 <PRIVATE0000000000134a87+0x3672>
  1c369a:	cmp    %edx,%eax
  1c369c:	jge    1c36e2 <PRIVATE0000000000134a87+0x3672>
  1c369e:	mov    0x294(%r14),%eax
  1c36a5:	test   %eax,%eax
  1c36a7:	je     1c36ad <PRIVATE0000000000134a87+0x363d>
  1c36a9:	mov    %eax,0x30(%rsp)
  1c36ad:	mov    %r14,%rdi
  1c36b0:	mov    %r15,%rsi
  1c36b3:	call   f15c0 <PRIVATE000000000005b257>
  1c36b8:	mov    %eax,%ebx
  1c36ba:	test   %ebx,%ebx
  1c36bc:	je     1c36ef <PRIVATE0000000000134a87+0x367f>
  1c36be:	mov    0x30(%rsp),%edx
  1c36c2:	test   %edx,%edx
  1c36c4:	je     1c36ef <PRIVATE0000000000134a87+0x367f>
  1c36c6:	mov    %r14,%rdi
  1c36c9:	mov    %r15,%rsi
  1c36cc:	mov    %edx,0x294(%r14)
  1c36d3:	movl   $0x0,0x30(%rsp)
  1c36db:	call   f15c0 <PRIVATE000000000005b257>
  1c36e0:	jmp    1c36ed <PRIVATE0000000000134a87+0x367d>
  1c36e2:	mov    %r14,%rdi
  1c36e5:	mov    %r15,%rsi
  1c36e8:	call   f15c0 <PRIVATE000000000005b257>
  1c36ed:	mov    %eax,%ebx
  1c36ef:	cmpl   $0x0,0x1850(%rsp)
  1c36f7:	jle    1c37dd <PRIVATE0000000000134a87+0x376d>
  1c36fd:	mov    %r14,%rdi
  1c3700:	mov    %r13,0x1c8(%r12)
  1c3708:	call   1bb6f0 <PRIVATE000000000012dd49>
  1c370d:	test   %eax,%eax
  1c370f:	jne    1c37dd <PRIVATE0000000000134a87+0x376d>
  1c3715:	test   %ebp,%ebp
  1c3717:	je     1c373b <PRIVATE0000000000134a87+0x36cb>
  1c3719:	xor    %ebp,%ebp
  1c371b:	cmpl   $0x0,0x8(%rsp)
  1c3720:	jle    1c373b <PRIVATE0000000000134a87+0x36cb>
  1c3722:	mov    0x20(%rsp),%rdx
  1c3727:	movl   $0x0,(%rdx,%rbp,4)
  1c372e:	inc    %rbp
  1c3731:	movslq 0x8(%rsp),%r10
  1c3736:	cmp    %r10,%rbp
  1c3739:	jl     1c3727 <PRIVATE0000000000134a87+0x36b7>
  1c373b:	mov    0x1850(%rsp),%r10d
  1c3743:	movslq %r10d,%r10
  1c3746:	movslq 0x2c(%rsp),%r11
  1c374b:	lea    (%r10,%r10,2),%rbp
  1c374f:	shl    $0x5,%rbp
  1c3753:	cmp    0x184c(%rsp),%r10d
  1c375b:	jge    1c3782 <PRIVATE0000000000134a87+0x3712>
  1c375d:	mov    0x10(%rsp),%rdx
  1c3762:	mov    0x90(%rsp,%rbp,1),%r12d
  1c376a:	inc    %r10d
  1c376d:	mov    %r12d,(%rdx,%r11,4)
  1c3771:	add    $0x60,%rbp
  1c3775:	inc    %r11
  1c3778:	cmp    0x184c(%rsp),%r10d
  1c3780:	jl     1c3762 <PRIVATE0000000000134a87+0x36f2>
  1c3782:	movl   $0x0,0x1850(%rsp)
  1c378d:	cmpl   $0x0,0x1848(%rsp)
  1c3795:	jle    1c37a2 <PRIVATE0000000000134a87+0x3732>
  1c3797:	movl   $0x0,0x184c(%rsp)
  1c37a2:	mov    0x28(%rsp),%r10d
  1c37a7:	movslq %r10d,%rbp
  1c37aa:	cmp    0xc(%rsp),%r10d
  1c37af:	jge    1c37ce <PRIVATE0000000000134a87+0x375e>
  1c37b1:	mov    0x10(%rsp),%rdx
  1c37b6:	mov    (%rdx,%rbp,4),%r12d
  1c37ba:	inc    %r10d
  1c37bd:	mov    %r12d,(%rdx,%r11,4)
  1c37c1:	inc    %rbp
  1c37c4:	inc    %r11
  1c37c7:	cmp    0xc(%rsp),%r10d
  1c37cc:	jl     1c37b6 <PRIVATE0000000000134a87+0x3746>
  1c37ce:	xor    %edx,%edx
  1c37d0:	mov    %r11d,0xc(%rsp)
  1c37d5:	mov    %edx,0x2c(%rsp)
  1c37d9:	mov    %edx,0x28(%rsp)
  1c37dd:	cmp    $0x1,%ebx
  1c37e0:	je     1c3646 <PRIVATE0000000000134a87+0x35d6>
  1c37e6:	mov    0x1f38(%rsp),%r13
  1c37ee:	mov    %r13,0x210(%r14)
  1c37f5:	mov    %r15,%rsi
  1c37f8:	mov    0x1f40(%rsp),%rdi
  1c3800:	call   8eacf0 <PRIVATE0000000000ade770>
  1c3805:	mov    %r15,%rsi
  1c3808:	mov    0x1f80(%rsp),%rdi
  1c3810:	call   8ead90 <PRIVATE0000000000ade8ca>
  1c3815:	cmp    $0xc,%ebx
  1c3818:	je     1c454b <PRIVATE0000000000134a87+0x44db>
  1c381e:	cmp    $0x1,%ebx
  1c3821:	jg     1c4591 <PRIVATE0000000000134a87+0x4521>
  1c3827:	cmpl   $0xfffffc18,0xf0(%r14)
  1c3832:	jle    1c44e8 <PRIVATE0000000000134a87+0x4478>
  1c3838:	cmpl   $0x0,0x1fa0(%rsp)
  1c3840:	jne    1c3864 <PRIVATE0000000000134a87+0x37f4>
  1c3842:	cmpl   $0x0,0x184(%r14)
  1c384a:	jne    1c44ad <PRIVATE0000000000134a87+0x443d>
  1c3850:	mov    0x1fe0(%rsp),%eax
  1c3857:	cmp    0x178(%r14),%eax
  1c385e:	jle    1c44ad <PRIVATE0000000000134a87+0x443d>
  1c3864:	cmpl   $0x0,0x294(%r14)
  1c386c:	je     1c3aed <PRIVATE0000000000134a87+0x3a7d>
  1c3872:	mov    0x2048(%rsp),%rax
  1c387a:	cmpl   $0x0,0x2068(%rsp)
  1c3882:	mov    0x18(%rax),%rdx
  1c3886:	mov    %rdx,0x1fe8(%rsp)
  1c388e:	jle    1c38bb <PRIVATE0000000000134a87+0x384b>
  1c3890:	mov    0x2000(%rsp),%rax
  1c3898:	mov    0x10(%rax),%rdi
  1c389c:	cmp    0x1fd0(%rsp),%rdi
  1c38a4:	je     1c38bb <PRIVATE0000000000134a87+0x384b>
  1c38a6:	mov    0x1fd0(%rsp),%rsi
  1c38ae:	mov    0x1f10(%rsp),%rdx
  1c38b6:	call   bcab0 <memcpy@plt>
  1c38bb:	mov    0x2000(%rsp),%rsi
  1c38c3:	xor    %ecx,%ecx
  1c38c5:	mov    %r15,%r8
  1c38c8:	mov    0x2048(%rsp),%rdi
  1c38d0:	mov    0x2008(%rsp),%rdx
  1c38d8:	movl   $0xffffffff,(%rsi)
  1c38de:	call   1205d0 <PRIVATE000000000009cf9b>
  1c38e3:	cmpl   $0x0,0x2068(%rsp)
  1c38eb:	jle    1c4586 <PRIVATE0000000000134a87+0x4516>
  1c38f1:	mov    0x2008(%rsp),%rax
  1c38f9:	mov    0x10(%rax),%rsi
  1c38fd:	cmp    0x2010(%rsp),%rsi
  1c3905:	je     1c391c <PRIVATE0000000000134a87+0x38ac>
  1c3907:	mov    0x2010(%rsp),%rdi
  1c390f:	mov    0x1f10(%rsp),%rdx
  1c3917:	call   bcab0 <memcpy@plt>
  1c391c:	xor    %ebx,%ebx
  1c391e:	xor    %edx,%edx
  1c3920:	mov    0x2018(%rsp),%rax
  1c3928:	pxor   %xmm2,%xmm2
  1c392c:	movsd  0x1f08(%rsp),%xmm4
  1c3935:	pxor   %xmm6,%xmm6
  1c3939:	movsd  0x1f00(%rsp),%xmm5
  1c3942:	movsd  0x2070(%rsp),%xmm7
  1c394b:	movsd  0x1fc8(%rsp),%xmm8
  1c3955:	mov    0x1f50(%rsp),%r10
  1c395d:	mov    0x1fe8(%rsp),%r11
  1c3965:	mov    0x1ff0(%rsp),%r12
  1c396d:	mov    0x2010(%rsp),%r13
  1c3975:	mov    0x1f88(%rsp),%rcx
  1c397d:	mov    0x1f90(%rsp),%rsi
  1c3985:	movslq (%r11,%rdx,4),%rbp
  1c3989:	movsd  (%rsi,%rbp,8),%xmm0
  1c398e:	comisd %xmm5,%xmm0
  1c3992:	jbe    1c39c6 <PRIVATE0000000000134a87+0x3956>
  1c3994:	movsd  (%rcx,%rbp,8),%xmm1
  1c3999:	comisd %xmm8,%xmm1
  1c399e:	movaps %xmm1,%xmm3
  1c39a1:	subsd  %xmm0,%xmm3
  1c39a5:	jb     1c39ab <PRIVATE0000000000134a87+0x393b>
  1c39a7:	movaps %xmm8,%xmm3
  1c39ab:	movsd  %xmm3,(%r12,%rdx,8)
  1c39b1:	movsd  0x0(%r13,%rdx,8),%xmm0
  1c39b8:	subsd  (%rsi,%rbp,8),%xmm0
  1c39bd:	movsd  %xmm0,0x0(%r13,%rdx,8)
  1c39c4:	jmp    1c39cc <PRIVATE0000000000134a87+0x395c>
  1c39c6:	movsd  %xmm8,(%r12,%rdx,8)
  1c39cc:	comisd (%rsi,%rbp,8),%xmm5
  1c39d1:	jb     1c39e7 <PRIVATE0000000000134a87+0x3977>
  1c39d3:	movsd  (%rcx,%rbp,8),%xmm0
  1c39d8:	comisd %xmm8,%xmm0
  1c39dd:	jb     1c39e7 <PRIVATE0000000000134a87+0x3977>
  1c39df:	movb   $0x46,(%rax)
  1c39e2:	jmp    1c3a64 <PRIVATE0000000000134a87+0x39f4>
  1c39e7:	movsd  0x0(%r13,%rdx,8),%xmm3
  1c39ee:	comisd %xmm3,%xmm4
  1c39f2:	jbe    1c3a02 <PRIVATE0000000000134a87+0x3992>
  1c39f4:	movb   $0x42,(%rax)
  1c39f7:	inc    %ebx
  1c39f9:	subsd  0x0(%r13,%rdx,8),%xmm2
  1c3a00:	jmp    1c3a64 <PRIVATE0000000000134a87+0x39f4>
  1c3a02:	comisd %xmm3,%xmm7
  1c3a06:	jbe    1c3a20 <PRIVATE0000000000134a87+0x39b0>
  1c3a08:	movsd  (%r12,%rdx,8),%xmm0
  1c3a0e:	ucomisd %xmm6,%xmm0
  1c3a12:	jp     1c3a16 <PRIVATE0000000000134a87+0x39a6>
  1c3a14:	je     1c3a1b <PRIVATE0000000000134a87+0x39ab>
  1c3a16:	movb   $0x4c,(%rax)
  1c3a19:	jmp    1c3a64 <PRIVATE0000000000134a87+0x39f4>
  1c3a1b:	movb   $0x58,(%rax)
  1c3a1e:	jmp    1c3a64 <PRIVATE0000000000134a87+0x39f4>
  1c3a20:	movsd  (%r12,%rdx,8),%xmm1
  1c3a26:	movaps %xmm1,%xmm0
  1c3a29:	subsd  %xmm7,%xmm0
  1c3a2d:	comisd %xmm3,%xmm0
  1c3a31:	ja     1c3a3a <PRIVATE0000000000134a87+0x39ca>
  1c3a33:	comisd %xmm8,%xmm1
  1c3a38:	jb     1c3a3f <PRIVATE0000000000134a87+0x39cf>
  1c3a3a:	movb   $0x4e,(%rax)
  1c3a3d:	jmp    1c3a64 <PRIVATE0000000000134a87+0x39f4>
  1c3a3f:	addsd  %xmm7,%xmm1
  1c3a43:	comisd %xmm3,%xmm1
  1c3a47:	jbe    1c3a4e <PRIVATE0000000000134a87+0x39de>
  1c3a49:	movb   $0x55,(%rax)
  1c3a4c:	jmp    1c3a64 <PRIVATE0000000000134a87+0x39f4>
  1c3a4e:	movb   $0x41,(%rax)
  1c3a51:	inc    %ebx
  1c3a53:	movsd  0x0(%r13,%rdx,8),%xmm0
  1c3a5a:	subsd  (%r12,%rdx,8),%xmm0
  1c3a60:	addsd  %xmm0,%xmm2
  1c3a64:	inc    %rdx
  1c3a67:	inc    %rax
  1c3a6a:	cmp    %r10,%rdx
  1c3a6d:	jl     1c3985 <PRIVATE0000000000134a87+0x3915>
  1c3a73:	mov    %ebx,0x2098(%rsp)
  1c3a7a:	pxor   %xmm3,%xmm3
  1c3a7e:	mov    0xc(%rsp),%ebx
  1c3a82:	mov    %r14,%rdi
  1c3a85:	mov    0x2c(%rsp),%eax
  1c3a89:	mov    $0x3,%esi
  1c3a8e:	sub    0x28(%rsp),%ebx
  1c3a92:	xor    %edx,%edx
  1c3a94:	add    0x1850(%rsp),%eax
  1c3a9b:	mov    %r15,%rcx
  1c3a9e:	add    %eax,%ebx
  1c3aa0:	pxor   %xmm1,%xmm1
  1c3aa4:	cvtsi2sd %ebx,%xmm3
  1c3aa8:	movsd  0x2a0(%r14),%xmm0
  1c3ab1:	movsd  %xmm2,0x120(%r14)
  1c3aba:	movsd  %xmm2,0x1ea8(%rsp)
  1c3ac3:	call   164c60 <PRIVATE00000000000d07d1>
  1c3ac8:	mov    0x2098(%rsp),%eax
  1c3acf:	movsd  0x1ea8(%rsp),%xmm2
  1c3ad8:	mov    %eax,0x1f28(%rsp)
  1c3adf:	cmpl   $0x2,0x1f98(%rsp)
  1c3ae7:	je     1c452b <PRIVATE0000000000134a87+0x44bb>
  1c3aed:	mov    0x2048(%rsp),%rax
  1c3af5:	mov    0x18c(%rax),%eax
  1c3afb:	cmp    0x1fd8(%rsp),%eax
  1c3b02:	jge    1c3b23 <PRIVATE0000000000134a87+0x3ab3>
  1c3b04:	test   %eax,%eax
  1c3b06:	je     1c3b6e <PRIVATE0000000000134a87+0x3afe>
  1c3b08:	mov    0x1fd8(%rsp),%edx
  1c3b0f:	mov    $0x32,%eax
  1c3b14:	cmp    $0x32,%edx
  1c3b17:	cmovg  %eax,%edx
  1c3b1a:	mov    %edx,0x1fd8(%rsp)
  1c3b21:	jmp    1c3b6e <PRIVATE0000000000134a87+0x3afe>
  1c3b23:	mov    %r14,%rdi
  1c3b26:	lea    0x2098(%rsp),%rdx
  1c3b2e:	mov    %r15,%rcx
  1c3b31:	mov    0x2fc(%r14),%esi
  1c3b38:	call   115ed0 <PRIVATE0000000000089f61>
  1c3b3d:	cmpl   $0xc,0xf0(%r14)
  1c3b45:	je     1c44e8 <PRIVATE0000000000134a87+0x4478>
  1c3b4b:	cmpl   $0x0,0x294(%r14)
  1c3b53:	jne    1c345a <PRIVATE0000000000134a87+0x33ea>
  1c3b59:	mov    0x1fd8(%rsp),%eax
  1c3b60:	add    0x2098(%rsp),%eax
  1c3b67:	mov    %eax,0x1fd8(%rsp)
  1c3b6e:	movl   $0x0,0x294(%r14)
  1c3b79:	jmp    1c3b9f <PRIVATE0000000000134a87+0x3b2f>
  1c3b7b:	mov    0x2c(%rsp),%edx
  1c3b7f:	movslq %edx,%rdx
  1c3b82:	mov    0x10(%rsp),%rax
  1c3b87:	mov    0x18(%rsp),%rbx
  1c3b8c:	movslq 0x2fc(%r14),%rbp
  1c3b93:	mov    %ebp,(%rax,%rdx,4)
  1c3b96:	inc    %edx
  1c3b98:	mov    %edx,0x2c(%rsp)
  1c3b9c:	incl   (%rbx,%rbp,4)
  1c3b9f:	mov    %r14,%rsi
  1c3ba2:	lea    0x8(%rsp),%rdi
  1c3ba7:	mov    %r15,%r8
  1c3baa:	lea    0x2090(%rsp),%rdx
  1c3bb2:	lea    0x209c(%rsp),%rcx
  1c3bba:	call   1c5310 <PRIVATE000000000025615c>
  1c3bbf:	movslq %eax,%rbx
  1c3bc2:	test   %eax,%eax
  1c3bc4:	jge    1c210a <PRIVATE0000000000134a87+0x209a>
  1c3bca:	mov    0x1ea0(%rsp),%ebp
  1c3bd1:	xor    %r12d,%r12d
  1c3bd4:	mov    0x1f40(%rsp),%rbx
  1c3bdc:	xor    %r13d,%r13d
  1c3bdf:	mov    0xc(%rsp),%edx
  1c3be3:	mov    0x2c(%rsp),%eax
  1c3be7:	cmp    %edx,%eax
  1c3be9:	jne    1c3c0c <PRIVATE0000000000134a87+0x3b9c>
  1c3beb:	test   %eax,%eax
  1c3bed:	je     1c4021 <PRIVATE0000000000134a87+0x3fb1>
  1c3bf3:	cmpl   $0x0,0x209c(%rsp)
  1c3bfb:	je     1c4021 <PRIVATE0000000000134a87+0x3fb1>
  1c3c01:	movl   $0x0,0x209c(%rsp)
  1c3c0c:	movsd  0x1e98(%rsp),%xmm0
  1c3c15:	call   bca60 <sqrt@plt>
  1c3c1a:	movaps %xmm0,%xmm1
  1c3c1d:	pxor   %xmm0,%xmm0
  1c3c21:	cvtsi2sd %ebp,%xmm0
  1c3c25:	comisd %xmm0,%xmm1
  1c3c29:	jbe    1c3c38 <PRIVATE0000000000134a87+0x3bc8>
  1c3c2b:	movl   $0x0,0x209c(%rsp)
  1c3c36:	jmp    1c3c65 <PRIVATE0000000000134a87+0x3bf5>
  1c3c38:	cmpl   $0x0,0x209c(%rsp)
  1c3c40:	je     1c3c65 <PRIVATE0000000000134a87+0x3bf5>
  1c3c42:	mov    %r13,%rdx
  1c3c45:	cmpl   $0x0,0x8(%rsp)
  1c3c4a:	jle    1c3c65 <PRIVATE0000000000134a87+0x3bf5>
  1c3c4c:	mov    0x20(%rsp),%rax
  1c3c51:	movl   $0x0,(%rax,%rdx,4)
  1c3c58:	inc    %rdx
  1c3c5b:	movslq 0x8(%rsp),%rbp
  1c3c60:	cmp    %rbp,%rdx
  1c3c63:	jl     1c3c51 <PRIVATE0000000000134a87+0x3be1>
  1c3c65:	mov    0x1850(%rsp),%ebp
  1c3c6c:	movslq %ebp,%rbp
  1c3c6f:	movslq 0x2c(%rsp),%r10
  1c3c74:	lea    0x0(%rbp,%rbp,2),%rdx
  1c3c79:	shl    $0x5,%rdx
  1c3c7d:	cmp    0x184c(%rsp),%ebp
  1c3c84:	jge    1c3ca9 <PRIVATE0000000000134a87+0x3c39>
  1c3c86:	mov    0x10(%rsp),%rax
  1c3c8b:	mov    0x90(%rsp,%rdx,1),%r11d
  1c3c93:	inc    %ebp
  1c3c95:	mov    %r11d,(%rax,%r10,4)
  1c3c99:	add    $0x60,%rdx
  1c3c9d:	inc    %r10
  1c3ca0:	cmp    0x184c(%rsp),%ebp
  1c3ca7:	jl     1c3c8b <PRIVATE0000000000134a87+0x3c1b>
  1c3ca9:	movl   $0x0,0x1850(%rsp)
  1c3cb4:	cmpl   $0x0,0x1848(%rsp)
  1c3cbc:	jle    1c3cc9 <PRIVATE0000000000134a87+0x3c59>
  1c3cbe:	movl   $0x0,0x184c(%rsp)
  1c3cc9:	mov    0x28(%rsp),%ebp
  1c3ccd:	movslq %ebp,%rdx
  1c3cd0:	cmp    0xc(%rsp),%ebp
  1c3cd4:	jge    1c3cf1 <PRIVATE0000000000134a87+0x3c81>
  1c3cd6:	mov    0x10(%rsp),%rax
  1c3cdb:	mov    (%rax,%rdx,4),%r11d
  1c3cdf:	inc    %ebp
  1c3ce1:	mov    %r11d,(%rax,%r10,4)
  1c3ce5:	inc    %rdx
  1c3ce8:	inc    %r10
  1c3ceb:	cmp    0xc(%rsp),%ebp
  1c3cef:	jl     1c3cdb <PRIVATE0000000000134a87+0x3c6b>
  1c3cf1:	mov    %r15,%rsi
  1c3cf4:	mov    0x1f80(%rsp),%rdi
  1c3cfc:	mov    %r10d,0xc(%rsp)
  1c3d01:	mov    %r12d,0x2c(%rsp)
  1c3d06:	mov    %r12d,0x28(%rsp)
  1c3d0b:	movl   $0xa,0x294(%r14)
  1c3d16:	call   8eacf0 <PRIVATE0000000000ade770>
  1c3d1b:	mov    %rbx,%rdi
  1c3d1e:	mov    %r15,%rsi
  1c3d21:	call   8ead90 <PRIVATE0000000000ade8ca>
  1c3d26:	mov    %r14,%rdi
  1c3d29:	mov    %r15,%rsi
  1c3d2c:	call   f15c0 <PRIVATE000000000005b257>
  1c3d31:	mov    %eax,%ebp
  1c3d33:	cmp    $0x1,%ebp
  1c3d36:	jne    1c3d4a <PRIVATE0000000000134a87+0x3cda>
  1c3d38:	mov    %r14,%rdi
  1c3d3b:	mov    %r15,%rsi
  1c3d3e:	call   f15c0 <PRIVATE000000000005b257>
  1c3d43:	mov    %eax,%ebp
  1c3d45:	cmp    $0x1,%ebp
  1c3d48:	je     1c3d38 <PRIVATE0000000000134a87+0x3cc8>
  1c3d4a:	mov    %rbx,%rdi
  1c3d4d:	mov    %r15,%rsi
  1c3d50:	call   8eacf0 <PRIVATE0000000000ade770>
  1c3d55:	mov    %r15,%rsi
  1c3d58:	mov    0x1f80(%rsp),%rdi
  1c3d60:	call   8ead90 <PRIVATE0000000000ade8ca>
  1c3d65:	cmp    $0xc,%ebp
  1c3d68:	je     1c454b <PRIVATE0000000000134a87+0x44db>
  1c3d6e:	cmp    $0x1,%ebp
  1c3d71:	jg     1c4918 <PRIVATE0000000000134a87+0x48a8>
  1c3d77:	cmpl   $0xfffffc18,0xf0(%r14)
  1c3d82:	jle    1c44e8 <PRIVATE0000000000134a87+0x4478>
  1c3d88:	cmpl   $0x0,0x1fa0(%rsp)
  1c3d90:	jne    1c3db4 <PRIVATE0000000000134a87+0x3d44>
  1c3d92:	cmpl   $0x0,0x184(%r14)
  1c3d9a:	jne    1c4739 <PRIVATE0000000000134a87+0x46c9>
  1c3da0:	mov    0x1fe0(%rsp),%eax
  1c3da7:	cmp    0x178(%r14),%eax
  1c3dae:	jle    1c4739 <PRIVATE0000000000134a87+0x46c9>
  1c3db4:	mov    0x2048(%rsp),%rax
  1c3dbc:	cmpl   $0x0,0x2068(%rsp)
  1c3dc4:	mov    0x18(%rax),%rdx
  1c3dc8:	mov    %rdx,0x1fe8(%rsp)
  1c3dd0:	jle    1c3dfd <PRIVATE0000000000134a87+0x3d8d>
  1c3dd2:	mov    0x2000(%rsp),%rax
  1c3dda:	mov    0x10(%rax),%rdi
  1c3dde:	cmp    0x1fd0(%rsp),%rdi
  1c3de6:	je     1c3dfd <PRIVATE0000000000134a87+0x3d8d>
  1c3de8:	mov    0x1fd0(%rsp),%rsi
  1c3df0:	mov    0x1f10(%rsp),%rdx
  1c3df8:	call   bcab0 <memcpy@plt>
  1c3dfd:	mov    0x2000(%rsp),%rsi
  1c3e05:	xor    %ecx,%ecx
  1c3e07:	mov    %r15,%r8
  1c3e0a:	mov    0x2048(%rsp),%rdi
  1c3e12:	mov    0x2008(%rsp),%rdx
  1c3e1a:	movl   $0xffffffff,(%rsi)
  1c3e20:	call   1205d0 <PRIVATE000000000009cf9b>
  1c3e25:	cmpl   $0x0,0x2068(%rsp)
  1c3e2d:	jle    1c490c <PRIVATE0000000000134a87+0x489c>
  1c3e33:	mov    0x2008(%rsp),%rax
  1c3e3b:	mov    0x10(%rax),%rsi
  1c3e3f:	cmp    0x2010(%rsp),%rsi
  1c3e47:	je     1c3e5e <PRIVATE0000000000134a87+0x3dee>
  1c3e49:	mov    0x2010(%rsp),%rdi
  1c3e51:	mov    0x1f10(%rsp),%rdx
  1c3e59:	call   bcab0 <memcpy@plt>
  1c3e5e:	mov    0x2018(%rsp),%rax
  1c3e66:	mov    %r12d,%ebp
  1c3e69:	movsd  0x1f08(%rsp),%xmm4
  1c3e72:	pxor   %xmm2,%xmm2
  1c3e76:	movsd  0x1f00(%rsp),%xmm5
  1c3e7f:	pxor   %xmm6,%xmm6
  1c3e83:	movsd  0x2070(%rsp),%xmm7
  1c3e8c:	mov    %r13,%rdx
  1c3e8f:	movsd  0x1fc8(%rsp),%xmm8
  1c3e99:	mov    0x1f50(%rsp),%r11
  1c3ea1:	mov    0x1fe8(%rsp),%rcx
  1c3ea9:	mov    0x1ff0(%rsp),%rsi
  1c3eb1:	mov    0x2010(%rsp),%rdi
  1c3eb9:	mov    0x1f88(%rsp),%r8
  1c3ec1:	mov    0x1f90(%rsp),%r9
  1c3ec9:	movslq (%rcx,%rdx,4),%r10
  1c3ecd:	movsd  (%r9,%r10,8),%xmm0
  1c3ed3:	comisd %xmm5,%xmm0
  1c3ed7:	jbe    1c3f08 <PRIVATE0000000000134a87+0x3e98>
  1c3ed9:	movsd  (%r8,%r10,8),%xmm1
  1c3edf:	comisd %xmm8,%xmm1
  1c3ee4:	movaps %xmm1,%xmm3
  1c3ee7:	subsd  %xmm0,%xmm3
  1c3eeb:	jb     1c3ef1 <PRIVATE0000000000134a87+0x3e81>
  1c3eed:	movaps %xmm8,%xmm3
  1c3ef1:	movsd  %xmm3,(%rsi,%rdx,8)
  1c3ef6:	movsd  (%rdi,%rdx,8),%xmm0
  1c3efb:	subsd  (%r9,%r10,8),%xmm0
  1c3f01:	movsd  %xmm0,(%rdi,%rdx,8)
  1c3f06:	jmp    1c3f0e <PRIVATE0000000000134a87+0x3e9e>
  1c3f08:	movsd  %xmm8,(%rsi,%rdx,8)
  1c3f0e:	comisd (%r9,%r10,8),%xmm5
  1c3f14:	jb     1c3f28 <PRIVATE0000000000134a87+0x3eb8>
  1c3f16:	movsd  (%r8,%r10,8),%xmm0
  1c3f1c:	comisd %xmm8,%xmm0
  1c3f21:	jb     1c3f28 <PRIVATE0000000000134a87+0x3eb8>
  1c3f23:	movb   $0x46,(%rax)
  1c3f26:	jmp    1c3f9c <PRIVATE0000000000134a87+0x3f2c>
  1c3f28:	movsd  (%rdi,%rdx,8),%xmm3
  1c3f2d:	comisd %xmm3,%xmm4
  1c3f31:	jbe    1c3f3f <PRIVATE0000000000134a87+0x3ecf>
  1c3f33:	movb   $0x42,(%rax)
  1c3f36:	inc    %ebp
  1c3f38:	subsd  (%rdi,%rdx,8),%xmm2
  1c3f3d:	jmp    1c3f9c <PRIVATE0000000000134a87+0x3f2c>
  1c3f3f:	comisd %xmm3,%xmm7
  1c3f43:	jbe    1c3f5c <PRIVATE0000000000134a87+0x3eec>
  1c3f45:	movsd  (%rsi,%rdx,8),%xmm0
  1c3f4a:	ucomisd %xmm6,%xmm0
  1c3f4e:	jp     1c3f52 <PRIVATE0000000000134a87+0x3ee2>
  1c3f50:	je     1c3f57 <PRIVATE0000000000134a87+0x3ee7>
  1c3f52:	movb   $0x4c,(%rax)
  1c3f55:	jmp    1c3f9c <PRIVATE0000000000134a87+0x3f2c>
  1c3f57:	movb   $0x58,(%rax)
  1c3f5a:	jmp    1c3f9c <PRIVATE0000000000134a87+0x3f2c>
  1c3f5c:	movsd  (%rsi,%rdx,8),%xmm1
  1c3f61:	movaps %xmm1,%xmm0
  1c3f64:	subsd  %xmm7,%xmm0
  1c3f68:	comisd %xmm3,%xmm0
  1c3f6c:	ja     1c3f75 <PRIVATE0000000000134a87+0x3f05>
  1c3f6e:	comisd %xmm8,%xmm1
  1c3f73:	jb     1c3f7a <PRIVATE0000000000134a87+0x3f0a>
  1c3f75:	movb   $0x4e,(%rax)
  1c3f78:	jmp    1c3f9c <PRIVATE0000000000134a87+0x3f2c>
  1c3f7a:	addsd  %xmm7,%xmm1
  1c3f7e:	comisd %xmm3,%xmm1
  1c3f82:	jbe    1c3f89 <PRIVATE0000000000134a87+0x3f19>
  1c3f84:	movb   $0x55,(%rax)
  1c3f87:	jmp    1c3f9c <PRIVATE0000000000134a87+0x3f2c>
  1c3f89:	movb   $0x41,(%rax)
  1c3f8c:	inc    %ebp
  1c3f8e:	movsd  (%rdi,%rdx,8),%xmm0
  1c3f93:	subsd  (%rsi,%rdx,8),%xmm0
  1c3f98:	addsd  %xmm0,%xmm2
  1c3f9c:	inc    %rdx
  1c3f9f:	inc    %rax
  1c3fa2:	cmp    %r11,%rdx
  1c3fa5:	jl     1c3ec9 <PRIVATE0000000000134a87+0x3e59>
  1c3fab:	pxor   %xmm3,%xmm3
  1c3faf:	mov    %r14,%rdi
  1c3fb2:	cvtsi2sdl 0xc(%rsp),%xmm3
  1c3fb8:	movsd  0x2a0(%r14),%xmm0
  1c3fc1:	pxor   %xmm1,%xmm1
  1c3fc5:	mov    $0x3,%esi
  1c3fca:	xor    %edx,%edx
  1c3fcc:	mov    %r15,%rcx
  1c3fcf:	mov    %ebp,0x2098(%rsp)
  1c3fd6:	movsd  %xmm2,0x120(%r14)
  1c3fdf:	movsd  %xmm2,(%rsp)
  1c3fe4:	call   164c60 <PRIVATE00000000000d07d1>
  1c3fe9:	mov    0x2098(%rsp),%eax
  1c3ff0:	movsd  (%rsp),%xmm2
  1c3ff5:	mov    %eax,0x1f28(%rsp)
  1c3ffc:	cmpl   $0x2,0x1f98(%rsp)
  1c4004:	je     1c48b1 <PRIVATE0000000000134a87+0x4841>
  1c400a:	mov    0xc(%rsp),%edx
  1c400e:	movl   $0x0,0x294(%r14)
  1c4019:	test   %edx,%edx
  1c401b:	jne    1c1ff8 <PRIVATE0000000000134a87+0x1f88>
  1c4021:	mov    %edx,0x1e90(%rsp)
  1c4028:	mov    0x1ee8(%rsp),%rbp
  1c4030:	mov    0x1ef0(%rsp),%r12
  1c4038:	mov    0x1f18(%rsp),%rbx
  1c4040:	mov    0x1f60(%rsp),%r13
  1c4048:	mov    0x1f30(%rsp),%r15
  1c4050:	mov    0x1e90(%rsp),%eax
  1c4057:	mov    %eax,0x1fb0(%rsp)
  1c405e:	cmpl   $0x0,0x2050(%rsp)
  1c4066:	je     1c4139 <PRIVATE0000000000134a87+0x40c9>
  1c406c:	xor    %esi,%esi
  1c406e:	cmpl   $0x0,0x2068(%rsp)
  1c4076:	jle    1c4774 <PRIVATE0000000000134a87+0x4704>
  1c407c:	mov    0x1f20(%rsp),%rax
  1c4084:	shl    $0x3,%rax
  1c4088:	neg    %rax
  1c408b:	add    %rax,0x2058(%rsp)
  1c4093:	mov    %r14,0x1f68(%rsp)
  1c409b:	movl   $0x0,0x2098(%rsp)
  1c40a6:	movsd  0x1f00(%rsp),%xmm2
  1c40af:	mov    0x2058(%rsp),%r10
  1c40b7:	mov    0x1f50(%rsp),%r9
  1c40bf:	mov    0x1fe8(%rsp),%r8
  1c40c7:	mov    0x2010(%rsp),%r14
  1c40cf:	mov    0x1f90(%rsp),%rax
  1c40d7:	mov    0x2060(%rsp),%edx
  1c40de:	mov    0x1fa8(%rsp),%rcx
  1c40e6:	movslq (%r8,%rsi,4),%r11
  1c40ea:	cmp    (%r8,%rsi,4),%edx
  1c40ee:	jg     1c40fa <PRIVATE0000000000134a87+0x408a>
  1c40f0:	mov    (%r14,%rsi,8),%rdi
  1c40f4:	mov    %rdi,(%r10,%r11,8)
  1c40f8:	jmp    1c4120 <PRIVATE0000000000134a87+0x40b0>
  1c40fa:	movsd  (%rax,%r11,8),%xmm1
  1c4100:	comisd %xmm1,%xmm2
  1c4104:	jb     1c4110 <PRIVATE0000000000134a87+0x40a0>
  1c4106:	mov    (%r14,%rsi,8),%rdi
  1c410a:	mov    %rdi,(%rcx,%r11,8)
  1c410e:	jmp    1c4120 <PRIVATE0000000000134a87+0x40b0>
  1c4110:	movsd  (%r14,%rsi,8),%xmm0
  1c4116:	addsd  %xmm0,%xmm1
  1c411a:	movsd  %xmm1,(%rcx,%r11,8)
  1c4120:	inc    %rsi
  1c4123:	mov    %esi,0x2098(%rsp)
  1c412a:	cmp    %r9,%rsi
  1c412d:	jl     1c40e6 <PRIVATE0000000000134a87+0x4076>
  1c412f:	mov    0x1f68(%rsp),%r14
  1c4137:	jmp    1c414d <PRIVATE0000000000134a87+0x40dd>
  1c4139:	xor    %eax,%eax
  1c413b:	pxor   %xmm2,%xmm2
  1c413f:	cmpl   $0x0,0x2068(%rsp)
  1c4147:	jle    1c4211 <PRIVATE0000000000134a87+0x41a1>
  1c414d:	xor    %eax,%eax
  1c414f:	xor    %edx,%edx
  1c4151:	movsd  0x1f08(%rsp),%xmm4
  1c415a:	pxor   %xmm2,%xmm2
  1c415e:	movsd  0x1f00(%rsp),%xmm5
  1c4167:	movsd  0x2070(%rsp),%xmm6
  1c4170:	movsd  0x1fc8(%rsp),%xmm7
  1c4179:	mov    0x1f50(%rsp),%r8
  1c4181:	mov    0x1fe8(%rsp),%r9
  1c4189:	mov    0x1ff0(%rsp),%r10
  1c4191:	mov    0x2010(%rsp),%r11
  1c4199:	mov    0x1f88(%rsp),%rsi
  1c41a1:	mov    0x1f90(%rsp),%rdi
  1c41a9:	movslq (%r9,%rdx,4),%rcx
  1c41ad:	comisd (%rdi,%rcx,8),%xmm5
  1c41b2:	jb     1c41bf <PRIVATE0000000000134a87+0x414f>
  1c41b4:	movsd  (%rsi,%rcx,8),%xmm0
  1c41b9:	comisd %xmm7,%xmm0
  1c41bd:	jae    1c4209 <PRIVATE0000000000134a87+0x4199>
  1c41bf:	movsd  (%r11,%rdx,8),%xmm3
  1c41c5:	comisd %xmm3,%xmm4
  1c41c9:	jbe    1c41d3 <PRIVATE0000000000134a87+0x4163>
  1c41cb:	inc    %eax
  1c41cd:	subsd  %xmm3,%xmm2
  1c41d1:	jmp    1c4209 <PRIVATE0000000000134a87+0x4199>
  1c41d3:	comisd %xmm3,%xmm6
  1c41d7:	ja     1c4209 <PRIVATE0000000000134a87+0x4199>
  1c41d9:	movsd  (%r10,%rdx,8),%xmm1
  1c41df:	movaps %xmm1,%xmm0
  1c41e2:	subsd  %xmm6,%xmm0
  1c41e6:	comisd %xmm3,%xmm0
  1c41ea:	ja     1c4209 <PRIVATE0000000000134a87+0x4199>
  1c41ec:	comisd %xmm7,%xmm1
  1c41f0:	jae    1c4209 <PRIVATE0000000000134a87+0x4199>
  1c41f2:	movaps %xmm6,%xmm0
  1c41f5:	addsd  %xmm1,%xmm0
  1c41f9:	comisd %xmm3,%xmm0
  1c41fd:	ja     1c4209 <PRIVATE0000000000134a87+0x4199>
  1c41ff:	inc    %eax
  1c4201:	subsd  %xmm1,%xmm3
  1c4205:	addsd  %xmm3,%xmm2
  1c4209:	inc    %rdx
  1c420c:	cmp    %r8,%rdx
  1c420f:	jl     1c41a9 <PRIVATE0000000000134a87+0x4139>
  1c4211:	pxor   %xmm3,%xmm3
  1c4215:	mov    %r14,%rdi
  1c4218:	cvtsi2sdl 0x1e90(%rsp),%xmm3
  1c4221:	movsd  0x2a0(%r14),%xmm0
  1c422a:	pxor   %xmm1,%xmm1
  1c422e:	mov    $0x3,%esi
  1c4233:	mov    $0x1,%edx
  1c4238:	mov    0x2100(%rsp),%rcx
  1c4240:	mov    %eax,0x2098(%rsp)
  1c4247:	movsd  %xmm2,0x120(%r14)
  1c4250:	movsd  %xmm2,(%rsp)
  1c4255:	call   164c60 <PRIVATE00000000000d07d1>
  1c425a:	mov    0x4b8(%r14),%rdi
  1c4261:	mov    0x20e0(%rsp),%rsi
  1c4269:	mov    0x2100(%rsp),%rdx
  1c4271:	call   1bbf80 <PRIVATE000000000012f282>
  1c4276:	mov    0x20e0(%rsp),%rax
  1c427e:	cmpl   $0x2,0x1f98(%rsp)
  1c4286:	movsd  (%rsp),%xmm2
  1c428b:	je     1c4295 <PRIVATE0000000000134a87+0x4225>
  1c428d:	movsd  0x289d03b(%rip),%xmm2        # 2a612d0 <__STRLITPACK_7+0x9a4>
  1c4295:	movsd  (%rax),%xmm1
  1c4299:	cmpl   $0x0,0x1fa0(%rsp)
  1c42a1:	jne    1c42dd <PRIVATE0000000000134a87+0x426d>
  1c42a3:	movsd  0x289d0e5(%rip),%xmm0        # 2a61390 <__STRLITPACK_7+0xa64>
  1c42ab:	comisd %xmm0,%xmm1
  1c42af:	ja     1c44d5 <PRIVATE0000000000134a87+0x4465>
  1c42b5:	movsd  0x289d0eb(%rip),%xmm0        # 2a613a8 <__STRLITPACK_7+0xa7c>
  1c42bd:	comisd %xmm0,%xmm2
  1c42c1:	ja     1c44d5 <PRIVATE0000000000134a87+0x4465>
  1c42c7:	mulsd  %xmm1,%xmm2
  1c42cb:	movsd  0x289d105(%rip),%xmm0        # 2a613d8 <__STRLITPACK_7+0xaac>
  1c42d3:	comisd %xmm0,%xmm2
  1c42d7:	ja     1c44d5 <PRIVATE0000000000134a87+0x4465>
  1c42dd:	movsd  0x289d0e3(%rip),%xmm0        # 2a613c8 <__STRLITPACK_7+0xa9c>
  1c42e5:	comisd %xmm0,%xmm1
  1c42e9:	ja     1c44d5 <PRIVATE0000000000134a87+0x4465>
  1c42ef:	cmpl   $0x0,0x1e90(%rsp)
  1c42f7:	je     1c437e <PRIVATE0000000000134a87+0x430e>
  1c42fd:	mov    $0x0,%edx
  1c4302:	mov    $0x0,%eax
  1c4307:	jle    1c4885 <PRIVATE0000000000134a87+0x4815>
  1c430d:	movl   $0x0,0x2098(%rsp)
  1c4318:	movsd  0x1f00(%rsp),%xmm1
  1c4321:	movsd  0x1fc8(%rsp),%xmm2
  1c432a:	mov    0x1e90(%rsp),%r8d
  1c4332:	mov    0x1fb8(%rsp),%r9
  1c433a:	mov    0x1f88(%rsp),%r10
  1c4342:	mov    0x1f90(%rsp),%r11
  1c434a:	jmp    1c4353 <PRIVATE0000000000134a87+0x42e3>
  1c434c:	mov    %edx,0x2098(%rsp)
  1c4353:	movslq (%r9,%rax,4),%rcx
  1c4357:	comisd (%r11,%rcx,8),%xmm1
  1c435d:	jb     1c436f <PRIVATE0000000000134a87+0x42ff>
  1c435f:	movsd  (%r10,%rcx,8),%xmm0
  1c4365:	comisd %xmm2,%xmm0
  1c4369:	jae    1c48a7 <PRIVATE0000000000134a87+0x4837>
  1c436f:	inc    %edx
  1c4371:	inc    %rax
  1c4374:	cmp    %r8d,%edx
  1c4377:	jl     1c434c <PRIVATE0000000000134a87+0x42dc>
  1c4379:	jmp    1c477f <PRIVATE0000000000134a87+0x470f>
  1c437e:	cmpl   $0x0,0x1fb0(%rsp)
  1c4386:	jge    1c08d5 <PRIVATE0000000000134a87+0x865>
  1c438c:	cmpl   $0x0,0x1f58(%rsp)
  1c4394:	jle    1c08d5 <PRIVATE0000000000134a87+0x865>
  1c439a:	xor    %r10d,%r10d
  1c439d:	mov    $0x1,%edx
  1c43a2:	mov    0x1f58(%rsp),%eax
  1c43a9:	xor    %r11d,%r11d
  1c43ac:	shr    %eax
  1c43ae:	je     1c4418 <PRIVATE0000000000134a87+0x43a8>
  1c43b0:	mov    0x1fc0(%rsp),%r9
  1c43b8:	mov    0x1f88(%rsp),%r8
  1c43c0:	mov    0x1f90(%rsp),%rdx
  1c43c8:	mov    0x1fa8(%rsp),%rcx
  1c43d0:	mov    (%r9,%r10,8),%esi
  1c43d4:	cmp    $0xffffffff,%esi
  1c43d7:	je     1c4492 <PRIVATE0000000000134a87+0x4422>
  1c43dd:	cmp    $0xfffffffe,%esi
  1c43e0:	jne    1c43ea <PRIVATE0000000000134a87+0x437a>
  1c43e2:	mov    (%r11,%r8,1),%rsi
  1c43e6:	mov    %rsi,(%r11,%rcx,1)
  1c43ea:	mov    0x4(%r9,%r10,8),%esi
  1c43ef:	cmp    $0xffffffff,%esi
  1c43f2:	je     1c4483 <PRIVATE0000000000134a87+0x4413>
  1c43f8:	cmp    $0xfffffffe,%esi
  1c43fb:	jne    1c4407 <PRIVATE0000000000134a87+0x4397>
  1c43fd:	mov    0x8(%r11,%r8,1),%rsi
  1c4402:	mov    %rsi,0x8(%r11,%rcx,1)
  1c4407:	inc    %r10
  1c440a:	add    $0x10,%r11
  1c440e:	cmp    %rax,%r10
  1c4411:	jb     1c43d0 <PRIVATE0000000000134a87+0x4360>
  1c4413:	lea    0x1(%r10,%r10,1),%edx
  1c4418:	lea    -0x1(%rdx),%eax
  1c441b:	cmp    0x1f58(%rsp),%eax
  1c4422:	jae    1c08d5 <PRIVATE0000000000134a87+0x865>
  1c4428:	movslq %edx,%rdx
  1c442b:	mov    0x1fc0(%rsp),%rax
  1c4433:	mov    -0x4(%rax,%rdx,4),%eax
  1c4437:	cmp    $0xffffffff,%eax
  1c443a:	je     1c4464 <PRIVATE0000000000134a87+0x43f4>
  1c443c:	cmp    $0xfffffffe,%eax
  1c443f:	jne    1c08d5 <PRIVATE0000000000134a87+0x865>
  1c4445:	mov    0x1f88(%rsp),%rax
  1c444d:	mov    0x1fa8(%rsp),%r8
  1c4455:	mov    -0x8(%rax,%rdx,8),%rcx
  1c445a:	mov    %rcx,-0x8(%r8,%rdx,8)
  1c445f:	jmp    1c08d5 <PRIVATE0000000000134a87+0x865>
  1c4464:	mov    0x1f90(%rsp),%rax
  1c446c:	mov    0x1fa8(%rsp),%r8
  1c4474:	mov    -0x8(%rax,%rdx,8),%rcx
  1c4479:	mov    %rcx,-0x8(%r8,%rdx,8)
  1c447e:	jmp    1c08d5 <PRIVATE0000000000134a87+0x865>
  1c4483:	mov    0x8(%r11,%rdx,1),%rsi
  1c4488:	mov    %rsi,0x8(%r11,%rcx,1)
  1c448d:	jmp    1c4407 <PRIVATE0000000000134a87+0x4397>
  1c4492:	mov    (%r11,%rdx,1),%rsi
  1c4496:	mov    %rsi,(%r11,%rcx,1)
  1c449a:	jmp    1c43ea <PRIVATE0000000000134a87+0x437a>
  1c449f:	xor    %eax,%eax
  1c44a1:	jmp    1c24b5 <PRIVATE0000000000134a87+0x2445>
  1c44a6:	xor    %eax,%eax
  1c44a8:	jmp    1c2805 <PRIVATE0000000000134a87+0x2795>
  1c44ad:	mov    0x1ee8(%rsp),%rbp
  1c44b5:	mov    0x1ef0(%rsp),%r12
  1c44bd:	mov    0x1f18(%rsp),%rbx
  1c44c5:	mov    0x1f60(%rsp),%r13
  1c44cd:	mov    0x1f30(%rsp),%r15
  1c44d5:	mov    0x20e8(%rsp),%rax
  1c44dd:	movl   $0x1,(%rax)
  1c44e3:	jmp    1c08d5 <PRIVATE0000000000134a87+0x865>
  1c44e8:	mov    0x20e8(%rsp),%rdx
  1c44f0:	mov    $0x1,%eax
  1c44f5:	mov    %eax,0xf0(%r14)
  1c44fc:	mov    0x1ee8(%rsp),%rbp
  1c4504:	mov    0x1ef0(%rsp),%r12
  1c450c:	mov    0x1f18(%rsp),%rbx
  1c4514:	mov    0x1f60(%rsp),%r13
  1c451c:	mov    0x1f30(%rsp),%r15
  1c4524:	mov    %eax,(%rdx)
  1c4526:	jmp    1c08d5 <PRIVATE0000000000134a87+0x865>
  1c452b:	movsd  0x289ce75(%rip),%xmm0        # 2a613a8 <__STRLITPACK_7+0xa7c>
  1c4533:	comisd %xmm0,%xmm2
  1c4537:	jbe    1c3aed <PRIVATE0000000000134a87+0x3a7d>
  1c453d:	cmpl   $0x0,0x1fa0(%rsp)
  1c4545:	jne    1c3aed <PRIVATE0000000000134a87+0x3a7d>
  1c454b:	mov    0x20e8(%rsp),%rax
  1c4553:	mov    0x1ee8(%rsp),%rbp
  1c455b:	mov    0x1ef0(%rsp),%r12
  1c4563:	mov    0x1f18(%rsp),%rbx
  1c456b:	mov    0x1f60(%rsp),%r13
  1c4573:	mov    0x1f30(%rsp),%r15
  1c457b:	movl   $0x1,(%rax)
  1c4581:	jmp    1c08d5 <PRIVATE0000000000134a87+0x865>
  1c4586:	xor    %ebx,%ebx
  1c4588:	pxor   %xmm2,%xmm2
  1c458c:	jmp    1c3a73 <PRIVATE0000000000134a87+0x3a03>
  1c4591:	mov    %ebx,%eax
  1c4593:	mov    0x1ee8(%rsp),%rbp
  1c459b:	mov    0x1ef0(%rsp),%r12
  1c45a3:	mov    0x1f18(%rsp),%rbx
  1c45ab:	mov    0x1f60(%rsp),%r13
  1c45b3:	mov    0x1f30(%rsp),%r15
  1c45bb:	mov    %eax,0x1f70(%rsp)
  1c45c2:	jmp    1c08d5 <PRIVATE0000000000134a87+0x865>
  1c45c7:	xor    %r13d,%r13d
  1c45ca:	xor    %r12d,%r12d
  1c45cd:	mov    0x1ea0(%rsp),%ebp
  1c45d4:	mov    %r13,%rdx
  1c45d7:	mov    0x1f40(%rsp),%rbx
  1c45df:	movl   $0xffffffff,0x209c(%rsp)
  1c45ea:	cmpl   $0x0,0x8(%rsp)
  1c45ef:	jle    1c460a <PRIVATE0000000000134a87+0x459a>
  1c45f1:	mov    0x20(%rsp),%rax
  1c45f6:	movl   $0x0,(%rax,%rdx,4)
  1c45fd:	inc    %rdx
  1c4600:	movslq 0x8(%rsp),%r10
  1c4605:	cmp    %r10,%rdx
  1c4608:	jl     1c45f6 <PRIVATE0000000000134a87+0x4586>
  1c460a:	mov    0x1850(%rsp),%r11d
  1c4612:	movslq %r11d,%r11
  1c4615:	movslq 0x2c(%rsp),%rdx
  1c461a:	lea    (%r11,%r11,2),%r10
  1c461e:	shl    $0x5,%r10
  1c4622:	cmp    0x184c(%rsp),%r11d
  1c462a:	jge    1c4650 <PRIVATE0000000000134a87+0x45e0>
  1c462c:	mov    0x10(%rsp),%rax
  1c4631:	mov    0x90(%rsp,%r10,1),%ecx
  1c4639:	inc    %r11d
  1c463c:	mov    %ecx,(%rax,%rdx,4)
  1c463f:	add    $0x60,%r10
  1c4643:	inc    %rdx
  1c4646:	cmp    0x184c(%rsp),%r11d
  1c464e:	jl     1c4631 <PRIVATE0000000000134a87+0x45c1>
  1c4650:	movl   $0x0,0x1850(%rsp)
  1c465b:	cmpl   $0x0,0x1848(%rsp)
  1c4663:	jle    1c4670 <PRIVATE0000000000134a87+0x4600>
  1c4665:	movl   $0x0,0x184c(%rsp)
  1c4670:	mov    0x28(%rsp),%r11d
  1c4675:	movslq %r11d,%r10
  1c4678:	cmp    0xc(%rsp),%r11d
  1c467d:	jge    1c469b <PRIVATE0000000000134a87+0x462b>
  1c467f:	mov    0x10(%rsp),%rax
  1c4684:	mov    (%rax,%r10,4),%ecx
  1c4688:	inc    %r11d
  1c468b:	mov    %ecx,(%rax,%rdx,4)
  1c468e:	inc    %r10
  1c4691:	inc    %rdx
  1c4694:	cmp    0xc(%rsp),%r11d
  1c4699:	jl     1c4684 <PRIVATE0000000000134a87+0x4614>
  1c469b:	mov    %edx,0xc(%rsp)
  1c469f:	mov    %r12d,0x2c(%rsp)
  1c46a4:	mov    %r12d,0x28(%rsp)
  1c46a9:	test   %edx,%edx
  1c46ab:	je     1c4021 <PRIVATE0000000000134a87+0x3fb1>
  1c46b1:	jmp    1c3c0c <PRIVATE0000000000134a87+0x3b9c>
  1c46b6:	dec    %r11d
  1c46b9:	jmp    1c3224 <PRIVATE0000000000134a87+0x31b4>
  1c46be:	mov    0x1f90(%rsp),%rbx
  1c46c6:	movsd  (%rbx,%rdx,8),%xmm0
  1c46cb:	xorps  0x289cbbe(%rip),%xmm0        # 2a61290 <__STRLITPACK_7+0x964>
  1c46d2:	jmp    1c311a <PRIVATE0000000000134a87+0x30aa>
  1c46d7:	cmpl   $0x0,(%rax)
  1c46da:	jg     1c3199 <PRIVATE0000000000134a87+0x3129>
  1c46e0:	jmp    1c3b9f <PRIVATE0000000000134a87+0x3b2f>
  1c46e5:	mov    0x1fc0(%rsp),%r10
  1c46ed:	lea    0x0(,%rbx,4),%rdx
  1c46f5:	mov    (%r10,%rdx,1),%r10d
  1c46f9:	cmp    $0xffffffff,%r10d
  1c46fd:	je     1c4729 <PRIVATE0000000000134a87+0x46b9>
  1c46ff:	cmp    $0xfffffffd,%r10d
  1c4703:	je     1c4713 <PRIVATE0000000000134a87+0x46a3>
  1c4705:	mov    0x1f88(%rsp),%r10
  1c470d:	subsd  (%r10,%rbx,8),%xmm0
  1c4713:	pxor   %xmm1,%xmm1
  1c4717:	ucomisd %xmm1,%xmm0
  1c471b:	jne    1c306b <PRIVATE0000000000134a87+0x2ffb>
  1c4721:	jp     1c306b <PRIVATE0000000000134a87+0x2ffb>
  1c4727:	jmp    1c46d7 <PRIVATE0000000000134a87+0x4667>
  1c4729:	mov    0x1f90(%rsp),%r10
  1c4731:	subsd  (%r10,%rbx,8),%xmm0
  1c4737:	jmp    1c4713 <PRIVATE0000000000134a87+0x46a3>
  1c4739:	mov    0x1ee8(%rsp),%rbp
  1c4741:	mov    0x1ef0(%rsp),%r12
  1c4749:	mov    0x1f18(%rsp),%rbx
  1c4751:	mov    0x1f60(%rsp),%r13
  1c4759:	mov    0x1f30(%rsp),%r15
  1c4761:	mov    0x20e8(%rsp),%rax
  1c4769:	movl   $0x1,(%rax)
  1c476f:	jmp    1c08d5 <PRIVATE0000000000134a87+0x865>
  1c4774:	xor    %eax,%eax
  1c4776:	pxor   %xmm2,%xmm2
  1c477a:	jmp    1c4211 <PRIVATE0000000000134a87+0x41a1>
  1c477f:	mov    %r8d,0x1e90(%rsp)
  1c4787:	mov    0x4b8(%r14),%rax
  1c478e:	mov    0x100(%rax),%rdx
  1c4795:	cmpl   $0xffffffff,0x2ba0(%rdx)
  1c479c:	jne    1c47de <PRIVATE0000000000134a87+0x476e>
  1c479e:	mov    $0x66666667,%eax
  1c47a3:	mov    $0x5,%r8d
  1c47a9:	mov    0x46c(%r14),%ecx
  1c47b0:	imul   %ecx
  1c47b2:	sar    $0x1f,%ecx
  1c47b5:	sar    $0x2,%edx
  1c47b8:	sub    %ecx,%edx
  1c47ba:	cmp    $0x5,%edx
  1c47bd:	mov    0x1e90(%rsp),%r9d
  1c47c5:	mov    %r9d,%r10d
  1c47c8:	cmovg  %edx,%r8d
  1c47cc:	neg    %r10d
  1c47cf:	cmp    %r8d,%r9d
  1c47d2:	cmovge %r9d,%r10d
  1c47d6:	mov    %r10d,0x1fb0(%rsp)
  1c47de:	xor    %eax,%eax
  1c47e0:	movl   $0x0,0x2098(%rsp)
  1c47eb:	cmpl   $0x0,0x1fb0(%rsp)
  1c47f3:	jle    1c437e <PRIVATE0000000000134a87+0x430e>
  1c47f9:	movsd  0x289cb4f(%rip),%xmm0        # 2a61350 <__STRLITPACK_7+0xa24>
  1c4801:	mov    0x1fb0(%rsp),%edx
  1c4808:	mov    0x1fb8(%rsp),%rcx
  1c4810:	mov    0x1fc0(%rsp),%r8
  1c4818:	mov    0x1f88(%rsp),%r9
  1c4820:	mov    0x1f90(%rsp),%r10
  1c4828:	mov    0x1fa8(%rsp),%r11
  1c4830:	movslq %eax,%rax
  1c4833:	movslq (%rcx,%rax,4),%rax
  1c4837:	movsd  (%r11,%rax,8),%xmm2
  1c483d:	movsd  (%r10,%rax,8),%xmm1
  1c4843:	mulsd  %xmm0,%xmm2
  1c4847:	addsd  (%r9,%rax,8),%xmm1
  1c484d:	comisd %xmm2,%xmm1
  1c4851:	jbe    1c485d <PRIVATE0000000000134a87+0x47ed>
  1c4853:	movl   $0xffffffff,(%r8,%rax,4)
  1c485b:	jmp    1c4865 <PRIVATE0000000000134a87+0x47f5>
  1c485d:	movl   $0xfffffffe,(%r8,%rax,4)
  1c4865:	mov    0x2098(%rsp),%eax
  1c486c:	inc    %eax
  1c486e:	mov    %eax,0x2098(%rsp)
  1c4875:	cmp    %edx,%eax
  1c4877:	jl     1c4830 <PRIVATE0000000000134a87+0x47c0>
  1c4879:	mov    %edx,0x1fb0(%rsp)
  1c4880:	jmp    1c08d5 <PRIVATE0000000000134a87+0x865>
  1c4885:	cmp    0x1e90(%rsp),%edx
  1c488c:	jge    1c4787 <PRIVATE0000000000134a87+0x4717>
  1c4892:	mov    0x1e90(%rsp),%eax
  1c4899:	neg    %eax
  1c489b:	mov    %eax,0x1fb0(%rsp)
  1c48a2:	jmp    1c47de <PRIVATE0000000000134a87+0x476e>
  1c48a7:	mov    %r8d,0x1e90(%rsp)
  1c48af:	jmp    1c4885 <PRIVATE0000000000134a87+0x4815>
  1c48b1:	movsd  0x289caef(%rip),%xmm0        # 2a613a8 <__STRLITPACK_7+0xa7c>
  1c48b9:	comisd %xmm0,%xmm2
  1c48bd:	jbe    1c400a <PRIVATE0000000000134a87+0x3f9a>
  1c48c3:	cmpl   $0x0,0x1fa0(%rsp)
  1c48cb:	jne    1c400a <PRIVATE0000000000134a87+0x3f9a>
  1c48d1:	mov    0x20e8(%rsp),%rax
  1c48d9:	mov    0x1ee8(%rsp),%rbp
  1c48e1:	mov    0x1ef0(%rsp),%r12
  1c48e9:	mov    0x1f18(%rsp),%rbx
  1c48f1:	mov    0x1f60(%rsp),%r13
  1c48f9:	mov    0x1f30(%rsp),%r15
  1c4901:	movl   $0x1,(%rax)
  1c4907:	jmp    1c08d5 <PRIVATE0000000000134a87+0x865>
  1c490c:	mov    %r12d,%ebp
  1c490f:	pxor   %xmm2,%xmm2
  1c4913:	jmp    1c3fab <PRIVATE0000000000134a87+0x3f3b>
  1c4918:	mov    %ebp,%eax
  1c491a:	mov    0x1ee8(%rsp),%rbp
  1c4922:	mov    0x1ef0(%rsp),%r12
  1c492a:	mov    0x1f18(%rsp),%rbx
  1c4932:	mov    0x1f60(%rsp),%r13
  1c493a:	mov    0x1f30(%rsp),%r15
  1c4942:	mov    %eax,0x1f70(%rsp)
  1c4949:	jmp    1c08d5 <PRIVATE0000000000134a87+0x865>
  1c494e:	movsd  0x1eb0(%rsp),%xmm1
  1c4957:	comisd %xmm4,%xmm1
  1c495b:	jbe    1c2a7c <PRIVATE0000000000134a87+0x2a0c>
  1c4961:	movsd  0x1ec8(%rsp),%xmm1
  1c496a:	comisd 0x1ee0(%rsp),%xmm1
  1c4973:	ja     1c2977 <PRIVATE0000000000134a87+0x2907>
  1c4979:	jmp    1c2a7c <PRIVATE0000000000134a87+0x2a0c>
  1c497e:	mov    0x1f90(%rsp),%rdx
  1c4986:	movsd  0x1f00(%rsp),%xmm1
  1c498f:	comisd (%rdx,%rbx,8),%xmm1
  1c4994:	jb     1c49b7 <PRIVATE0000000000134a87+0x4947>
  1c4996:	mov    0x1f88(%rsp),%rdx
  1c499e:	movsd  (%rdx,%rbx,8),%xmm1
  1c49a3:	comisd 0x1fc8(%rsp),%xmm1
  1c49ac:	jb     1c49b7 <PRIVATE0000000000134a87+0x4947>
  1c49ae:	mov    0x328(%r14),%ecx
  1c49b5:	jmp    1c49e2 <PRIVATE0000000000134a87+0x4972>
  1c49b7:	mov    0x328(%r14),%ecx
  1c49be:	test   %ecx,%ecx
  1c49c0:	jne    1c4d41 <PRIVATE0000000000134a87+0x4cd1>
  1c49c6:	mov    0x1f88(%rsp),%rdx
  1c49ce:	movsd  (%rdx,%rbx,8),%xmm1
  1c49d3:	comisd 0x1fc8(%rsp),%xmm1
  1c49dc:	jb     1c4d35 <PRIVATE0000000000134a87+0x4cc5>
  1c49e2:	mov    0x2090(%rsp),%r9
  1c49ea:	neg    %ecx
  1c49ec:	inc    %ecx
  1c49ee:	mov    %ecx,0x328(%r14)
  1c49f5:	movslq (%r9),%rdx
  1c49f8:	test   %rdx,%rdx
  1c49fb:	jle    1c4af8 <PRIVATE0000000000134a87+0x4a88>
  1c4a01:	mov    0x10(%r9),%r11
  1c4a05:	cmp    $0x8,%rdx
  1c4a09:	jl     1c4db7 <PRIVATE0000000000134a87+0x4d47>
  1c4a0f:	mov    %r11,%r10
  1c4a12:	and    $0xf,%r10
  1c4a16:	test   %r10d,%r10d
  1c4a19:	je     1c4a2e <PRIVATE0000000000134a87+0x49be>
  1c4a1b:	test   $0x7,%r10d
  1c4a22:	jne    1c4db7 <PRIVATE0000000000134a87+0x4d47>
  1c4a28:	mov    $0x1,%r10d
  1c4a2e:	lea    0x8(%r10),%eax
  1c4a32:	cmp    %rax,%rdx
  1c4a35:	jl     1c4db7 <PRIVATE0000000000134a87+0x4d47>
  1c4a3b:	mov    %edx,%eax
  1c4a3d:	mov    %eax,%ecx
  1c4a3f:	sub    %r10d,%ecx
  1c4a42:	and    $0x7,%ecx
  1c4a45:	sub    %ecx,%eax
  1c4a47:	xor    %ecx,%ecx
  1c4a49:	mov    %r10d,%esi
  1c4a4c:	test   %r10d,%r10d
  1c4a4f:	jbe    1c4a6c <PRIVATE0000000000134a87+0x49fc>
  1c4a51:	movsd  (%r11,%rcx,8),%xmm0
  1c4a57:	xorps  0x289c832(%rip),%xmm0        # 2a61290 <__STRLITPACK_7+0x964>
  1c4a5e:	movsd  %xmm0,(%r11,%rcx,8)
  1c4a64:	inc    %rcx
  1c4a67:	cmp    %rsi,%rcx
  1c4a6a:	jb     1c4a51 <PRIVATE0000000000134a87+0x49e1>
  1c4a6c:	movslq %eax,%r10
  1c4a6f:	nop
  1c4a70:	movups (%r11,%rsi,8),%xmm0
  1c4a75:	movups 0x10(%r11,%rsi,8),%xmm1
  1c4a7b:	movups 0x20(%r11,%rsi,8),%xmm2
  1c4a81:	movups 0x30(%r11,%rsi,8),%xmm3
  1c4a87:	xorps  0x289c822(%rip),%xmm0        # 2a612b0 <__STRLITPACK_7+0x984>
  1c4a8e:	xorps  0x289c81b(%rip),%xmm1        # 2a612b0 <__STRLITPACK_7+0x984>
  1c4a95:	xorps  0x289c814(%rip),%xmm2        # 2a612b0 <__STRLITPACK_7+0x984>
  1c4a9c:	xorps  0x289c80d(%rip),%xmm3        # 2a612b0 <__STRLITPACK_7+0x984>
  1c4aa3:	movups %xmm0,(%r11,%rsi,8)
  1c4aa8:	movups %xmm1,0x10(%r11,%rsi,8)
  1c4aae:	movups %xmm2,0x20(%r11,%rsi,8)
  1c4ab4:	movups %xmm3,0x30(%r11,%rsi,8)
  1c4aba:	add    $0x8,%rsi
  1c4abe:	cmp    %r10,%rsi
  1c4ac1:	jb     1c4a70 <PRIVATE0000000000134a87+0x4a00>
  1c4ac3:	xor    %ecx,%ecx
  1c4ac5:	lea    0x1(%rax),%r10d
  1c4ac9:	movslq %r10d,%r10
  1c4acc:	cmp    %rdx,%r10
  1c4acf:	ja     1c4af8 <PRIVATE0000000000134a87+0x4a88>
  1c4ad1:	movslq %eax,%rax
  1c4ad4:	sub    %rax,%rdx
  1c4ad7:	lea    (%r11,%rax,8),%r10
  1c4adb:	movsd  (%r10),%xmm0
  1c4ae0:	inc    %rcx
  1c4ae3:	xorps  0x289c7a6(%rip),%xmm0        # 2a61290 <__STRLITPACK_7+0x964>
  1c4aea:	movsd  %xmm0,(%r10)
  1c4aef:	add    $0x8,%r10
  1c4af3:	cmp    %rdx,%rcx
  1c4af6:	jb     1c4adb <PRIVATE0000000000134a87+0x4a6b>
  1c4af8:	cmpl   $0x0,0x1f28(%rsp)
  1c4b00:	je     1c4c72 <PRIVATE0000000000134a87+0x4c02>
  1c4b06:	add    $0xffffffffffffffd0,%rsp
  1c4b0a:	mov    %r14,%rdi
  1c4b0d:	mov    0x2038(%rsp),%r11
  1c4b15:	mov    %r9,%rdx
  1c4b18:	mov    0x8(%r11),%r10
  1c4b1c:	mov    %r10,(%rsp)
  1c4b20:	mov    0x10(%r11),%r10
  1c4b24:	mov    %r10,0x8(%rsp)
  1c4b29:	mov    %rbp,0x10(%rsp)
  1c4b2e:	mov    %r12,0x18(%rsp)
  1c4b33:	mov    %r15,0x20(%rsp)
  1c4b38:	movsd  0x1ff8(%rsp),%xmm2
  1c4b41:	movsd  0x20a0(%rsp),%xmm0
  1c4b4a:	movaps %xmm2,%xmm3
  1c4b4d:	movsd  0x20a8(%rsp),%xmm1
  1c4b56:	mov    0x2098(%rsp),%esi
  1c4b5d:	mov    0x2040(%rsp),%rcx
  1c4b65:	mov    0x2020(%rsp),%r8
  1c4b6d:	mov    0x2048(%rsp),%r9
  1c4b75:	movsd  %xmm4,0x1f00(%rsp)
  1c4b7e:	movsd  %xmm5,0x1f08(%rsp)
  1c4b87:	call   eefa0 <PRIVATE0000000000054eef>
  1c4b8c:	movsd  0x1f08(%rsp),%xmm5
  1c4b95:	mov    %eax,%esi
  1c4b97:	movsd  0x1f00(%rsp),%xmm4
  1c4ba0:	add    $0x30,%rsp
  1c4ba4:	test   %esi,%esi
  1c4ba6:	jl     1c4bca <PRIVATE0000000000134a87+0x4b5a>
  1c4ba8:	movsd  0x1ef8(%rsp),%xmm1
  1c4bb1:	andps  0x289c6e8(%rip),%xmm1        # 2a612a0 <__STRLITPACK_7+0x974>
  1c4bb8:	movsd  0x289c730(%rip),%xmm0        # 2a612f0 <__STRLITPACK_7+0x9c4>
  1c4bc0:	comisd %xmm1,%xmm0
  1c4bc4:	jbe    1c4d0c <PRIVATE0000000000134a87+0x4c9c>
  1c4bca:	add    $0xffffffffffffffd0,%rsp
  1c4bce:	mov    %r14,%rdi
  1c4bd1:	mov    0x20c0(%rsp),%rdx
  1c4bd9:	mov    %rbp,(%rsp)
  1c4bdd:	mov    %r12,0x8(%rsp)
  1c4be2:	mov    0x1f20(%rsp),%rbp
  1c4bea:	mov    %rbp,0x10(%rsp)
  1c4bef:	mov    0x1f18(%rsp),%r10
  1c4bf7:	mov    %r10,0x18(%rsp)
  1c4bfc:	mov    %r15,0x20(%rsp)
  1c4c01:	movsd  0x1ff8(%rsp),%xmm2
  1c4c0a:	movsd  0x20a0(%rsp),%xmm0
  1c4c13:	movaps %xmm2,%xmm3
  1c4c16:	movsd  0x20a8(%rsp),%xmm1
  1c4c1f:	mov    0x2098(%rsp),%esi
  1c4c26:	mov    0x2040(%rsp),%rcx
  1c4c2e:	mov    0x2020(%rsp),%r8
  1c4c36:	mov    0x2048(%rsp),%r9
  1c4c3e:	movsd  %xmm4,0x1f00(%rsp)
  1c4c47:	movsd  %xmm5,0x1f08(%rsp)
  1c4c50:	call   f0120 <PRIVATE0000000000056973>
  1c4c55:	movsd  0x1f08(%rsp),%xmm5
  1c4c5e:	mov    %eax,%esi
  1c4c60:	movsd  0x1f00(%rsp),%xmm4
  1c4c69:	add    $0x30,%rsp
  1c4c6d:	jmp    1c4d0c <PRIVATE0000000000134a87+0x4c9c>
  1c4c72:	push   %r15
  1c4c74:	push   %r12
  1c4c76:	push   %rbp
  1c4c77:	mov    0x2020(%rsp),%rbp
  1c4c7f:	mov    %r14,%rdi
  1c4c82:	xor    %edx,%edx
  1c4c84:	push   0x10(%rbp)
  1c4c87:	push   0x8(%rbp)
  1c4c8a:	push   0x2040(%rsp)
  1c4c91:	push   0x2020(%rsp)
  1c4c98:	push   0x2048(%rsp)
  1c4c9f:	movsd  %xmm4,0x1f10(%rsp)
  1c4ca8:	movsd  0x2008(%rsp),%xmm2
  1c4cb1:	movsd  0x20b0(%rsp),%xmm0
  1c4cba:	movaps %xmm2,%xmm3
  1c4cbd:	movsd  0x20b8(%rsp),%xmm1
  1c4cc6:	mov    0x20a8(%rsp),%esi
  1c4ccd:	mov    0x2078(%rsp),%rcx
  1c4cd5:	mov    0x2028(%rsp),%r8
  1c4cdd:	movsd  %xmm5,0x1f18(%rsp)
  1c4ce6:	call   ef960 <PRIVATE0000000000055f68>
  1c4ceb:	movsd  0x1f18(%rsp),%xmm5
  1c4cf4:	mov    %eax,%esi
  1c4cf6:	movsd  0x1f10(%rsp),%xmm4
  1c4cff:	add    $0x40,%rsp
  1c4d03:	cmp    $0xffffffff,%esi
  1c4d06:	je     1c4d99 <PRIVATE0000000000134a87+0x4d29>
  1c4d0c:	mov    %esi,0x2f8(%r14)
  1c4d13:	movsd  0x1eb8(%rsp),%xmm0
  1c4d1c:	ucomisd 0x1fc8(%rsp),%xmm0
  1c4d25:	jp     1c4d29 <PRIVATE0000000000134a87+0x4cb9>
  1c4d27:	je     1c4d4d <PRIVATE0000000000134a87+0x4cdd>
  1c4d29:	mov    0x328(%r14),%ecx
  1c4d30:	jmp    1c290d <PRIVATE0000000000134a87+0x289d>
  1c4d35:	mov    0x2f8(%r14),%esi
  1c4d3c:	jmp    1c2924 <PRIVATE0000000000134a87+0x28b4>
  1c4d41:	mov    0x2f8(%r14),%esi
  1c4d48:	jmp    1c2911 <PRIVATE0000000000134a87+0x28a1>
  1c4d4d:	mov    0x1f90(%rsp),%rdx
  1c4d55:	movsd  0x1f00(%rsp),%xmm1
  1c4d5e:	comisd (%rdx,%rbx,8),%xmm1
  1c4d63:	jb     1c4d29 <PRIVATE0000000000134a87+0x4cb9>
  1c4d65:	mov    0x1f88(%rsp),%rdx
  1c4d6d:	movsd  (%rdx,%rbx,8),%xmm1
  1c4d72:	comisd 0x1fc8(%rsp),%xmm1
  1c4d7b:	jb     1c4d29 <PRIVATE0000000000134a87+0x4cb9>
  1c4d7d:	movl   $0xffffffff,0x2f8(%r14)
  1c4d88:	mov    $0xffffffff,%esi
  1c4d8d:	mov    0x328(%r14),%ecx
  1c4d94:	jmp    1c290d <PRIVATE0000000000134a87+0x289d>
  1c4d99:	movsd  0x1fc8(%rsp),%xmm0
  1c4da2:	movsd  %xmm0,0x1eb8(%rsp)
  1c4dab:	mov    0x2f8(%r14),%esi
  1c4db2:	jmp    1c4d1c <PRIVATE0000000000134a87+0x4cac>
  1c4db7:	xor    %eax,%eax
  1c4db9:	jmp    1c4ac3 <PRIVATE0000000000134a87+0x4a53>
  1c4dbe:	mov    %eax,0xf0(%r14)
  1c4dc5:	jne    1c4dfd <PRIVATE0000000000134a87+0x4d8d>
  1c4dc7:	jmp    1c2164 <PRIVATE0000000000134a87+0x20f4>
  1c4dcc:	xor    %esi,%esi
  1c4dce:	mov    $0x2,%edx
  1c4dd3:	xor    %ecx,%ecx
  1c4dd5:	xor    %r8d,%r8d
  1c4dd8:	mov    %r15,%r9
  1c4ddb:	mov    0x1f30(%rsp),%rdi
  1c4de3:	call   7cd7f0 <PRIVATE00000000009a91f3>
  1c4de8:	mov    %eax,0x1f70(%rsp)
  1c4def:	cmpl   $0x0,0x1f70(%rsp)
  1c4df7:	je     1c212a <PRIVATE0000000000134a87+0x20ba>
  1c4dfd:	mov    0x1ee8(%rsp),%rbp
  1c4e05:	mov    0x1ef0(%rsp),%r12
  1c4e0d:	mov    0x1f18(%rsp),%rbx
  1c4e15:	mov    0x1f60(%rsp),%r13
  1c4e1d:	mov    0x1f30(%rsp),%r15
  1c4e25:	jmp    1c08d5 <PRIVATE0000000000134a87+0x865>
  1c4e2a:	mov    0xc(%rsp),%edx
  1c4e2e:	mov    0x2c(%rsp),%eax
  1c4e32:	jmp    1c3be7 <PRIVATE0000000000134a87+0x3b77>
  1c4e37:	mov    0x20f8(%rsp),%rdx
  1c4e3f:	mov    0x20e8(%rsp),%rax
  1c4e47:	incl   0x178(%r14)
  1c4e4e:	mov    0x1ee8(%rsp),%rbp
  1c4e56:	lea    0xc0(%rdx),%rcx
  1c4e5d:	mov    0x1ef0(%rsp),%r12
  1c4e65:	mov    0x1f18(%rsp),%rbx
  1c4e6d:	mov    0x1f60(%rsp),%r13
  1c4e75:	mov    0x1f30(%rsp),%r15
  1c4e7d:	movl   $0x1,(%rax)
  1c4e83:	mov    %rcx,0x1f80(%rsp)
  1c4e8b:	jmp    1c08d5 <PRIVATE0000000000134a87+0x865>
  1c4e90:	mov    0x20f8(%rsp),%rdx
  1c4e98:	mov    0x20e8(%rsp),%rax
  1c4ea0:	mov    0x1ee8(%rsp),%rbp
  1c4ea8:	mov    0x1ef0(%rsp),%r12
  1c4eb0:	lea    0xc0(%rdx),%rcx
  1c4eb7:	mov    0x1f18(%rsp),%rbx
  1c4ebf:	mov    0x1f60(%rsp),%r13
  1c4ec7:	mov    0x1f30(%rsp),%r15
  1c4ecf:	movl   $0x1,(%rax)
  1c4ed5:	mov    %rcx,0x1f80(%rsp)
  1c4edd:	jmp    1c08d5 <PRIVATE0000000000134a87+0x865>
  1c4ee2:	xor    %eax,%eax
  1c4ee4:	movl   $0x0,0x2098(%rsp)
  1c4eef:	pxor   %xmm2,%xmm2
  1c4ef3:	jmp    1c1786 <PRIVATE0000000000134a87+0x1716>
  1c4ef8:	movsd  0x289c498(%rip),%xmm4        # 2a61398 <__STRLITPACK_7+0xa6c>
  1c4f00:	mulsd  %xmm4,%xmm1
  1c4f04:	comisd %xmm1,%xmm2
  1c4f08:	ja     1c4f1c <PRIVATE0000000000134a87+0x4eac>
  1c4f0a:	movsd  0x289c4a6(%rip),%xmm1        # 2a613b8 <__STRLITPACK_7+0xa8c>
  1c4f12:	comisd %xmm1,%xmm3
  1c4f16:	jbe    1c14fc <PRIVATE0000000000134a87+0x148c>
  1c4f1c:	movsd  0x289c49c(%rip),%xmm0        # 2a613c0 <__STRLITPACK_7+0xa94>
  1c4f24:	jmp    1c14fc <PRIVATE0000000000134a87+0x148c>
  1c4f29:	cmp    $0xfffffffd,%r14d
  1c4f2d:	jl     1c0f44 <PRIVATE0000000000134a87+0xed4>
  1c4f33:	cmp    $0xffffffff,%r14d
  1c4f37:	je     1c4f84 <PRIVATE0000000000134a87+0x4f14>
  1c4f39:	cmp    $0xfffffffd,%r14d
  1c4f3d:	jne    1c4f68 <PRIVATE0000000000134a87+0x4ef8>
  1c4f3f:	movslq %eax,%rax
  1c4f42:	movslq %r12d,%r12
  1c4f45:	mov    0x2058(%rsp),%r14
  1c4f4d:	movsd  (%r14,%rax,8),%xmm0
  1c4f53:	mov    %r8d,(%rdi,%r12,4)
  1c4f57:	movsd  %xmm0,0x0(%r13,%r12,8)
  1c4f5e:	inc    %r12d
  1c4f61:	mov    0x2098(%rsp),%eax
  1c4f68:	movslq %eax,%rax
  1c4f6b:	mov    0x10(%rdx),%r8
  1c4f6f:	movsd  (%r8,%rax,8),%xmm1
  1c4f75:	subsd  %xmm0,%xmm1
  1c4f79:	movsd  %xmm1,(%r8,%rax,8)
  1c4f7f:	jmp    1c0f44 <PRIVATE0000000000134a87+0xed4>
  1c4f84:	movsd  0x289c364(%rip),%xmm0        # 2a612f0 <__STRLITPACK_7+0x9c4>
  1c4f8c:	lea    0x1(%r10),%r8d
  1c4f90:	mulsd  %xmm5,%xmm0
  1c4f94:	lea    0x1(%r11),%r14d
  1c4f98:	movsd  (%rsi,%r15,8),%xmm2
  1c4f9e:	lea    0x1(%r9),%r15d
  1c4fa2:	comisd %xmm0,%xmm2
  1c4fa6:	movsd  0x289c3f2(%rip),%xmm1        # 2a613a0 <__STRLITPACK_7+0xa74>
  1c4fae:	movaps %xmm4,%xmm0
  1c4fb1:	mulsd  %xmm5,%xmm1
  1c4fb5:	cmova  %r8d,%r10d
  1c4fb9:	comisd %xmm1,%xmm2
  1c4fbd:	movsd  0x289c313(%rip),%xmm3        # 2a612d8 <__STRLITPACK_7+0x9ac>
  1c4fc5:	mulsd  %xmm5,%xmm3
  1c4fc9:	cmova  %r14d,%r11d
  1c4fcd:	comisd %xmm3,%xmm2
  1c4fd1:	cmova  %r15d,%r9d
  1c4fd5:	jmp    1c4f68 <PRIVATE0000000000134a87+0x4ef8>
  1c4fd7:	cmp    $0xfffffffd,%edx
  1c4fda:	jge    1c4ff3 <PRIVATE0000000000134a87+0x4f83>
  1c4fdc:	cmp    $0xfffffffc,%edx
  1c4fdf:	je     1c50ee <PRIVATE0000000000134a87+0x507e>
  1c4fe5:	cmp    $0xfffffffb,%edx
  1c4fe8:	je     1c50e3 <PRIVATE0000000000134a87+0x5073>
  1c4fee:	movaps %xmm2,%xmm0
  1c4ff1:	jmp    1c5061 <PRIVATE0000000000134a87+0x4ff1>
  1c4ff3:	cmp    $0xffffffff,%edx
  1c4ff6:	je     1c515b <PRIVATE0000000000134a87+0x50eb>
  1c4ffc:	movsd  (%r10,%rax,8),%xmm6
  1c5002:	cmp    $0xfffffffe,%edx
  1c5005:	je     1c50f8 <PRIVATE0000000000134a87+0x5088>
  1c500b:	movsd  (%r9,%rax,8),%xmm0
  1c5011:	movaps %xmm6,%xmm3
  1c5014:	movsd  (%rsi,%rax,8),%xmm5
  1c5019:	movaps %xmm0,%xmm4
  1c501c:	movslq %edi,%rdx
  1c501f:	addsd  %xmm0,%xmm4
  1c5023:	addsd  %xmm5,%xmm3
  1c5027:	comisd %xmm4,%xmm3
  1c502b:	jbe    1c5043 <PRIVATE0000000000134a87+0x4fd3>
  1c502d:	movaps %xmm0,%xmm3
  1c5030:	mov    0x1ed8(%rsp),%rcx
  1c5038:	subsd  %xmm5,%xmm3
  1c503c:	movsd  %xmm3,(%rcx,%rdx,8)
  1c5041:	jmp    1c5054 <PRIVATE0000000000134a87+0x4fe4>
  1c5043:	mov    0x1ed8(%rsp),%rcx
  1c504b:	subsd  %xmm0,%xmm6
  1c504f:	movsd  %xmm6,(%rcx,%rdx,8)
  1c5054:	mov    0x1fb8(%rsp),%rcx
  1c505c:	inc    %edi
  1c505e:	mov    %eax,(%rcx,%rdx,4)
  1c5061:	ucomisd %xmm2,%xmm0
  1c5065:	jp     1c506d <PRIVATE0000000000134a87+0x4ffd>
  1c5067:	je     1c0e2b <PRIVATE0000000000134a87+0xdbb>
  1c506d:	mov    (%r8,%rax,8),%rbx
  1c5071:	mov    %rbx,%rcx
  1c5074:	cmpl   $0x0,(%r12,%rax,4)
  1c5079:	jle    1c0e2b <PRIVATE0000000000134a87+0xdbb>
  1c507f:	mov    0x10(%r11),%rdx
  1c5083:	mov    0x2028(%rsp),%rsi
  1c508b:	mov    0x2030(%rsp),%r8
  1c5093:	movaps %xmm0,%xmm3
  1c5096:	mulsd  (%rsi,%rcx,8),%xmm3
  1c509b:	movslq (%r8,%rcx,4),%r9
  1c509f:	inc    %rcx
  1c50a2:	mov    %r9d,0x2098(%rsp)
  1c50aa:	movsd  (%rdx,%r9,8),%xmm4
  1c50b0:	subsd  %xmm3,%xmm4
  1c50b4:	movsd  %xmm4,(%rdx,%r9,8)
  1c50ba:	movslq (%r12,%rax,4),%r9
  1c50be:	add    %rbx,%r9
  1c50c1:	cmp    %r9,%rcx
  1c50c4:	jl     1c5093 <PRIVATE0000000000134a87+0x5023>
  1c50c6:	mov    0x1f90(%rsp),%rsi
  1c50ce:	mov    0x2040(%rsp),%r8
  1c50d6:	mov    0x1fa8(%rsp),%r9
  1c50de:	jmp    1c0e2b <PRIVATE0000000000134a87+0xdbb>
  1c50e3:	movsd  (%r10,%rax,8),%xmm0
  1c50e9:	jmp    1c5061 <PRIVATE0000000000134a87+0x4ff1>
  1c50ee:	movsd  (%rsi,%rax,8),%xmm0
  1c50f3:	jmp    1c5061 <PRIVATE0000000000134a87+0x4ff1>
  1c50f8:	movsd  0x289c1f0(%rip),%xmm0        # 2a612f0 <__STRLITPACK_7+0x9c4>
  1c5100:	movaps %xmm6,%xmm4
  1c5103:	mulsd  %xmm1,%xmm0
  1c5107:	subsd  (%r9,%rax,8),%xmm4
  1c510d:	comisd %xmm0,%xmm4
  1c5111:	movsd  0x289c287(%rip),%xmm3        # 2a613a0 <__STRLITPACK_7+0xa74>
  1c5119:	lea    0x1(%rbp),%edx
  1c511c:	mulsd  %xmm1,%xmm3
  1c5120:	cmova  %edx,%ebp
  1c5123:	comisd %xmm3,%xmm4
  1c5127:	movsd  0x289c1a9(%rip),%xmm5        # 2a612d8 <__STRLITPACK_7+0x9ac>
  1c512f:	lea    0x1(%r14),%edx
  1c5133:	mulsd  %xmm1,%xmm5
  1c5137:	movaps %xmm6,%xmm0
  1c513a:	mov    0x1e78(%rsp),%ebx
  1c5141:	lea    0x1(%rbx),%ecx
  1c5144:	cmova  %ecx,%ebx
  1c5147:	comisd %xmm5,%xmm4
  1c514b:	mov    %ebx,0x1e78(%rsp)
  1c5152:	cmova  %edx,%r14d
  1c5156:	jmp    1c5061 <PRIVATE0000000000134a87+0x4ff1>
  1c515b:	movsd  (%r9,%rax,8),%xmm5
  1c5161:	lea    0x1(%rbp),%edx
  1c5164:	movsd  (%rsi,%rax,8),%xmm0
  1c5169:	movsd  0x289c17f(%rip),%xmm3        # 2a612f0 <__STRLITPACK_7+0x9c4>
  1c5171:	subsd  %xmm0,%xmm5
  1c5175:	mulsd  %xmm1,%xmm3
  1c5179:	comisd %xmm3,%xmm5
  1c517d:	movsd  0x289c21b(%rip),%xmm4        # 2a613a0 <__STRLITPACK_7+0xa74>
  1c5185:	mulsd  %xmm1,%xmm4
  1c5189:	cmova  %edx,%ebp
  1c518c:	comisd %xmm4,%xmm5
  1c5190:	movsd  0x289c140(%rip),%xmm6        # 2a612d8 <__STRLITPACK_7+0x9ac>
  1c5198:	lea    0x1(%r14),%edx
  1c519c:	mulsd  %xmm1,%xmm6
  1c51a0:	mov    0x1e78(%rsp),%ebx
  1c51a7:	lea    0x1(%rbx),%ecx
  1c51aa:	cmova  %ecx,%ebx
  1c51ad:	comisd %xmm6,%xmm5
  1c51b1:	mov    %ebx,0x1e78(%rsp)
  1c51b8:	cmova  %edx,%r14d
  1c51bc:	jmp    1c5061 <PRIVATE0000000000134a87+0x4ff1>
  1c51c1:	mov    0x20f8(%rsp),%rax
  1c51c9:	movl   $0x2711,0x1f70(%rsp)
  1c51d4:	lea    0xc0(%rax),%rdx
  1c51db:	mov    %rdx,0x1f80(%rsp)
  1c51e3:	jmp    1c08d5 <PRIVATE0000000000134a87+0x865>
  1c51e8:	xor    %r12d,%r12d
  1c51eb:	xor    %ebp,%ebp
  1c51ed:	jmp    1c077c <PRIVATE0000000000134a87+0x70c>
  1c51f2:	xor    %r13d,%r13d
  1c51f5:	xor    %ebx,%ebx
  1c51f7:	jmp    1c0727 <PRIVATE0000000000134a87+0x6b7>
  1c51fc:	movq   $0x0,0x1f78(%rsp)
  1c5208:	jmp    1c06b2 <PRIVATE0000000000134a87+0x642>
  1c520d:	mov    0x20f8(%rsp),%rax
  1c5215:	mov    0x2100(%rsp),%rsi
  1c521d:	movl   $0x2711,0x1f70(%rsp)
  1c5228:	lea    0xc0(%rax),%rdi
  1c522f:	call   8ead40 <PRIVATE0000000000ade86f>
  1c5234:	mov    0x20f0(%rsp),%rax
  1c523c:	movl   $0x0,(%rax)
  1c5242:	jmp    1c0956 <PRIVATE0000000000134a87+0x8e6>
  1c5247:	movq   $0x0,0x1880(%rsp,%r13,1)
  1c5253:	jmp    1c0630 <PRIVATE0000000000134a87+0x5c0>
  1c5258:	movq   $0x0,0x1870(%rsp,%r13,1)
  1c5264:	jmp    1c05c6 <PRIVATE0000000000134a87+0x556>
  1c5269:	movq   $0x0,0x1878(%rsp,%r13,1)
  1c5275:	jmp    1c05f2 <PRIVATE0000000000134a87+0x582>
  1c527a:	movq   $0x0,0x1858(%rsp)
  1c5286:	jmp    1c0574 <PRIVATE0000000000134a87+0x504>
  1c528b:	mov    0x1f68(%rsp),%r14
  1c5293:	jmp    1c520d <PRIVATE0000000000134a87+0x519d>
  1c5298:	movq   $0x0,0x88(%rsp,%r13,1)
  1c52a4:	jmp    1c0520 <PRIVATE0000000000134a87+0x4b0>
  1c52a9:	movq   $0x0,0x80(%rsp,%r13,1)
  1c52b5:	jmp    1c04f1 <PRIVATE0000000000134a87+0x481>
  1c52ba:	movq   $0x0,0x70(%rsp,%r13,1)
  1c52c3:	jmp    1c04c2 <PRIVATE0000000000134a87+0x452>
  1c52c8:	movq   $0x0,0x68(%rsp,%r13,1)
  1c52d1:	jmp    1c0496 <PRIVATE0000000000134a87+0x426>
  1c52d6:	movq   $0x0,0x58(%rsp,%r13,1)
  1c52df:	jmp    1c046a <PRIVATE0000000000134a87+0x3fa>
  1c52e4:	movq   $0x0,0x50(%rsp,%r13,1)
  1c52ed:	jmp    1c043e <PRIVATE0000000000134a87+0x3ce>
  1c52f2:	movslq 0x2060(%rsp),%rax
  1c52fa:	mov    %rax,0x1f20(%rsp)
  1c5302:	jmp    1c0fc3 <PRIVATE0000000000134a87+0xf53>
