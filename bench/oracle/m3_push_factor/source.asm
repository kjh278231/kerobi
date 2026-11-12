00000000001c336d <PRIVATE0000000000134a87+0x32fd>:
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
... 229 more lines (narrow --range/--grep or raise --max)
00000000001c44ad <PRIVATE0000000000134a87+0x443d>:
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
  1c45bb:	.byte 0x89
  1c45bc:	test   %ah,(%rax,%rsi,2)
  1c45bf:	(bad)  
	...
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
  1c00aa:	rex.WB
  1c00ab:	.byte 0x8b
  1c00ac:	scas   %es:(%rdi),%al
  1c00ad:	.byte 0x68
  1c00ae:	add    %eax,(%rax)
