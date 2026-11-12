00000000001205d0 <PRIVATE000000000009cf9b>:
  1205d0:	push   %r12
  1205d2:	push   %rbx
  1205d3:	push   %rbp
  1205d4:	sub    $0x10,%rsp
  1205d8:	test   %ecx,%ecx
  1205da:	je     12064a <PRIVATE000000000009cf9b+0x7a>
  1205dc:	mov    0x1c8(%rdi),%r9
  1205e3:	mov    (%r9),%ebp
  1205e6:	test   %ebp,%ebp
  1205e8:	jle    120643 <PRIVATE000000000009cf9b+0x73>
  1205ea:	mov    %ebp,%r12d
  1205ed:	mov    $0x1,%r11d
  1205f3:	mov    0x10(%r9),%rbx
  1205f7:	xor    %eax,%eax
  1205f9:	mov    0x8(%r9),%r10
  1205fd:	shr    %r12d
  120600:	je     12062b <PRIVATE000000000009cf9b+0x5b>
  120602:	xor    %r11d,%r11d
  120605:	mov    %r13,(%rsp)
  120609:	movslq (%r10,%rax,8),%r13
  12060d:	mov    %r11,(%rbx,%r13,8)
  120611:	movslq 0x4(%r10,%rax,8),%r13
  120616:	inc    %rax
  120619:	mov    %r11,(%rbx,%r13,8)
  12061d:	cmp    %r12,%rax
  120620:	jb     120609 <PRIVATE000000000009cf9b+0x39>
  120622:	mov    (%rsp),%r13
  120626:	lea    0x1(%rax,%rax,1),%r11d
  12062b:	lea    -0x1(%r11),%eax
  12062f:	cmp    %ebp,%eax
  120631:	jae    120643 <PRIVATE000000000009cf9b+0x73>
  120633:	movslq %r11d,%r11
  120636:	movslq -0x4(%r10,%r11,4),%rax
  12063b:	movq   $0x0,(%rbx,%rax,8)
  120643:	movl   $0x0,(%r9)
  12064a:	add    $0xffffffffffffffd0,%rsp
  12064e:	xor    %eax,%eax
  120650:	xor    %r9d,%r9d
  120653:	mov    %rax,(%rsp)
  120657:	mov    %rax,0x8(%rsp)
  12065c:	mov    %rax,0x10(%rsp)
  120661:	mov    %rax,0x18(%rsp)
  120666:	mov    %r8,0x20(%rsp)
  12066b:	xor    %r8d,%r8d
  12066e:	call   120740 <PRIVATE00000000002207de>
  120673:	add    $0x40,%rsp
  120677:	pop    %rbp
  120678:	pop    %rbx
  120679:	pop    %r12
  12067b:	ret    
  12067c:	nopl   0x0(%rax)

0000000000120680 <PRIVATE000000000009d008>:
  120680:	push   %r15
  120682:	push   %rbx
  120683:	push   %rbp
  120684:	sub    $0x10,%rsp
  120688:	test   %rcx,%rcx
  12068b:	je     120739 <PRIVATE000000000009d008+0xb9>
  120691:	mov    (%rcx),%ebp
  120693:	test   %ebp,%ebp
  120695:	jle    1206f2 <PRIVATE000000000009d008+0x72>
  120697:	mov    %ebp,%r15d
  12069a:	mov    $0x1,%r11d
  1206a0:	mov    0x10(%rcx),%rbx
  1206a4:	xor    %r10d,%r10d
  1206a7:	mov    0x8(%rcx),%rax
  1206ab:	shr    %r15d
  1206ae:	je     1206d9 <PRIVATE000000000009d008+0x59>
  1206b0:	xor    %r11d,%r11d
  1206b3:	mov    %r14,(%rsp)
  1206b7:	movslq (%rax,%r10,8),%r14
  1206bb:	mov    %r11,(%rbx,%r14,8)
  1206bf:	movslq 0x4(%rax,%r10,8),%r14
  1206c4:	inc    %r10
  1206c7:	mov    %r11,(%rbx,%r14,8)
  1206cb:	cmp    %r15,%r10
  1206ce:	jb     1206b7 <PRIVATE000000000009d008+0x37>
  1206d0:	mov    (%rsp),%r14
  1206d4:	lea    0x1(%r10,%r10,1),%r11d
  1206d9:	lea    -0x1(%r11),%r10d
  1206dd:	cmp    %ebp,%r10d
  1206e0:	jae    1206f2 <PRIVATE000000000009d008+0x72>
  1206e2:	movslq %r11d,%r11
  1206e5:	movslq -0x4(%rax,%r11,4),%rax
  1206ea:	movq   $0x0,(%rbx,%rax,8)
  1206f2:	movl   $0x0,(%rcx)
  1206f8:	mov    $0x1,%ebp
  1206fd:	add    $0xffffffffffffffd0,%rsp
  120701:	mov    0x60(%rsp),%rax
  120706:	mov    0x68(%rsp),%rbx
  12070b:	mov    %rcx,(%rsp)
  12070f:	mov    %ebp,%ecx
  120711:	mov    %r8,0x8(%rsp)
  120716:	xor    %r8d,%r8d
  120719:	mov    %r9,0x10(%rsp)
  12071e:	xor    %r9d,%r9d
  120721:	mov    %rax,0x18(%rsp)
  120726:	mov    %rbx,0x20(%rsp)
  12072b:	call   120740 <PRIVATE00000000002207de>
  120730:	add    $0x40,%rsp
  120734:	pop    %rbp
  120735:	pop    %rbx
  120736:	pop    %r15
  120738:	ret    
  120739:	xor    %ebp,%ebp
  12073b:	xor    %ecx,%ecx
  12073d:	jmp    1206fd <PRIVATE000000000009d008+0x7d>
