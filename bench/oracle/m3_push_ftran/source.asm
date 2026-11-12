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
0000000000120740 <PRIVATE00000000002207de>:
  120740:	push   %r12
  120742:	push   %r13
  120744:	push   %r14
  120746:	push   %r15
  120748:	push   %rbx
  120749:	push   %rbp
  12074a:	sub    $0xa8,%rsp
  120751:	mov    %rdi,%r13
  120754:	mov    %rsi,%r12
  120757:	mov    %rdx,%r15
  12075a:	mov    %r9,0x80(%rsp)
  120762:	mov    %ecx,0x88(%rsp)
  120769:	pxor   %xmm1,%xmm1
  12076d:	mov    0x1a0(%r13),%r10
  120774:	lea    0x20(%r13),%rbx
  120778:	mov    0xe0(%rbx),%edi
  12077e:	mov    0xcc(%rbx),%edx
  120784:	neg    %edi
  120786:	mov    %r10,0x98(%rsp)
  12078e:	mov    0x198(%r13),%r10
  120795:	movsd  0xe8(%rbx),%xmm5
  12079d:	mov    -0x18(%rbx),%ebp
  1207a0:	add    %ebp,%edi
  1207a2:	mov    %edx,0x60(%rsp)
  1207a6:	mov    (%r12),%edx
  1207aa:	mov    %r10,0x90(%rsp)
  1207b2:	mov    %r8d,0x78(%rsp)
  1207b7:	mov    0xe0(%rsp),%r14
  1207bf:	mov    0xe8(%rsp),%r11
  1207c7:	mov    0xf0(%rsp),%r9
  1207cf:	mov    0xf8(%rsp),%rsi
  1207d7:	mov    0x160(%rbx),%eax
  1207dd:	movsd  -0x10(%rbx),%xmm0
  1207e2:	mov    (%rbx),%rcx
  1207e5:	mov    0xe4(%rbx),%r10d
  1207ec:	mov    %edi,0x68(%rsp)
  1207f0:	movsd  %xmm1,0x70(%rsp)
  1207f6:	addsd  0xf0(%rbx),%xmm5
  1207fe:	test   %edx,%edx
  120800:	je     121144 <PRIVATE00000000002207de+0xa04>
  120806:	test   %r11,%r11
  120809:	jne    120812 <PRIVATE00000000002207de+0xd2>
  12080b:	mov    0x1b8(%r13),%r11
  120812:	test   %r9,%r9
  120815:	jne    12081e <PRIVATE00000000002207de+0xde>
  120817:	mov    0x1b0(%r13),%r9
  12081e:	test   %rsi,%rsi
  120821:	jne    12082a <PRIVATE00000000002207de+0xea>
  120823:	mov    0x1a8(%r13),%rsi
  12082a:	xor    %edi,%edi
  12082c:	test   %edx,%edx
  12082e:	jl     121e0f <PRIVATE00000000002207de+0x16cf>
  120834:	jle    12085f <PRIVATE00000000002207de+0x11f>
  120836:	mov    0x8(%r12),%r8
  12083b:	mov    0x30(%r13),%rcx
  12083f:	movslq (%r8),%rdx
  120842:	inc    %rdi
  120845:	mov    (%rcx,%rdx,4),%edx
  120848:	mov    %edx,(%r8)
  12084b:	add    $0x4,%r8
  12084f:	mov    (%r12),%edx
  120853:	movslq %edx,%rdx
  120856:	cmp    %rdx,%rdi
  120859:	jl     12083f <PRIVATE00000000002207de+0xff>
  12085b:	test   %edx,%edx
  12085d:	jl     12088a <PRIVATE00000000002207de+0x14a>
  12085f:	imul   %r10d,%edx
  120863:	cmp    %ebp,%edx
  120865:	jg     12088a <PRIVATE00000000002207de+0x14a>
  120867:	pxor   %xmm2,%xmm2
  12086b:	pxor   %xmm1,%xmm1
  12086f:	cvtsi2sd %ebp,%xmm2
  120873:	cvtsi2sdl 0x128(%r13),%xmm1
  12087c:	mulsd  0x293ddac(%rip),%xmm2        # 2a5e630 <_fini+0x1a2c>
  120884:	comisd %xmm2,%xmm1
  120888:	jb     1208db <PRIVATE00000000002207de+0x19b>
  12088a:	mov    %ebp,%edi
  12088c:	mov    %rbx,%rsi
  12088f:	mov    %r12,%rdx
  120892:	mov    %r15,%rcx
  120895:	mov    %r10d,(%rsp)
  120899:	lea    0x70(%rsp),%r8
  12089e:	movsd  %xmm5,-0x38(%r8)
  1208a4:	movsd  %xmm0,-0x30(%r8)
  1208aa:	mov    %eax,-0x68(%r8)
  1208ae:	mov    %r9,-0x18(%r8)
  1208b2:	mov    %r11,-0x40(%r8)
  1208b6:	call   12bd30 <PRIVATE0000000000214344>
  1208bb:	mov    0x30(%rsp),%r11
  1208c0:	mov    0x58(%rsp),%r9
  1208c5:	mov    0x8(%rsp),%eax
  1208c9:	movsd  0x40(%rsp),%xmm0
  1208cf:	movsd  0x38(%rsp),%xmm5
  1208d5:	mov    (%rsp),%r10d
  1208d9:	jmp    12094a <PRIVATE00000000002207de+0x20a>
  1208db:	add    $0xffffffffffffffe0,%rsp
  1208df:	mov    %r13,%rdi
  1208e2:	mov    %r12,%rcx
  1208e5:	lea    0x90(%rsp),%rdx
  1208ed:	mov    %r15,%r8
  1208f0:	mov    %r11,(%rsp)
  1208f4:	mov    %rsi,0x8(%rsp)
  1208f9:	mov    %ebp,%esi
  1208fb:	mov    %rdx,0x10(%rsp)
  120900:	mov    %rbx,%rdx
  120903:	mov    %r10d,0x20(%rsp)
  120908:	movsd  %xmm5,0x58(%rsp)
  12090e:	movsd  %xmm0,0x60(%rsp)
  120914:	mov    %eax,0x28(%rsp)
  120918:	mov    %r9,0x78(%rsp)
  12091d:	mov    %r11,0x50(%rsp)
  120922:	call   12a990 <PRIVATE000000000021b9de>
  120927:	mov    0x50(%rsp),%r11
  12092c:	mov    0x78(%rsp),%r9
  120931:	mov    0x28(%rsp),%eax
  120935:	movsd  0x60(%rsp),%xmm0
  12093b:	movsd  0x58(%rsp),%xmm5
  120941:	mov    0x20(%rsp),%r10d
  120946:	add    $0x20,%rsp
  12094a:	mov    (%r15),%edx
  12094d:	test   %edx,%edx
  12094f:	je     1219b2 <PRIVATE00000000002207de+0x1272>
  120955:	test   %eax,%eax
  120957:	je     121150 <PRIVATE00000000002207de+0xa10>
  12095d:	test   %edx,%edx
  12095f:	jl     121a3f <PRIVATE00000000002207de+0x12ff>
  120965:	imul   %edx,%r10d
  120969:	cmp    %ebp,%r10d
  12096c:	jle    120d53 <PRIVATE00000000002207de+0x613>
  120972:	mov    0x10(%r12),%r10
  120977:	mov    %r10,0x28(%rsp)
  12097c:	test   %edx,%edx
  12097e:	jle    1219f7 <PRIVATE00000000002207de+0x12b7>
  120984:	mov    0x8(%r15),%r10
  120988:	mov    %r10,0x20(%rsp)
  12098d:	mov    0x10(%r15),%r10
  120991:	mov    %r10,0x18(%rsp)
  120996:	test   %ebp,%ebp
  120998:	jle    120a59 <PRIVATE00000000002207de+0x319>
  12099e:	movslq %ebp,%rdi
  1209a1:	cmp    $0x8,%rdi
  1209a5:	jl     1219f0 <PRIVATE00000000002207de+0x12b0>
  1209ab:	mov    0x28(%rsp),%rsi
  1209b0:	and    $0xf,%rsi
  1209b4:	test   %esi,%esi
  1209b6:	je     1209c9 <PRIVATE00000000002207de+0x289>
  1209b8:	test   $0x7,%esi
  1209be:	jne    1219f0 <PRIVATE00000000002207de+0x12b0>
  1209c4:	mov    $0x1,%esi
  1209c9:	mov    %esi,%esi
  1209cb:	mov    %rsi,(%rsp)
  1209cf:	lea    0x8(%rsi),%r10
  1209d3:	cmp    %r10,%rdi
  1209d6:	jl     1219f0 <PRIVATE00000000002207de+0x12b0>
  1209dc:	mov    %ebp,%ecx
  1209de:	xor    %r10d,%r10d
  1209e1:	sub    %esi,%ecx
  1209e3:	and    $0x7,%ecx
  1209e6:	neg    %ecx
  1209e8:	add    %ebp,%ecx
  1209ea:	movslq %ecx,%rcx
  1209ed:	test   %esi,%esi
  1209ef:	jbe    120a10 <PRIVATE00000000002207de+0x2d0>
  1209f1:	mov    0x28(%rsp),%rsi
  1209f6:	mov    (%rsp),%r8
  1209fa:	pxor   %xmm1,%xmm1
  1209fe:	movsd  %xmm1,(%rsi,%r10,8)
  120a04:	inc    %r10
  120a07:	cmp    %r8,%r10
  120a0a:	jb     1209fe <PRIVATE00000000002207de+0x2be>
  120a0c:	mov    %r8,(%rsp)
  120a10:	mov    0x28(%rsp),%r10
  120a15:	mov    (%rsp),%rsi
  120a19:	pxor   %xmm1,%xmm1
  120a1d:	movups %xmm1,(%r10,%rsi,8)
  120a22:	movups %xmm1,0x10(%r10,%rsi,8)
  120a28:	movups %xmm1,0x20(%r10,%rsi,8)
  120a2e:	movups %xmm1,0x30(%r10,%rsi,8)
  120a34:	add    $0x8,%rsi
  120a38:	cmp    %rcx,%rsi
  120a3b:	jb     120a1d <PRIVATE00000000002207de+0x2dd>
  120a3d:	cmp    %rdi,%rcx
  120a40:	jae    120a59 <PRIVATE00000000002207de+0x319>
  120a42:	mov    0x28(%rsp),%r10
  120a47:	pxor   %xmm1,%xmm1
  120a4b:	movsd  %xmm1,(%r10,%rcx,8)
  120a51:	inc    %rcx
  120a54:	cmp    %rdi,%rcx
  120a57:	jb     120a4b <PRIVATE00000000002207de+0x30b>
  120a59:	mov    %edx,%r8d
  120a5c:	xor    %r10d,%r10d
  120a5f:	mov    %eax,%edi
  120a61:	mov    $0x1,%esi
  120a66:	xor    %ecx,%ecx
  120a68:	shr    %r8d
  120a6b:	je     120afe <PRIVATE00000000002207de+0x3be>
  120a71:	mov    %r14,0x8(%rsp)
  120a76:	mov    %r13,0x48(%rsp)
  120a7b:	mov    %r15,0x50(%rsp)
  120a80:	mov    %r8d,%esi
  120a83:	mov    %r11,0x30(%rsp)
  120a88:	mov    %r12,0x10(%rsp)
  120a8d:	mov    0x28(%rsp),%r8
  120a92:	mov    0x18(%rsp),%r13
  120a97:	mov    0x20(%rsp),%r14
  120a9c:	mov    0x98(%rsp),%r15
  120aa4:	movslq (%r14,%r10,8),%r12
  120aa8:	mov    (%r15,%r12,4),%r11d
  120aac:	cmp    %edi,%r11d
  120aaf:	cmovl  %r11d,%edi
  120ab3:	mov    (%rcx,%r13,1),%r11
  120ab7:	mov    %r11,(%r8,%r12,8)
  120abb:	movslq 0x4(%r14,%r10,8),%r11
  120ac0:	inc    %r10
  120ac3:	mov    (%r15,%r11,4),%r12d
  120ac7:	cmp    %edi,%r12d
  120aca:	cmovl  %r12d,%edi
  120ace:	mov    0x8(%rcx,%r13,1),%r12
  120ad3:	add    $0x10,%rcx
  120ad7:	mov    %r12,(%r8,%r11,8)
  120adb:	cmp    %rsi,%r10
  120ade:	jb     120aa4 <PRIVATE00000000002207de+0x364>
  120ae0:	mov    0x30(%rsp),%r11
  120ae5:	lea    0x1(%r10,%r10,1),%esi
  120aea:	mov    0x8(%rsp),%r14
  120aef:	mov    0x48(%rsp),%r13
  120af4:	mov    0x10(%rsp),%r12
  120af9:	mov    0x50(%rsp),%r15
  120afe:	lea    -0x1(%rsi),%r10d
  120b02:	cmp    %edx,%r10d
  120b05:	jae    120b38 <PRIVATE00000000002207de+0x3f8>
  120b07:	movslq %esi,%rsi
  120b0a:	mov    0x20(%rsp),%r10
  120b0f:	mov    0x98(%rsp),%rdx
  120b17:	mov    0x18(%rsp),%rcx
  120b1c:	movslq -0x4(%r10,%rsi,4),%r10
  120b21:	mov    -0x8(%rcx,%rsi,8),%r8
  120b26:	cmp    (%rdx,%r10,4),%edi
  120b2a:	cmovg  (%rdx,%r10,4),%edi
  120b2f:	mov    0x28(%rsp),%rdx
  120b34:	mov    %r8,(%rdx,%r10,8)
  120b38:	movslq %edi,%r10
  120b3b:	xor    %edx,%edx
  120b3d:	cmp    %eax,%edi
  120b3f:	jge    120d20 <PRIVATE00000000002207de+0x5e0>
  120b45:	movslq %eax,%rax
  120b48:	lea    (%r10,%r10,2),%rcx
  120b4c:	sub    %r10,%rax
  120b4f:	mov    0x90(%rsp),%r10
  120b57:	pxor   %xmm2,%xmm2
  120b5b:	mov    %r15,0x50(%rsp)
  120b60:	mov    %r14,0x8(%rsp)
  120b65:	mov    %r13,0x48(%rsp)
  120b6a:	lea    (%r10,%rcx,8),%r10
  120b6e:	mov    %r12,0x10(%rsp)
  120b73:	mov    0x28(%rsp),%r15
  120b78:	movslq 0x4(%r10),%rdi
  120b7c:	movaps %xmm2,%xmm1
  120b7f:	mov    0x8(%r10),%r8
  120b83:	mov    0x10(%r10),%rsi
  120b87:	test   %rdi,%rdi
  120b8a:	jle    120cdb <PRIVATE00000000002207de+0x59b>
  120b90:	cmp    $0x8,%rdi
  120b94:	jl     1219e8 <PRIVATE00000000002207de+0x12a8>
  120b9a:	mov    %rsi,%r14
  120b9d:	and    $0xf,%r14
  120ba1:	test   %r14d,%r14d
  120ba4:	je     120bb9 <PRIVATE00000000002207de+0x479>
  120ba6:	test   $0x7,%r14d
  120bad:	jne    1219e8 <PRIVATE00000000002207de+0x12a8>
  120bb3:	mov    $0x1,%r14d
  120bb9:	mov    %r14d,%r13d
  120bbc:	lea    0x8(%r13),%rcx
  120bc0:	cmp    %rcx,%rdi
  120bc3:	jl     1219e8 <PRIVATE00000000002207de+0x12a8>
  120bc9:	mov    %edi,%r12d
  120bcc:	mov    %r12d,%ecx
  120bcf:	sub    %r14d,%ecx
  120bd2:	and    $0x7,%ecx
  120bd5:	sub    %ecx,%r12d
  120bd8:	xor    %ecx,%ecx
  120bda:	movslq %r12d,%r12
  120bdd:	test   %r14d,%r14d
  120be0:	jbe    120bfd <PRIVATE00000000002207de+0x4bd>
  120be2:	movslq (%r8,%rcx,4),%r14
  120be6:	movsd  (%r15,%r14,8),%xmm3
  120bec:	mulsd  (%rsi,%rcx,8),%xmm3
  120bf1:	inc    %rcx
  120bf4:	addsd  %xmm3,%xmm1
  120bf8:	cmp    %r13,%rcx
  120bfb:	jb     120be2 <PRIVATE00000000002207de+0x4a2>
  120bfd:	pxor   %xmm4,%xmm4
  120c01:	movaps %xmm4,%xmm6
  120c04:	movaps %xmm4,%xmm3
  120c07:	movsd  %xmm1,%xmm6
  120c0b:	movaps %xmm3,%xmm1
  120c0e:	movslq (%r8,%r13,4),%rcx
  120c12:	movslq 0x4(%r8,%r13,4),%r14
  120c17:	movsd  (%r15,%rcx,8),%xmm7
  120c1d:	movslq 0x8(%r8,%r13,4),%rcx
  120c22:	movhpd (%r15,%r14,8),%xmm7
  120c28:	movslq 0xc(%r8,%r13,4),%r14
  120c2d:	movsd  (%r15,%rcx,8),%xmm8
  120c33:	movslq 0x10(%r8,%r13,4),%rcx
  120c38:	movhpd (%r15,%r14,8),%xmm8
  120c3e:	movslq 0x14(%r8,%r13,4),%r14
  120c43:	movsd  (%r15,%rcx,8),%xmm9
  120c49:	movslq 0x18(%r8,%r13,4),%rcx
  120c4e:	movhpd (%r15,%r14,8),%xmm9
  120c54:	movslq 0x1c(%r8,%r13,4),%r14
  120c59:	movsd  (%r15,%rcx,8),%xmm10
  120c5f:	mulpd  (%rsi,%r13,8),%xmm7
  120c65:	mulpd  0x10(%rsi,%r13,8),%xmm8
  120c6c:	mulpd  0x20(%rsi,%r13,8),%xmm9
  120c73:	addpd  %xmm7,%xmm6
  120c77:	addpd  %xmm8,%xmm4
  120c7c:	addpd  %xmm9,%xmm3
  120c81:	movhpd (%r15,%r14,8),%xmm10
  120c87:	mulpd  0x30(%rsi,%r13,8),%xmm10
  120c8e:	add    $0x8,%r13
  120c92:	addpd  %xmm10,%xmm1
  120c97:	cmp    %r12,%r13
  120c9a:	jb     120c0e <PRIVATE00000000002207de+0x4ce>
  120ca0:	addpd  %xmm4,%xmm6
  120ca4:	addpd  %xmm1,%xmm3
  120ca8:	addpd  %xmm3,%xmm6
  120cac:	movaps %xmm6,%xmm1
  120caf:	unpckhpd %xmm6,%xmm1
  120cb3:	addsd  %xmm1,%xmm6
  120cb7:	movaps %xmm6,%xmm1
  120cba:	cmp    %rdi,%r12
  120cbd:	jae    120cdb <PRIVATE00000000002207de+0x59b>
  120cbf:	movslq (%r8,%r12,4),%rcx
  120cc3:	movsd  (%r15,%rcx,8),%xmm3
  120cc9:	mulsd  (%rsi,%r12,8),%xmm3
  120ccf:	inc    %r12
  120cd2:	addsd  %xmm3,%xmm1
  120cd6:	cmp    %rdi,%r12
  120cd9:	jb     120cbf <PRIVATE00000000002207de+0x57f>
  120cdb:	movaps %xmm1,%xmm3
  120cde:	andps  0x293d90b(%rip),%xmm3        # 2a5e5f0 <_fini+0x19ec>
  120ce5:	movslq (%r10),%rcx
  120ce8:	comisd %xmm0,%xmm3
  120cec:	jbe    120cf6 <PRIVATE00000000002207de+0x5b6>
  120cee:	movsd  %xmm1,(%r15,%rcx,8)
  120cf4:	jmp    120cfc <PRIVATE00000000002207de+0x5bc>
  120cf6:	movsd  %xmm2,(%r15,%rcx,8)
  120cfc:	inc    %rdx
  120cff:	add    $0x18,%r10
  120d03:	cmp    %rax,%rdx
  120d06:	jb     120b78 <PRIVATE00000000002207de+0x438>
  120d0c:	mov    0x8(%rsp),%r14
  120d11:	mov    0x48(%rsp),%r13
  120d16:	mov    0x10(%rsp),%r12
  120d1b:	mov    0x50(%rsp),%r15
  120d20:	pxor   %xmm1,%xmm1
  120d24:	cvtsi2sdl 0x184(%r13),%xmm1
  120d2d:	mulsd  0x293d903(%rip),%xmm1        # 2a5e638 <_fini+0x1a34>
  120d35:	movl   $0xffffffff,(%r12)
  120d3d:	addsd  0x70(%rsp),%xmm1
  120d43:	movsd  %xmm1,0x70(%rsp)
  120d49:	mov    $0xffffffff,%edx
  120d4e:	jmp    121184 <PRIVATE00000000002207de+0xa44>
  120d53:	mov    0x8(%r15),%r10
  120d57:	mov    %eax,%edi
  120d59:	movslq %edx,%rdx
  120d5c:	mov    %r10,0x20(%rsp)
  120d61:	mov    0x10(%r15),%r10
  120d65:	mov    %rdx,0x18(%rsp)
  120d6a:	mov    %r10,0x28(%rsp)
  120d6f:	test   %edx,%edx
  120d71:	jle    120e5e <PRIVATE00000000002207de+0x71e>
  120d77:	mov    %edx,%r10d
  120d7a:	xor    %ecx,%ecx
  120d7c:	mov    $0x1,%r8d
  120d82:	xor    %esi,%esi
  120d84:	shr    %r10d
  120d87:	je     120e21 <PRIVATE00000000002207de+0x6e1>
  120d8d:	mov    %ebp,(%rsp)
  120d90:	mov    %r14,0x8(%rsp)
  120d95:	mov    %r12,0x10(%rsp)
  120d9a:	mov    %r13,0x48(%rsp)
  120d9f:	mov    $0x1,%r13d
  120da5:	mov    %r15,0x50(%rsp)
  120daa:	mov    0x28(%rsp),%r14
  120daf:	mov    0x20(%rsp),%rbp
  120db4:	mov    0x98(%rsp),%r12
  120dbc:	nopl   0x0(%rax)
  120dc0:	movslq 0x0(%rbp,%rcx,8),%r8
  120dc5:	mov    (%rsi,%r14,1),%r15
  120dc9:	mov    %r13d,(%r9,%r8,4)
  120dcd:	mov    %r15,(%r11,%r8,8)
  120dd1:	mov    (%r12,%r8,4),%r15d
  120dd5:	cmp    %edi,%r15d
  120dd8:	movslq 0x4(%rbp,%rcx,8),%r8
  120ddd:	cmovl  %r15d,%edi
  120de1:	inc    %rcx
  120de4:	mov    0x8(%rsi,%r14,1),%r15
  120de9:	add    $0x10,%rsi
  120ded:	mov    %r13d,(%r9,%r8,4)
  120df1:	mov    %r15,(%r11,%r8,8)
  120df5:	mov    (%r12,%r8,4),%r15d
  120df9:	cmp    %edi,%r15d
  120dfc:	cmovl  %r15d,%edi
  120e00:	cmp    %r10,%rcx
  120e03:	jb     120dc0 <PRIVATE00000000002207de+0x680>
  120e05:	mov    (%rsp),%ebp
  120e08:	lea    0x1(%rcx,%rcx,1),%r8d
  120e0d:	mov    0x8(%rsp),%r14
  120e12:	mov    0x48(%rsp),%r13
  120e17:	mov    0x10(%rsp),%r12
  120e1c:	mov    0x50(%rsp),%r15
  120e21:	lea    -0x1(%r8),%r10d
  120e25:	cmp    %edx,%r10d
  120e28:	jae    120e5e <PRIVATE00000000002207de+0x71e>
  120e2a:	movslq %r8d,%r8
  120e2d:	mov    0x20(%rsp),%r10
  120e32:	movslq -0x4(%r10,%r8,4),%rdx
  120e37:	mov    0x28(%rsp),%r10
  120e3c:	movl   $0x1,(%r9,%rdx,4)
  120e44:	mov    -0x8(%r10,%r8,8),%r10
  120e49:	mov    %r10,(%r11,%rdx,8)
  120e4d:	mov    0x98(%rsp),%r10
  120e55:	cmp    (%r10,%rdx,4),%edi
  120e59:	cmovg  (%r10,%rdx,4),%edi
  120e5e:	mov    %eax,%r10d
  120e61:	pxor   %xmm4,%xmm4
  120e65:	sub    %edi,%r10d
  120e68:	xor    %edx,%edx
  120e6a:	cvtsi2sd %r10d,%xmm4
  120e6f:	movsd  0x293d7a9(%rip),%xmm2        # 2a5e620 <_fini+0x1a1c>
  120e77:	mulsd  %xmm2,%xmm4
  120e7b:	movslq %edi,%r10
  120e7e:	cmp    %eax,%edi
  120e80:	jge    1210ac <PRIVATE00000000002207de+0x96c>
  120e86:	movslq %eax,%rax
  120e89:	lea    (%r10,%r10,2),%rcx
  120e8d:	sub    %r10,%rax
  120e90:	mov    0x90(%rsp),%r10
  120e98:	pxor   %xmm3,%xmm3
  120e9c:	mov    %ebp,(%rsp)
  120e9f:	movsd  0x293d791(%rip),%xmm1        # 2a5e638 <_fini+0x1a34>
  120ea7:	mov    %r14,0x8(%rsp)
  120eac:	lea    (%r10,%rcx,8),%r10
  120eb0:	mov    %r13,0x48(%rsp)
  120eb5:	mov    %r12,0x10(%rsp)
  120eba:	mov    %r15,0x50(%rsp)
  120ebf:	mov    0x20(%rsp),%rdi
  120ec4:	mov    0x18(%rsp),%rbp
  120ec9:	pxor   %xmm6,%xmm6
  120ecd:	mov    0x4(%r10),%esi
  120ed1:	cvtsi2sd %esi,%xmm6
  120ed5:	mulsd  %xmm1,%xmm6
  120ed9:	mov    0x8(%r10),%r13
  120edd:	addsd  %xmm6,%xmm4
  120ee1:	mov    0x10(%r10),%r14
  120ee5:	movaps %xmm3,%xmm6
  120ee8:	test   %esi,%esi
  120eea:	jle    121044 <PRIVATE00000000002207de+0x904>
  120ef0:	movslq %esi,%r12
  120ef3:	cmp    $0x8,%r12
  120ef7:	jl     121a37 <PRIVATE00000000002207de+0x12f7>
  120efd:	mov    %r14,%rcx
  120f00:	and    $0xf,%rcx
  120f04:	test   %ecx,%ecx
  120f06:	je     120f16 <PRIVATE00000000002207de+0x7d6>
  120f08:	test   $0x7,%cl
  120f0b:	jne    121a37 <PRIVATE00000000002207de+0x12f7>
  120f11:	mov    $0x1,%ecx
  120f16:	mov    %ecx,%r15d
  120f19:	lea    0x8(%r15),%r8
  120f1d:	cmp    %r8,%r12
  120f20:	jl     121a37 <PRIVATE00000000002207de+0x12f7>
  120f26:	mov    %esi,%r8d
  120f29:	sub    %ecx,%r8d
  120f2c:	and    $0x7,%r8d
  120f30:	sub    %r8d,%esi
  120f33:	movslq %esi,%r8
  120f36:	xor    %esi,%esi
  120f38:	test   %ecx,%ecx
  120f3a:	jbe    120f59 <PRIVATE00000000002207de+0x819>
  120f3c:	movslq 0x0(%r13,%rsi,4),%rcx
  120f41:	movsd  (%r11,%rcx,8),%xmm7
  120f47:	mulsd  (%r14,%rsi,8),%xmm7
  120f4d:	inc    %rsi
  120f50:	addsd  %xmm7,%xmm6
  120f54:	cmp    %r15,%rsi
  120f57:	jb     120f3c <PRIVATE00000000002207de+0x7fc>
  120f59:	pxor   %xmm8,%xmm8
  120f5e:	movaps %xmm8,%xmm9
  120f62:	movaps %xmm8,%xmm7
  120f66:	movsd  %xmm6,%xmm9
  120f6b:	movaps %xmm7,%xmm6
  120f6e:	movslq 0x0(%r13,%r15,4),%rcx
  120f73:	movsd  (%r11,%rcx,8),%xmm10
  120f79:	movslq 0x4(%r13,%r15,4),%rcx
  120f7e:	movhpd (%r11,%rcx,8),%xmm10
  120f84:	movslq 0x8(%r13,%r15,4),%rcx
  120f89:	mulpd  (%r14,%r15,8),%xmm10
  120f8f:	movsd  (%r11,%rcx,8),%xmm11
  120f95:	movslq 0xc(%r13,%r15,4),%rcx
  120f9a:	addpd  %xmm10,%xmm9
  120f9f:	movhpd (%r11,%rcx,8),%xmm11
  120fa5:	movslq 0x10(%r13,%r15,4),%rcx
  120faa:	mulpd  0x10(%r14,%r15,8),%xmm11
  120fb1:	movsd  (%r11,%rcx,8),%xmm12
  120fb7:	movslq 0x14(%r13,%r15,4),%rcx
  120fbc:	addpd  %xmm11,%xmm8
  120fc1:	movhpd (%r11,%rcx,8),%xmm12
  120fc7:	movslq 0x18(%r13,%r15,4),%rcx
  120fcc:	mulpd  0x20(%r14,%r15,8),%xmm12
  120fd3:	movsd  (%r11,%rcx,8),%xmm13
  120fd9:	movslq 0x1c(%r13,%r15,4),%rcx
  120fde:	addpd  %xmm12,%xmm7
  120fe3:	movhpd (%r11,%rcx,8),%xmm13
  120fe9:	mulpd  0x30(%r14,%r15,8),%xmm13
  120ff0:	add    $0x8,%r15
  120ff4:	addpd  %xmm13,%xmm6
  120ff9:	cmp    %r8,%r15
  120ffc:	jb     120f6e <PRIVATE00000000002207de+0x82e>
  121002:	addpd  %xmm8,%xmm9
  121007:	addpd  %xmm6,%xmm7
  12100b:	addpd  %xmm7,%xmm9
  121010:	movaps %xmm9,%xmm6
  121014:	unpckhpd %xmm9,%xmm6
  121019:	addsd  %xmm6,%xmm9
  12101e:	movaps %xmm9,%xmm6
  121022:	cmp    %r12,%r8
  121025:	jae    121044 <PRIVATE00000000002207de+0x904>
  121027:	movslq 0x0(%r13,%r8,4),%rcx
  12102c:	movsd  (%r11,%rcx,8),%xmm7
  121032:	mulsd  (%r14,%r8,8),%xmm7
  121038:	inc    %r8
  12103b:	addsd  %xmm7,%xmm6
  12103f:	cmp    %r12,%r8
  121042:	jb     121027 <PRIVATE00000000002207de+0x8e7>
  121044:	movaps %xmm6,%xmm7
  121047:	andps  0x293d5a2(%rip),%xmm7        # 2a5e5f0 <_fini+0x19ec>
  12104e:	movslq (%r10),%rcx
  121051:	comisd %xmm0,%xmm7
  121055:	jbe    12107a <PRIVATE00000000002207de+0x93a>
  121057:	cmpl   $0x0,(%r9,%rcx,4)
  12105c:	jne    121072 <PRIVATE00000000002207de+0x932>
  12105e:	movl   $0x1,(%r9,%rcx,4)
  121066:	mov    (%r10),%ecx
  121069:	mov    %ecx,(%rdi,%rbp,4)
  12106c:	inc    %rbp
  12106f:	movslq (%r10),%rcx
  121072:	movsd  %xmm6,(%r11,%rcx,8)
  121078:	jmp    121080 <PRIVATE00000000002207de+0x940>
  12107a:	movsd  %xmm3,(%r11,%rcx,8)
  121080:	inc    %rdx
  121083:	add    $0x18,%r10
  121087:	cmp    %rax,%rdx
  12108a:	jb     120ec9 <PRIVATE00000000002207de+0x789>
  121090:	mov    %rbp,0x18(%rsp)
  121095:	mov    (%rsp),%ebp
  121098:	mov    0x8(%rsp),%r14
  12109d:	mov    0x48(%rsp),%r13
  1210a2:	mov    0x10(%rsp),%r12
  1210a7:	mov    0x50(%rsp),%r15
  1210ac:	mov    0x10(%r12),%r10
  1210b1:	xor    %edx,%edx
  1210b3:	mov    0x8(%r12),%rax
  1210b8:	mov    %r10,0x28(%rsp)
  1210bd:	xor    %r10d,%r10d
  1210c0:	cmpq   $0x0,0x18(%rsp)
  1210c6:	jle    12111b <PRIVATE00000000002207de+0x9db>
  1210c8:	mov    0x28(%rsp),%rcx
  1210cd:	mov    0x20(%rsp),%rdi
  1210d2:	pxor   %xmm6,%xmm6
  1210d6:	mov    0x18(%rsp),%r8
  1210db:	movslq (%rdi,%r10,4),%rsi
  1210df:	movsd  (%r11,%rsi,8),%xmm3
  1210e5:	movaps %xmm3,%xmm1
  1210e8:	andps  0x293d501(%rip),%xmm1        # 2a5e5f0 <_fini+0x19ec>
  1210ef:	comisd %xmm0,%xmm1
  1210f3:	jb     121100 <PRIVATE00000000002207de+0x9c0>
  1210f5:	mov    %esi,(%rax,%rdx,4)
  1210f8:	movsd  %xmm3,(%rcx,%rdx,8)
  1210fd:	inc    %rdx
  121100:	inc    %r10
  121103:	movsd  %xmm6,(%r11,%rsi,8)
  121109:	movl   $0x0,(%r9,%rsi,4)
  121111:	cmp    %r8,%r10
  121114:	jl     1210db <PRIVATE00000000002207de+0x99b>
  121116:	mov    %r8,0x18(%rsp)
  12111b:	pxor   %xmm1,%xmm1
  12111f:	mov    0x18(%rsp),%rax
  121124:	cvtsi2sd %eax,%xmm1
  121128:	mulsd  %xmm1,%xmm2
  12112c:	mov    %edx,(%r12)
  121130:	addsd  %xmm2,%xmm4
  121134:	addsd  0x70(%rsp),%xmm4
  12113a:	movsd  %xmm4,0x70(%rsp)
  121140:	test   %edx,%edx
  121142:	jne    121184 <PRIVATE00000000002207de+0xa44>
  121144:	movl   $0x0,(%r15)
  12114b:	jmp    1219b2 <PRIVATE00000000002207de+0x1272>
  121150:	mov    (%r12),%eax
  121154:	mov    0x8(%r12),%r10
  121159:	mov    %eax,(%r15)
  12115c:	mov    0x8(%r15),%rax
  121160:	mov    %r10,0x8(%r15)
  121164:	mov    %rax,0x8(%r12)
  121169:	mov    0x10(%r12),%rax
  12116e:	mov    0x10(%r15),%r10
  121172:	mov    %rax,0x10(%r15)
  121176:	mov    %edx,(%r12)
  12117a:	mov    %r10,0x28(%rsp)
  12117f:	mov    %r10,0x10(%r12)
  121184:	cmpl   $0x0,0x88(%rsp)
  12118c:	je     121216 <PRIVATE00000000002207de+0xad6>
  121192:	pxor   %xmm1,%xmm1
  121196:	cvtsi2sdl 0x88(%rsp),%xmm1
  12119f:	test   %r14,%r14
  1211a2:	jne    1211ab <PRIVATE00000000002207de+0xa6b>
  1211a4:	mov    0x1c8(%r13),%r14
  1211ab:	test   %edx,%edx
  1211ad:	jle    121d2d <PRIVATE00000000002207de+0x15ed>
  1211b3:	mov    %edx,(%r14)
  1211b6:	mov    (%r12),%edx
  1211ba:	test   %edx,%edx
  1211bc:	jle    121216 <PRIVATE00000000002207de+0xad6>
  1211be:	mov    0x8(%r12),%rsi
  1211c3:	mov    0x8(%r14),%rdi
  1211c7:	cmp    %rsi,%rdi
  1211ca:	je     121241 <PRIVATE00000000002207de+0xb01>
  1211cc:	movslq %edx,%rdx
  1211cf:	shl    $0x2,%rdx
  1211d3:	movsd  %xmm1,(%rsp)
  1211d8:	movsd  %xmm5,0x38(%rsp)
  1211de:	movsd  %xmm0,0x40(%rsp)
  1211e4:	mov    %r9,0x58(%rsp)
  1211e9:	mov    %r11,0x30(%rsp)
  1211ee:	call   bcab0 <memcpy@plt>
  1211f3:	mov    (%r12),%edx
  1211f7:	mov    0x30(%rsp),%r11
  1211fc:	mov    0x58(%rsp),%r9
  121201:	movsd  0x40(%rsp),%xmm0
  121207:	movsd  0x38(%rsp),%xmm5
  12120d:	movsd  (%rsp),%xmm1
  121212:	test   %edx,%edx
  121214:	jg     121232 <PRIVATE00000000002207de+0xaf2>
  121216:	cmpq   $0x0,0x80(%rsp)
  12121f:	je     121c5d <PRIVATE00000000002207de+0x151d>
  121225:	test   %edx,%edx
  121227:	jge    1212d1 <PRIVATE00000000002207de+0xb91>
  12122d:	jmp    121309 <PRIVATE00000000002207de+0xbc9>
  121232:	mov    0x10(%r12),%rax
  121237:	mov    0x8(%r12),%rsi
  12123c:	mov    %rax,0x28(%rsp)
  121241:	mov    0x10(%r14),%rax
  121245:	xor    %r14d,%r14d
  121248:	mov    %rax,(%rsp)
  12124c:	mov    %edx,%eax
  12124e:	mov    $0x1,%r10d
  121254:	xor    %ecx,%ecx
  121256:	shr    %eax
  121258:	je     12129c <PRIVATE00000000002207de+0xb5c>
  12125a:	mov    (%rsp),%r10
  12125e:	mov    0x28(%rsp),%rdi
  121263:	movsd  (%rcx,%rdi,1),%xmm2
  121268:	mulsd  %xmm1,%xmm2
  12126c:	movslq (%rsi,%r14,8),%r8
  121270:	movsd  %xmm2,(%r10,%r8,8)
  121276:	movsd  0x8(%rcx,%rdi,1),%xmm3
  12127c:	add    $0x10,%rcx
  121280:	mulsd  %xmm1,%xmm3
  121284:	movslq 0x4(%rsi,%r14,8),%r8
  121289:	inc    %r14
  12128c:	movsd  %xmm3,(%r10,%r8,8)
  121292:	cmp    %rax,%r14
  121295:	jb     121263 <PRIVATE00000000002207de+0xb23>
  121297:	lea    0x1(%r14,%r14,1),%r10d
  12129c:	lea    -0x1(%r10),%eax
  1212a0:	cmp    %edx,%eax
  1212a2:	jae    1212c2 <PRIVATE00000000002207de+0xb82>
  1212a4:	movslq %r10d,%r10
  1212a7:	mov    0x28(%rsp),%rax
  1212ac:	mov    (%rsp),%r14
  1212b0:	mulsd  -0x8(%rax,%r10,8),%xmm1
  1212b7:	movslq -0x4(%rsi,%r10,4),%rax
  1212bc:	movsd  %xmm1,(%r14,%rax,8)
  1212c2:	cmpq   $0x0,0x80(%rsp)
  1212cb:	je     121c61 <PRIVATE00000000002207de+0x1521>
  1212d1:	mov    0x68(%rsp),%eax
  1212d5:	add    0x60(%rsp),%eax
  1212d9:	add    %edx,%eax
  1212db:	cmp    %ebp,%eax
  1212dd:	jl     121974 <PRIVATE00000000002207de+0x1234>
  1212e3:	pxor   %xmm2,%xmm2
  1212e7:	pxor   %xmm1,%xmm1
  1212eb:	cvtsi2sd %ebp,%xmm2
  1212ef:	cvtsi2sd %edx,%xmm1
  1212f3:	mulsd  %xmm5,%xmm5
  1212f7:	addsd  %xmm2,%xmm2
  1212fb:	mulsd  %xmm1,%xmm5
  1212ff:	comisd %xmm2,%xmm5
  121303:	jb     121974 <PRIVATE00000000002207de+0x1234>
  121309:	mov    0x98(%r13),%rax
  121310:	mov    %rax,0x38(%rsp)
  121315:	mov    0xf8(%r13),%rax
  12131c:	mov    %rax,0x10(%rsp)
  121321:	mov    0xa0(%r13),%rax
  121328:	mov    0x88(%r13),%r9
  12132f:	mov    %rax,0x20(%rsp)
  121334:	mov    0xb0(%r13),%r11
  12133b:	mov    0xc8(%r13),%rax
  121342:	mov    %r9,0x40(%rsp)
  121347:	mov    0xa8(%r13),%rbx
  12134e:	mov    0xd0(%r13),%r9
  121355:	mov    %r11,0x68(%rsp)
  12135a:	mov    %rax,0x8(%rsp)
  12135f:	mov    0xb8(%r13),%r14
  121366:	mov    0x50(%r13),%rax
  12136a:	mov    0x10(%r15),%r11
  12136e:	mov    %rbx,0x30(%rsp)
  121373:	mov    %r9,0x18(%rsp)
  121378:	movslq 0x100(%r13),%r10
  12137f:	mov    %r14,0x60(%rsp)
  121384:	mov    0x48(%r13),%r9
  121388:	mov    %rax,0x58(%rsp)
  12138d:	mov    %r11,0x28(%rsp)
  121392:	mov    0x10(%r12),%rbx
  121397:	test   %edx,%edx
  121399:	jle    121d22 <PRIVATE00000000002207de+0x15e2>
  12139f:	pxor   %xmm2,%xmm2
  1213a3:	pxor   %xmm1,%xmm1
  1213a7:	cvtsi2sd %ebp,%xmm2
  1213ab:	cvtsi2sd %edx,%xmm1
  1213af:	addsd  0x70(%rsp),%xmm2
  1213b5:	mulsd  0x293d27b(%rip),%xmm1        # 2a5e638 <_fini+0x1a34>
  1213bd:	mov    0x8(%r12),%rax
  1213c2:	addsd  %xmm1,%xmm2
  1213c6:	test   %ebp,%ebp
  1213c8:	jle    12146d <PRIVATE00000000002207de+0xd2d>
  1213ce:	movslq %ebp,%rsi
  1213d1:	cmp    $0x8,%rsi
  1213d5:	jl     121d1b <PRIVATE00000000002207de+0x15db>
  1213db:	mov    %r11,%r14
  1213de:	and    $0xf,%r14
  1213e2:	test   %r14d,%r14d
  1213e5:	je     1213fa <PRIVATE00000000002207de+0xcba>
  1213e7:	test   $0x7,%r14d
  1213ee:	jne    121d1b <PRIVATE00000000002207de+0x15db>
  1213f4:	mov    $0x1,%r14d
  1213fa:	mov    %r14d,%r14d
  1213fd:	lea    0x8(%r14),%rcx
  121401:	cmp    %rcx,%rsi
  121404:	jl     121d1b <PRIVATE00000000002207de+0x15db>
  12140a:	mov    %ebp,%ecx
  12140c:	xor    %edi,%edi
  12140e:	sub    %r14d,%ecx
  121411:	and    $0x7,%ecx
  121414:	neg    %ecx
  121416:	add    %ebp,%ecx
  121418:	movslq %ecx,%rcx
  12141b:	test   %r14d,%r14d
  12141e:	jbe    121432 <PRIVATE00000000002207de+0xcf2>
  121420:	pxor   %xmm1,%xmm1
  121424:	movsd  %xmm1,(%r11,%rdi,8)
  12142a:	inc    %rdi
  12142d:	cmp    %r14,%rdi
  121430:	jb     121424 <PRIVATE00000000002207de+0xce4>
  121432:	pxor   %xmm1,%xmm1
  121436:	movups %xmm1,(%r11,%r14,8)
  12143b:	movups %xmm1,0x10(%r11,%r14,8)
  121441:	movups %xmm1,0x20(%r11,%r14,8)
  121447:	movups %xmm1,0x30(%r11,%r14,8)
  12144d:	add    $0x8,%r14
  121451:	cmp    %rcx,%r14
  121454:	jb     121436 <PRIVATE00000000002207de+0xcf6>
  121456:	cmp    %rsi,%rcx
  121459:	jae    12146d <PRIVATE00000000002207de+0xd2d>
  12145b:	pxor   %xmm1,%xmm1
  12145f:	movsd  %xmm1,(%r11,%rcx,8)
  121465:	inc    %rcx
  121468:	cmp    %rsi,%rcx
  12146b:	jb     12145f <PRIVATE00000000002207de+0xd1f>
  12146d:	test   %edx,%edx
  12146f:	jle    121534 <PRIVATE00000000002207de+0xdf4>
  121475:	movslq %edx,%rsi
  121478:	cmp    $0x2,%rsi
  12147c:	jl     121d14 <PRIVATE00000000002207de+0x15d4>
  121482:	mov    %rbx,%r14
  121485:	and    $0xf,%r14
  121489:	test   %r14d,%r14d
  12148c:	je     1214a1 <PRIVATE00000000002207de+0xd61>
  12148e:	test   $0x7,%r14d
  121495:	jne    121d14 <PRIVATE00000000002207de+0x15d4>
  12149b:	mov    $0x1,%r14d
  1214a1:	mov    %r14d,%r14d
  1214a4:	mov    %r14,(%rsp)
  1214a8:	lea    0x2(%r14),%rcx
  1214ac:	cmp    %rcx,%rsi
  1214af:	jl     121d14 <PRIVATE00000000002207de+0x15d4>
  1214b5:	mov    %edx,%edi
  1214b7:	xor    %ecx,%ecx
  1214b9:	sub    %r14d,%edi
  1214bc:	and    $0x1,%edi
  1214bf:	neg    %edi
  1214c1:	add    %edx,%edi
  1214c3:	movslq %edi,%rdi
  1214c6:	test   %r14d,%r14d
  1214c9:	jbe    1214f0 <PRIVATE00000000002207de+0xdb0>
  1214cb:	mov    %r15,0x50(%rsp)
  1214d0:	mov    %r14,%r15
  1214d3:	movslq (%rax,%rcx,4),%r14
  1214d7:	mov    (%rbx,%rcx,8),%r8
  1214db:	inc    %rcx
  1214de:	mov    %r8,(%r11,%r14,8)
  1214e2:	cmp    %r15,%rcx
  1214e5:	jb     1214d3 <PRIVATE00000000002207de+0xd93>
  1214e7:	mov    %r15,(%rsp)
  1214eb:	mov    0x50(%rsp),%r15
  1214f0:	mov    (%rsp),%rcx
  1214f4:	movslq (%rax,%rcx,4),%r14
  1214f8:	movups (%rbx,%rcx,8),%xmm1
  1214fc:	movsd  %xmm1,(%r11,%r14,8)
  121502:	movslq 0x4(%rax,%rcx,4),%r14
  121507:	add    $0x2,%rcx
  12150b:	psrldq $0x8,%xmm1
  121510:	movsd  %xmm1,(%r11,%r14,8)
  121516:	cmp    %rdi,%rcx
  121519:	jb     1214f4 <PRIVATE00000000002207de+0xdb4>
  12151b:	cmp    %rsi,%rdi
  12151e:	jae    121534 <PRIVATE00000000002207de+0xdf4>
  121520:	movslq (%rax,%rdi,4),%rcx
  121524:	mov    (%rbx,%rdi,8),%r14
  121528:	inc    %rdi
  12152b:	mov    %r14,(%r11,%rcx,8)
  12152f:	cmp    %rsi,%rdi
  121532:	jb     121520 <PRIVATE00000000002207de+0xde0>
  121534:	mov    (%r15),%r14d
  121537:	mov    %edx,(%r15)
  12153a:	mov    %r14d,(%r12)
  12153e:	mov    %rbx,0x10(%r15)
  121542:	mov    0x8(%r15),%r14
  121546:	mov    %r11,0x10(%r12)
  12154b:	mov    %rax,0x8(%r15)
  12154f:	mov    %r14,0x8(%r12)
  121554:	mov    0x10(%r15),%r11
  121558:	mov    0x28(%rsp),%rbx
  12155d:	mov    %r10d,%edx
  121560:	pxor   %xmm4,%xmm4
  121564:	neg    %edx
  121566:	lea    -0x1(%rbp),%eax
  121569:	add    %ebp,%edx
  12156b:	cvtsi2sd %edx,%xmm4
  12156f:	mulsd  0x293d0c9(%rip),%xmm4        # 2a5e640 <_fini+0x1a3c>
  121577:	xor    %edx,%edx
  121579:	movslq %eax,%rax
  12157c:	addsd  %xmm2,%xmm4
  121580:	cmp    %r10,%rax
  121583:	jl     121746 <PRIVATE00000000002207de+0x1006>
  121589:	movslq %ebp,%rbp
  12158c:	movsd  0x293d0a4(%rip),%xmm2        # 2a5e638 <_fini+0x1a34>
  121594:	pxor   %xmm1,%xmm1
  121598:	mov    %r11,(%rsp)
  12159c:	mov    %r13,0x48(%rsp)
  1215a1:	mov    0x58(%rsp),%rsi
  1215a6:	lea    (%r9,%rbp,4),%rax
  1215aa:	mov    0x20(%rsp),%rdi
  1215af:	sub    %r10,%rbp
  1215b2:	mov    0x30(%rsp),%r9
  1215b7:	movslq -0x4(%rax),%rcx
  1215bb:	movsd  (%rbx,%rcx,8),%xmm3
  1215c0:	movaps %xmm3,%xmm5
  1215c3:	andps  0x293d026(%rip),%xmm5        # 2a5e5f0 <_fini+0x19ec>
  1215ca:	comisd %xmm0,%xmm5
  1215ce:	jbe    121728 <PRIVATE00000000002207de+0xfe8>
  1215d4:	divsd  (%rsi,%rcx,8),%xmm3
  1215d9:	movaps %xmm3,%xmm5
  1215dc:	andps  0x293d00d(%rip),%xmm5        # 2a5e5f0 <_fini+0x19ec>
  1215e3:	comisd %xmm0,%xmm5
  1215e7:	jbe    121728 <PRIVATE00000000002207de+0xfe8>
  1215ed:	mov    0x38(%rsp),%r10
  1215f2:	pxor   %xmm5,%xmm5
  1215f6:	mov    0x40(%rsp),%r8
  1215fb:	movsd  %xmm3,(%rbx,%rcx,8)
  121600:	mov    (%r10,%rcx,8),%r11
  121604:	mov    %r11,%r10
  121607:	mov    (%r8,%rcx,8),%r8
  12160b:	sub    %r8,%r10
  12160e:	cvtsi2sd %r10,%xmm5
  121613:	addsd  %xmm2,%xmm5
  121617:	mulsd  %xmm2,%xmm5
  12161b:	addsd  %xmm5,%xmm4
  12161f:	cmp    %r11,%r8
  121622:	jge    12172d <PRIVATE00000000002207de+0xfed>
  121628:	xor    %ecx,%ecx
  12162a:	mov    %r10,%r11
  12162d:	xor    %r12d,%r12d
  121630:	mov    $0x1,%r14d
  121636:	xor    %r13d,%r13d
  121639:	shr    $0x2,%r11
  12163d:	je     1216f0 <PRIVATE00000000002207de+0xfb0>
  121643:	lea    (%r9,%r8,4),%r15
  121647:	lea    (%rdi,%r8,8),%r14
  12164b:	movsd  0x0(%r13,%r14,1),%xmm5
  121652:	inc    %rcx
  121655:	mulsd  %xmm3,%xmm5
  121659:	movslq (%r12,%r15,1),%r9
  12165d:	movsd  (%rbx,%r9,8),%xmm6
  121663:	subsd  %xmm5,%xmm6
  121667:	movsd  %xmm6,(%rbx,%r9,8)
  12166d:	movsd  0x8(%r13,%r14,1),%xmm7
  121674:	mulsd  %xmm3,%xmm7
  121678:	movslq 0x4(%r12,%r15,1),%r9
  12167d:	movsd  (%rbx,%r9,8),%xmm8
  121683:	subsd  %xmm7,%xmm8
  121688:	movsd  %xmm8,(%rbx,%r9,8)
  12168e:	movsd  0x10(%r13,%r14,1),%xmm9
  121695:	mulsd  %xmm3,%xmm9
  12169a:	movslq 0x8(%r12,%r15,1),%r9
  12169f:	movsd  (%rbx,%r9,8),%xmm10
  1216a5:	subsd  %xmm9,%xmm10
  1216aa:	movsd  %xmm10,(%rbx,%r9,8)
  1216b0:	movsd  0x18(%r13,%r14,1),%xmm11
  1216b7:	add    $0x20,%r13
  1216bb:	mulsd  %xmm3,%xmm11
  1216c0:	movslq 0xc(%r12,%r15,1),%r9
  1216c5:	add    $0x10,%r12
  1216c9:	movsd  (%rbx,%r9,8),%xmm12
  1216cf:	subsd  %xmm11,%xmm12
  1216d4:	movsd  %xmm12,(%rbx,%r9,8)
  1216da:	cmp    %r11,%rcx
  1216dd:	jb     12164b <PRIVATE00000000002207de+0xf0b>
  1216e3:	mov    0x30(%rsp),%r9
  1216e8:	lea    0x1(,%rcx,4),%r14
  1216f0:	dec    %r14
  1216f3:	cmp    %r10,%r14
  1216f6:	jae    12172d <PRIVATE00000000002207de+0xfed>
  1216f8:	lea    (%r9,%r8,4),%r11
  1216fc:	lea    (%rdi,%r8,8),%rcx
  121700:	movsd  (%rcx,%r14,8),%xmm5
  121706:	mulsd  %xmm3,%xmm5
  12170a:	movslq (%r11,%r14,4),%r8
  12170e:	inc    %r14
  121711:	movsd  (%rbx,%r8,8),%xmm6
  121717:	subsd  %xmm5,%xmm6
  12171b:	movsd  %xmm6,(%rbx,%r8,8)
  121721:	cmp    %r10,%r14
  121724:	jb     121700 <PRIVATE00000000002207de+0xfc0>
  121726:	jmp    12172d <PRIVATE00000000002207de+0xfed>
  121728:	movsd  %xmm1,(%rbx,%rcx,8)
  12172d:	inc    %rdx
  121730:	add    $0xfffffffffffffffc,%rax
  121734:	cmp    %rbp,%rdx
  121737:	jb     1215b7 <PRIVATE00000000002207de+0xe77>
  12173d:	mov    (%rsp),%r11
  121741:	mov    0x48(%rsp),%r13
  121746:	pxor   %xmm0,%xmm0
  12174a:	xor    %eax,%eax
  12174c:	mov    0x78(%rsp),%edx
  121750:	cvtsi2sd %edx,%xmm0
  121754:	addsd  %xmm0,%xmm0
  121758:	addsd  %xmm4,%xmm0
  12175c:	test   %edx,%edx
  12175e:	jle    121d09 <PRIVATE00000000002207de+0x15c9>
  121764:	movslq 0x78(%rsp),%rdx
  121769:	mov    %r13,0x48(%rsp)
  12176e:	movsd  0x293cec2(%rip),%xmm1        # 2a5e638 <_fini+0x1a34>
  121776:	mov    %rdx,(%rsp)
  12177a:	mov    0x8(%rsp),%r14
  12177f:	mov    0x10(%rsp),%r13
  121784:	mov    0x18(%rsp),%r10
  121789:	mov    0x80(%rsp),%r9
  121791:	movslq (%r9,%rax,4),%r15
  121795:	mov    0x0(%r13,%r15,4),%edx
  12179a:	cmp    $0xffffffff,%edx
  12179d:	je     121cee <PRIVATE00000000002207de+0x15ae>
  1217a3:	cmp    $0xfffffffe,%edx
  1217a6:	je     121ce2 <PRIVATE00000000002207de+0x15a2>
  1217ac:	mov    0x60(%rsp),%r12
  1217b1:	pxor   %xmm2,%xmm2
  1217b5:	mov    0x68(%rsp),%rdi
  1217ba:	mov    (%r12,%r15,8),%r12
  1217be:	mov    %r12,%rdx
  1217c1:	mov    (%rdi,%r15,8),%rdi
  1217c5:	sub    %rdi,%rdx
  1217c8:	cvtsi2sd %rdx,%xmm2
  1217cd:	addsd  %xmm1,%xmm2
  1217d1:	mulsd  %xmm1,%xmm2
  1217d5:	addsd  %xmm2,%xmm0
  1217d9:	movsd  (%rbx,%r15,8),%xmm2
  1217df:	cmp    %r12,%rdi
  1217e2:	jge    121946 <PRIVATE00000000002207de+0x1206>
  1217e8:	lea    0x1(%rdi),%rdx
  1217ec:	sub    %rdx,%r12
  1217ef:	inc    %r12
  1217f2:	cmp    $0x8,%r12
  1217f6:	jl     121cda <PRIVATE00000000002207de+0x159a>
  1217fc:	lea    (%r14,%rdi,8),%rbp
  121800:	mov    %rbp,%rcx
  121803:	and    $0xf,%rcx
  121807:	je     12181b <PRIVATE00000000002207de+0x10db>
  121809:	test   $0x7,%rcx
  121810:	jne    121cda <PRIVATE00000000002207de+0x159a>
  121816:	mov    $0x1,%ecx
  12181b:	lea    0x8(%rcx),%rdx
  12181f:	cmp    %rdx,%r12
  121822:	jl     121cda <PRIVATE00000000002207de+0x159a>
  121828:	mov    %r12,%r8
  12182b:	xor    %edx,%edx
  12182d:	sub    %rcx,%r8
  121830:	lea    (%r10,%rdi,4),%rsi
  121834:	and    $0x7,%r8
  121838:	neg    %r8
  12183b:	add    %r12,%r8
  12183e:	test   %rcx,%rcx
  121841:	jbe    121867 <PRIVATE00000000002207de+0x1127>
  121843:	movslq (%rsi,%rdx,4),%r9
  121847:	movsd  0x0(%rbp,%rdx,8),%xmm3
  12184d:	inc    %rdx
  121850:	mulsd  (%rbx,%r9,8),%xmm3
  121856:	subsd  %xmm3,%xmm2
  12185a:	cmp    %rcx,%rdx
  12185d:	jb     121843 <PRIVATE00000000002207de+0x1103>
  12185f:	mov    0x80(%rsp),%r9
  121867:	pxor   %xmm4,%xmm4
  12186b:	movaps %xmm4,%xmm5
  12186e:	movaps %xmm4,%xmm3
  121871:	movsd  %xmm2,%xmm5
  121875:	movaps %xmm3,%xmm2
  121878:	movslq (%rsi,%rcx,4),%rdx
  12187c:	movsd  (%rbx,%rdx,8),%xmm6
  121881:	movslq 0x4(%rsi,%rcx,4),%rdx
  121886:	movhpd (%rbx,%rdx,8),%xmm6
  12188b:	movslq 0x8(%rsi,%rcx,4),%rdx
  121890:	mulpd  0x0(%rbp,%rcx,8),%xmm6
  121896:	movsd  (%rbx,%rdx,8),%xmm7
  12189b:	movslq 0xc(%rsi,%rcx,4),%rdx
  1218a0:	subpd  %xmm6,%xmm5
  1218a4:	movhpd (%rbx,%rdx,8),%xmm7
  1218a9:	movslq 0x10(%rsi,%rcx,4),%rdx
  1218ae:	mulpd  0x10(%rbp,%rcx,8),%xmm7
  1218b4:	movsd  (%rbx,%rdx,8),%xmm8
  1218ba:	movslq 0x14(%rsi,%rcx,4),%rdx
  1218bf:	subpd  %xmm7,%xmm4
  1218c3:	movhpd (%rbx,%rdx,8),%xmm8
  1218c9:	movslq 0x18(%rsi,%rcx,4),%rdx
  1218ce:	mulpd  0x20(%rbp,%rcx,8),%xmm8
  1218d5:	movsd  (%rbx,%rdx,8),%xmm9
  1218db:	movslq 0x1c(%rsi,%rcx,4),%rdx
  1218e0:	subpd  %xmm8,%xmm3
  1218e5:	movhpd (%rbx,%rdx,8),%xmm9
  1218eb:	mulpd  0x30(%rbp,%rcx,8),%xmm9
  1218f2:	add    $0x8,%rcx
  1218f6:	subpd  %xmm9,%xmm2
  1218fb:	cmp    %r8,%rcx
  1218fe:	jb     121878 <PRIVATE00000000002207de+0x1138>
  121904:	addpd  %xmm4,%xmm5
  121908:	addpd  %xmm2,%xmm3
  12190c:	addpd  %xmm3,%xmm5
  121910:	movaps %xmm5,%xmm2
  121913:	unpckhpd %xmm5,%xmm2
  121917:	addsd  %xmm2,%xmm5
  12191b:	movaps %xmm5,%xmm2
  12191e:	cmp    %r12,%r8
  121921:	jae    121946 <PRIVATE00000000002207de+0x1206>
  121923:	lea    (%r14,%rdi,8),%rcx
  121927:	lea    (%r10,%rdi,4),%rdx
  12192b:	movslq (%rdx,%r8,4),%rbp
  12192f:	movsd  (%rcx,%r8,8),%xmm3
  121935:	inc    %r8
  121938:	mulsd  (%rbx,%rbp,8),%xmm3
  12193d:	subsd  %xmm3,%xmm2
  121941:	cmp    %r12,%r8
  121944:	jb     12192b <PRIVATE00000000002207de+0x11eb>
  121946:	mov    0x58(%rsp),%rdx
  12194b:	divsd  (%rdx,%r15,8),%xmm2
  121951:	movsd  %xmm2,(%r11)
  121956:	inc    %rax
  121959:	add    $0x8,%r11
  12195d:	cmp    (%rsp),%rax
  121961:	jb     121791 <PRIVATE00000000002207de+0x1051>
  121967:	mov    0x48(%rsp),%r13
  12196c:	movsd  %xmm0,0x70(%rsp)
  121972:	jmp    1219b2 <PRIVATE00000000002207de+0x1272>
  121974:	add    $0xffffffffffffffe0,%rsp
  121978:	mov    %ebp,%edi
  12197a:	mov    0x98(%rsp),%eax
  121981:	mov    %rbx,%rsi
  121984:	mov    %r12,%rdx
  121987:	mov    %r15,%rcx
  12198a:	mov    %r9,%r8
  12198d:	mov    %r11,%r9
  121990:	mov    %eax,(%rsp)
  121993:	lea    0x90(%rsp),%r14
  12199b:	mov    0x10(%r14),%r10
  12199f:	mov    %r10,0x8(%rsp)
  1219a4:	mov    %r14,0x10(%rsp)
  1219a9:	call   123690 <PRIVATE000000000021574d>
  1219ae:	add    $0x20,%rsp
  1219b2:	mov    0x100(%rsp),%rax
  1219ba:	test   %rax,%rax
  1219bd:	je     1219d6 <PRIVATE00000000002207de+0x1296>
  1219bf:	movsd  0x70(%rsp),%xmm0
  1219c5:	mulsd  0x1f8(%r13),%xmm0
  1219ce:	addsd  (%rax),%xmm0
  1219d2:	movsd  %xmm0,(%rax)
  1219d6:	add    $0xa8,%rsp
  1219dd:	pop    %rbp
  1219de:	pop    %rbx
  1219df:	pop    %r15
  1219e1:	pop    %r14
  1219e3:	pop    %r13
  1219e5:	pop    %r12
  1219e7:	ret
  1219e8:	xor    %r12d,%r12d
  1219eb:	jmp    120cba <PRIVATE00000000002207de+0x57a>
  1219f0:	xor    %ecx,%ecx
  1219f2:	jmp    120a3d <PRIVATE00000000002207de+0x2fd>
  1219f7:	mov    0x8(%r15),%r10
  1219fb:	xor    %edi,%edi
  1219fd:	mov    %edx,(%r12)
  121a01:	mov    0x8(%r12),%rdx
  121a06:	mov    %r10,0x8(%r12)
  121a0b:	mov    0x10(%r15),%r10
  121a0f:	mov    %r10,0x10(%r12)
  121a14:	mov    0x28(%rsp),%r10
  121a19:	mov    %r10,0x10(%r15)
  121a1d:	mov    0x10(%r12),%r10
  121a22:	mov    %rdx,0x8(%r15)
  121a26:	movl   $0xffffffff,(%r15)
  121a2d:	mov    %r10,0x28(%rsp)
  121a32:	jmp    120b38 <PRIVATE00000000002207de+0x3f8>
  121a37:	xor    %r8d,%r8d
  121a3a:	jmp    121022 <PRIVATE00000000002207de+0x8e2>
  121a3f:	mov    0x8(%r15),%r10
  121a43:	mov    %edx,(%r12)
  121a47:	mov    0x8(%r12),%rdx
  121a4c:	mov    %r10,0x8(%r12)
  121a51:	mov    0x10(%r15),%r10
  121a55:	mov    0x10(%r12),%rcx
  121a5a:	mov    %r10,0x10(%r12)
  121a5f:	mov    %rcx,0x10(%r15)
  121a63:	mov    0x10(%r12),%r10
  121a68:	mov    %rdx,0x8(%r15)
  121a6c:	movl   $0xffffffff,(%r15)
  121a73:	mov    %r10,0x28(%rsp)
  121a78:	xor    %r10d,%r10d
  121a7b:	test   %eax,%eax
  121a7d:	jle    120d20 <PRIVATE00000000002207de+0x5e0>
  121a83:	movslq %eax,%rax
  121a86:	mov    %r14,0x8(%rsp)
  121a8b:	pxor   %xmm2,%xmm2
  121a8f:	mov    %r13,0x48(%rsp)
  121a94:	mov    %r12,0x10(%rsp)
  121a99:	mov    %r15,0x50(%rsp)
  121a9e:	mov    0x28(%rsp),%r8
  121aa3:	mov    0x90(%rsp),%rdi
  121aab:	movslq 0x4(%rdi),%r15
  121aaf:	movaps %xmm2,%xmm1
  121ab2:	mov    0x8(%rdi),%rdx
  121ab6:	mov    0x10(%rdi),%rsi
  121aba:	test   %r15,%r15
  121abd:	jle    121c0e <PRIVATE00000000002207de+0x14ce>
  121ac3:	cmp    $0x8,%r15
  121ac7:	jl     121c58 <PRIVATE00000000002207de+0x1518>
  121acd:	mov    %rsi,%r14
  121ad0:	and    $0xf,%r14
  121ad4:	test   %r14d,%r14d
  121ad7:	je     121aec <PRIVATE00000000002207de+0x13ac>
  121ad9:	test   $0x7,%r14d
  121ae0:	jne    121c58 <PRIVATE00000000002207de+0x1518>
  121ae6:	mov    $0x1,%r14d
  121aec:	mov    %r14d,%r13d
  121aef:	lea    0x8(%r13),%rcx
  121af3:	cmp    %rcx,%r15
  121af6:	jl     121c58 <PRIVATE00000000002207de+0x1518>
  121afc:	mov    %r15d,%r12d
  121aff:	mov    %r12d,%ecx
  121b02:	sub    %r14d,%ecx
  121b05:	and    $0x7,%ecx
  121b08:	sub    %ecx,%r12d
  121b0b:	xor    %ecx,%ecx
  121b0d:	movslq %r12d,%r12
  121b10:	test   %r14d,%r14d
  121b13:	jbe    121b30 <PRIVATE00000000002207de+0x13f0>
  121b15:	movslq (%rdx,%rcx,4),%r14
  121b19:	movsd  (%r8,%r14,8),%xmm3
  121b1f:	mulsd  (%rsi,%rcx,8),%xmm3
  121b24:	inc    %rcx
  121b27:	addsd  %xmm3,%xmm1
  121b2b:	cmp    %r13,%rcx
  121b2e:	jb     121b15 <PRIVATE00000000002207de+0x13d5>
  121b30:	pxor   %xmm4,%xmm4
  121b34:	movaps %xmm4,%xmm6
  121b37:	movaps %xmm4,%xmm3
  121b3a:	movsd  %xmm1,%xmm6
  121b3e:	movaps %xmm3,%xmm1
  121b41:	movslq (%rdx,%r13,4),%rcx
  121b45:	movslq 0x4(%rdx,%r13,4),%r14
  121b4a:	movsd  (%r8,%rcx,8),%xmm7
  121b50:	movslq 0x8(%rdx,%r13,4),%rcx
  121b55:	movhpd (%r8,%r14,8),%xmm7
  121b5b:	movslq 0xc(%rdx,%r13,4),%r14
  121b60:	movsd  (%r8,%rcx,8),%xmm8
  121b66:	movslq 0x10(%rdx,%r13,4),%rcx
  121b6b:	movhpd (%r8,%r14,8),%xmm8
  121b71:	movslq 0x14(%rdx,%r13,4),%r14
  121b76:	movsd  (%r8,%rcx,8),%xmm9
  121b7c:	movslq 0x18(%rdx,%r13,4),%rcx
  121b81:	movhpd (%r8,%r14,8),%xmm9
  121b87:	movslq 0x1c(%rdx,%r13,4),%r14
  121b8c:	movsd  (%r8,%rcx,8),%xmm10
  121b92:	mulpd  (%rsi,%r13,8),%xmm7
  121b98:	mulpd  0x10(%rsi,%r13,8),%xmm8
  121b9f:	mulpd  0x20(%rsi,%r13,8),%xmm9
  121ba6:	addpd  %xmm7,%xmm6
  121baa:	addpd  %xmm8,%xmm4
  121baf:	addpd  %xmm9,%xmm3
  121bb4:	movhpd (%r8,%r14,8),%xmm10
  121bba:	mulpd  0x30(%rsi,%r13,8),%xmm10
  121bc1:	add    $0x8,%r13
  121bc5:	addpd  %xmm10,%xmm1
  121bca:	cmp    %r12,%r13
  121bcd:	jb     121b41 <PRIVATE00000000002207de+0x1401>
  121bd3:	addpd  %xmm4,%xmm6
  121bd7:	addpd  %xmm1,%xmm3
  121bdb:	addpd  %xmm3,%xmm6
  121bdf:	movaps %xmm6,%xmm1
  121be2:	unpckhpd %xmm6,%xmm1
  121be6:	addsd  %xmm1,%xmm6
  121bea:	movaps %xmm6,%xmm1
  121bed:	cmp    %r15,%r12
  121bf0:	jae    121c0e <PRIVATE00000000002207de+0x14ce>
  121bf2:	movslq (%rdx,%r12,4),%rcx
  121bf6:	movsd  (%r8,%rcx,8),%xmm3
  121bfc:	mulsd  (%rsi,%r12,8),%xmm3
  121c02:	inc    %r12
  121c05:	addsd  %xmm3,%xmm1
  121c09:	cmp    %r15,%r12
  121c0c:	jb     121bf2 <PRIVATE00000000002207de+0x14b2>
  121c0e:	movaps %xmm1,%xmm3
  121c11:	andps  0x293c9d8(%rip),%xmm3        # 2a5e5f0 <_fini+0x19ec>
  121c18:	movslq (%rdi),%rcx
  121c1b:	comisd %xmm0,%xmm3
  121c1f:	jbe    121c29 <PRIVATE00000000002207de+0x14e9>
  121c21:	movsd  %xmm1,(%r8,%rcx,8)
  121c27:	jmp    121c2f <PRIVATE00000000002207de+0x14ef>
  121c29:	movsd  %xmm2,(%r8,%rcx,8)
  121c2f:	inc    %r10
  121c32:	add    $0x18,%rdi
  121c36:	cmp    %rax,%r10
  121c39:	jb     121aab <PRIVATE00000000002207de+0x136b>
  121c3f:	mov    0x8(%rsp),%r14
  121c44:	mov    0x48(%rsp),%r13
  121c49:	mov    0x10(%rsp),%r12
  121c4e:	mov    0x50(%rsp),%r15
  121c53:	jmp    120d20 <PRIVATE00000000002207de+0x5e0>
  121c58:	xor    %r12d,%r12d
  121c5b:	jmp    121bed <PRIVATE00000000002207de+0x14ad>
  121c5d:	test   %edx,%edx
  121c5f:	jl     121c91 <PRIVATE00000000002207de+0x1551>
  121c61:	mov    0x68(%rsp),%eax
  121c65:	add    0x60(%rsp),%eax
  121c69:	add    %edx,%eax
  121c6b:	cmp    %ebp,%eax
  121c6d:	jl     121cab <PRIVATE00000000002207de+0x156b>
  121c6f:	pxor   %xmm2,%xmm2
  121c73:	pxor   %xmm1,%xmm1
  121c77:	cvtsi2sd %ebp,%xmm2
  121c7b:	cvtsi2sd %edx,%xmm1
  121c7f:	mulsd  %xmm5,%xmm5
  121c83:	addsd  %xmm2,%xmm2
  121c87:	mulsd  %xmm1,%xmm5
  121c8b:	comisd %xmm2,%xmm5
  121c8f:	jb     121cab <PRIVATE00000000002207de+0x156b>
  121c91:	mov    %ebp,%edi
  121c93:	mov    %rbx,%rsi
  121c96:	mov    %r12,%rdx
  121c99:	mov    %r15,%rcx
  121c9c:	lea    0x70(%rsp),%r8
  121ca1:	call   121ef0 <PRIVATE00000000002126cd>
  121ca6:	jmp    1219b2 <PRIVATE00000000002207de+0x1272>
  121cab:	add    $0xfffffffffffffff0,%rsp
  121caf:	mov    %ebp,%edi
  121cb1:	mov    %rbx,%rsi
  121cb4:	mov    %r12,%rdx
  121cb7:	mov    %r15,%rcx
  121cba:	mov    %r9,%r8
  121cbd:	mov    %r11,%r9
  121cc0:	lea    0x80(%rsp),%rax
  121cc8:	mov    %rax,(%rsp)
  121ccc:	call   1223e0 <PRIVATE000000000021686f>
  121cd1:	add    $0x10,%rsp
  121cd5:	jmp    1219b2 <PRIVATE00000000002207de+0x1272>
  121cda:	xor    %r8d,%r8d
  121cdd:	jmp    12191e <PRIVATE00000000002207de+0x11de>
  121ce2:	mov    (%rbx,%r15,8),%rdx
  121ce6:	mov    %rdx,(%r11)
  121ce9:	jmp    121956 <PRIVATE00000000002207de+0x1216>
  121cee:	mov    0x58(%rsp),%rdx
  121cf3:	movsd  (%rbx,%r15,8),%xmm2
  121cf9:	divsd  (%rdx,%r15,8),%xmm2
  121cff:	movsd  %xmm2,(%r11)
  121d04:	jmp    121956 <PRIVATE00000000002207de+0x1216>
  121d09:	movsd  %xmm0,0x70(%rsp)
  121d0f:	jmp    1219b2 <PRIVATE00000000002207de+0x1272>
  121d14:	xor    %edi,%edi
  121d16:	jmp    12151b <PRIVATE00000000002207de+0xddb>
  121d1b:	xor    %ecx,%ecx
  121d1d:	jmp    121456 <PRIVATE00000000002207de+0xd16>
  121d22:	movsd  0x70(%rsp),%xmm2
  121d28:	jmp    12155d <PRIVATE00000000002207de+0xe1d>
  121d2d:	xor    %ecx,%ecx
  121d2f:	test   %ebp,%ebp
  121d31:	jle    121e03 <PRIVATE00000000002207de+0x16c3>
  121d37:	mov    %ebp,%eax
  121d39:	xor    %esi,%esi
  121d3b:	mov    $0x1,%r10d
  121d41:	xor    %edx,%edx
  121d43:	shr    %eax
  121d45:	je     121dc3 <PRIVATE00000000002207de+0x1683>
  121d47:	mov    0x28(%rsp),%r10
  121d4c:	movsd  (%rdx,%r10,1),%xmm3
  121d52:	movaps %xmm3,%xmm2
  121d55:	andps  0x293c894(%rip),%xmm2        # 2a5e5f0 <_fini+0x19ec>
  121d5c:	comisd %xmm0,%xmm2
  121d60:	jbe    121d7e <PRIVATE00000000002207de+0x163e>
  121d62:	mov    0x8(%r14),%r8
  121d66:	mov    %esi,%edi
  121d68:	add    %edi,%edi
  121d6a:	mulsd  %xmm1,%xmm3
  121d6e:	mov    %edi,(%r8,%rcx,4)
  121d72:	inc    %rcx
  121d75:	mov    0x10(%r14),%rdi
  121d79:	movsd  %xmm3,(%rdi,%rdx,1)
  121d7e:	movsd  0x8(%rdx,%r10,1),%xmm3
  121d85:	movaps %xmm3,%xmm2
  121d88:	andps  0x293c861(%rip),%xmm2        # 2a5e5f0 <_fini+0x19ec>
  121d8f:	comisd %xmm0,%xmm2
  121d93:	jbe    121db2 <PRIVATE00000000002207de+0x1672>
  121d95:	mov    0x8(%r14),%r8
  121d99:	lea    0x1(%rsi,%rsi,1),%edi
  121d9d:	mulsd  %xmm1,%xmm3
  121da1:	mov    %edi,(%r8,%rcx,4)
  121da5:	inc    %rcx
  121da8:	mov    0x10(%r14),%rdi
  121dac:	movsd  %xmm3,0x8(%rdi,%rdx,1)
  121db2:	inc    %rsi
  121db5:	add    $0x10,%rdx
  121db9:	cmp    %rax,%rsi
  121dbc:	jb     121d4c <PRIVATE00000000002207de+0x160c>
  121dbe:	lea    0x1(%rsi,%rsi,1),%r10d
  121dc3:	lea    -0x1(%r10),%edx
  121dc7:	cmp    %ebp,%edx
  121dc9:	jae    121e03 <PRIVATE00000000002207de+0x16c3>
  121dcb:	movslq %r10d,%r10
  121dce:	mov    0x28(%rsp),%rax
  121dd3:	movsd  -0x8(%rax,%r10,8),%xmm3
  121dda:	movaps %xmm3,%xmm2
  121ddd:	andps  0x293c80c(%rip),%xmm2        # 2a5e5f0 <_fini+0x19ec>
  121de4:	comisd %xmm0,%xmm2
  121de8:	jbe    121e03 <PRIVATE00000000002207de+0x16c3>
  121dea:	mov    0x8(%r14),%rax
  121dee:	mulsd  %xmm3,%xmm1
  121df2:	mov    %edx,(%rax,%rcx,4)
  121df5:	inc    %rcx
  121df8:	mov    0x10(%r14),%rax
  121dfc:	movsd  %xmm1,-0x8(%rax,%r10,8)
  121e03:	mov    %ecx,(%r14)
  121e06:	mov    (%r12),%edx
  121e0a:	jmp    121216 <PRIVATE00000000002207de+0xad6>
  121e0f:	mov    0x10(%r15),%rsi
  121e13:	mov    0x10(%r12),%rdi
  121e18:	mov    %rsi,0x8(%rsp)
  121e1d:	mov    %rdi,(%rsp)
  121e21:	test   %ebp,%ebp
  121e23:	jle    121eb2 <PRIVATE00000000002207de+0x1772>
  121e29:	mov    %ebp,%r8d
  121e2c:	xor    %edi,%edi
  121e2e:	mov    $0x1,%esi
  121e33:	shr    %r8d
  121e36:	je     121e91 <PRIVATE00000000002207de+0x1751>
  121e38:	mov    %r13,0x48(%rsp)
  121e3d:	mov    %r12,0x10(%rsp)
  121e42:	mov    %r15,0x50(%rsp)
  121e47:	mov    %r8d,%esi
  121e4a:	mov    %rdi,%r15
  121e4d:	mov    (%rsp),%r13
  121e51:	mov    0x8(%rsp),%r12
  121e56:	movslq (%rcx,%rdi,8),%r8
  121e5a:	mov    0x0(%r13,%r8,8),%r8
  121e5f:	mov    %r8,(%r15,%r12,1)
  121e63:	movslq 0x4(%rcx,%rdi,8),%r8
  121e68:	inc    %rdi
  121e6b:	mov    0x0(%r13,%r8,8),%r8
  121e70:	mov    %r8,0x8(%r15,%r12,1)
  121e75:	add    $0x10,%r15
  121e79:	cmp    %rsi,%rdi
  121e7c:	jb     121e56 <PRIVATE00000000002207de+0x1716>
  121e7e:	mov    0x48(%rsp),%r13
  121e83:	lea    0x1(%rdi,%rdi,1),%esi
  121e87:	mov    0x10(%rsp),%r12
  121e8c:	mov    0x50(%rsp),%r15
  121e91:	lea    -0x1(%rsi),%edi
  121e94:	cmp    %ebp,%edi
  121e96:	jae    121eb2 <PRIVATE00000000002207de+0x1772>
  121e98:	movslq %esi,%rsi
  121e9b:	movslq -0x4(%rcx,%rsi,4),%rdi
  121ea0:	mov    (%rsp),%rcx
  121ea4:	mov    (%rcx,%rdi,8),%r8
  121ea8:	mov    0x8(%rsp),%rcx
  121ead:	mov    %r8,-0x8(%rcx,%rsi,8)
  121eb2:	mov    %edx,(%r15)
  121eb5:	mov    0x8(%r12),%rdx
  121eba:	mov    (%rsp),%rsi
  121ebe:	mov    0x8(%rsp),%rdi
  121ec3:	mov    0x8(%r15),%rcx
  121ec7:	mov    %rdx,0x8(%r15)
  121ecb:	mov    %rsi,0x10(%r15)
  121ecf:	mov    %rcx,0x8(%r12)
  121ed4:	mov    %rdi,0x10(%r12)
  121ed9:	movl   $0xffffffff,(%r12)
  121ee1:	jmp    12088a <PRIVATE00000000002207de+0x14a>
