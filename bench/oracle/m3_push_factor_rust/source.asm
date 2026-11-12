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
00000000001c3465 <PRIVATE0000000000134a87+0x33f5>:
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
00000000001c362d <PRIVATE0000000000134a87+0x35bd>:
  1c362d:	cmp    $0x1,%ebx
  1c3630:	je     1c363e <PRIVATE0000000000134a87+0x35ce>
  1c3632:	mov    %r13,0x210(%r14)
  1c3639:	jmp    1c37f5 <PRIVATE0000000000134a87+0x3785>
  1c363e:	mov    %r13,0x1f38(%rsp)
  1c3646:	mov    0x470(%r14),%r12
  1c364d:	mov    0x2090(%rsp),%rax
  1c3655:	mov    %rax,0x210(%r14)
  1c365c:	.byte 0x8b
  1c365d:	.byte 0x84
00000000001c37dd <PRIVATE0000000000134a87+0x376d>:
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
00000000001c3aed <PRIVATE0000000000134a87+0x3a7d>:
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
00000000001c44d5 <PRIVATE0000000000134a87+0x4465>:
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
00000000001c454b <PRIVATE0000000000134a87+0x44db>:
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
