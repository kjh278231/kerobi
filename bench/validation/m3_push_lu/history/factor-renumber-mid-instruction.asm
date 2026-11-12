00000000000f44d0 <PRIVATE000000000005b257+0x2f10>:
   f44d0:	rex.XB or $0x0,%al
   f44d3:	add    %al,(%rax)
   f44d5:	add    %al,0xf024bc(%rbx)
   f44db:	add    %al,(%rax)
   f44dd:	add    %cl,(%rdi)
   f44df:	mov    %edx,%ds
   f44e1:	adc    %eax,(%rax)
   f44e3:	add    %cl,-0x54(%rbx,%riz,2)
   f44e7:	and    $0xf0,%al
   f44e9:	add    %al,(%rax)
   f44eb:	add    %cl,-0x75(%rax)
   f44ee:	xorl   $0xffffff8b,0x4c000000(%rax)
   f44f5:	movsxd 0x4e(%rax),%ebx
   f44f8:	mov    (%rax,%rbp,8),%ebx
   f44fb:	add    %r13,%r11
   f44fe:	mov    %r11,0x1e8(%rbx)
   f4505:	add    (%r12,%r13,8),%r13
   f4509:	mov    %r13,0x1e0(%rbx)
   f4510:	cmpq   $0x0,0x118(%rsp)
   f4519:	je     f4594 <PRIVATE000000000005b257+0x2fd4>
   f451b:	cmpl   $0x0,0x3ec(%r14)
   f4523:	je     f454a <PRIVATE000000000005b257+0x2f8a>
   f4525:	cmpl   $0x0,0x450(%r14)
   f452d:	jne    f4539 <PRIVATE000000000005b257+0x2f79>
   f452f:	cmpl   $0x0,0x420(%r14)
   f4537:	je     f454a <PRIVATE000000000005b257+0x2f8a>
   f4539:	mov    0x118(%rsp),%rax
   f4541:	movl   $0x0,0x50(%rax)
   f4548:	jmp    f4594 <PRIVATE000000000005b257+0x2fd4>
   f454a:	cmpl   $0x0,0x58(%rsp)
   f454f:	jne    f4562 <PRIVATE000000000005b257+0x2fa2>
   f4551:	mov    0x118(%rsp),%rax
   f4559:	movl   $0xa,0x50(%rax)
   f4560:	jmp    f4594 <PRIVATE000000000005b257+0x2fd4>
   f4562:	pxor   %xmm0,%xmm0
   f4566:	mov    $0xa,%r11d
   f456c:	cvtsi2sdl 0x1f0(%rbx),%xmm0
   f4574:	mulsd  0x2969824(%rip),%xmm0        # 2a5dda0 <_fini+0x119c>
   f457c:	cvttsd2si %xmm0,%eax
   f4580:	mov    0x118(%rsp),%r12
   f4588:	cmp    $0xa,%eax
   f458b:	cmovg  %eax,%r11d
   f458f:	mov    %r11d,0x50(%r12)
   f4594:	mov    0x1c8(%rbx),%r13
   f459b:	mov    0x0(%r13),%eax
   f459f:	test   %eax,%eax
   f45a1:	jle    f461f <PRIVATE000000000005b257+0x305f>
   f45a3:	mov    %eax,%r15d
   f45a6:	xor    %esi,%esi
   f45a8:	mov    0x8(%r13),%r11
   f45ac:	mov    $0x1,%ecx
   f45b1:	mov    0x10(%r13),%r12
   f45b5:	xor    %edx,%edx
   f45b7:	shr    %r15d
   f45ba:	je     f45e0 <PRIVATE000000000005b257+0x3020>
   f45bc:	xor    %edi,%edi
   f45be:	movslq %edx,%rdx
   f45c1:	movslq (%r11,%rdx,8),%rcx
   f45c5:	mov    %rdi,(%r12,%rcx,8)
   f45c9:	movslq 0x4(%r11,%rdx,8),%rsi
   f45ce:	inc    %edx
   f45d0:	mov    %rdi,(%r12,%rsi,8)
   f45d4:	cmp    %r15d,%edx
   f45d7:	jb     f45be <PRIVATE000000000005b257+0x2ffe>
   f45d9:	lea    (%rdx,%rdx,1),%esi
   f45dc:	lea    0x1(%rdx,%rdx,1),%ecx
   f45e0:	lea    -0x1(%rcx),%r15d
   f45e4:	cmp    %eax,%r15d
   f45e7:	jae    f45fb <PRIVATE000000000005b257+0x303b>
   f45e9:	movslq %ecx,%rax
   f45ec:	mov    %ecx,%esi
   f45ee:	movslq -0x4(%r11,%rax,4),%r11
   f45f3:	movq   $0x0,(%r12,%r11,8)
   f45fb:	test   %rbp,%rbp
   f45fe:	je     f461f <PRIVATE000000000005b257+0x305f>
   f4600:	pxor   %xmm0,%xmm0
   f4604:	cvtsi2sd %esi,%xmm0
   f4608:	addsd  %xmm0,%xmm0
   f460c:	mulsd  0x148(%r14),%xmm0
   f4615:	addsd  0x0(%rbp),%xmm0
   f461a:	movsd  %xmm0,0x0(%rbp)
   f461f:	movl   $0x0,0x0(%r13)
   f4627:	cmpl   $0x0,0xf0(%rsp)
   f462f:	jle    f4ee2 <PRIVATE000000000005b257+0x3922>
   f4635:	mov    0x478(%r14),%r13
   f463c:	test   %r13,%r13
   f463f:	je     f4905 <PRIVATE000000000005b257+0x3345>
   f4645:	mov    0x470(%r14),%rdi
   f464c:	mov    0x168(%r14),%r12
   f4653:	mov    %r12,(%rsp)
   f4657:	mov    %rdi,0x10(%rsp)
   f465c:	mov    0x18(%rdi),%rax
   f4660:	mov    0x38(%rdi),%r11
   f4664:	mov    0x28(%rdi),%r12
   f4668:	mov    0x1c8(%rdi),%rdi
   f466f:	mov    %r12,0x18(%rsp)
   f4674:	mov    %rax,0x8(%rsp)
   f4679:	mov    0x8(%r13),%r12
   f467d:	mov    0x18(%r13),%rax
   f4681:	mov    0x294(%r14),%edx
   f4688:	mov    %r12,0x30(%rsp)
   f468d:	mov    0x64(%r14),%r15d
   f4691:	mov    %r11,0x28(%rsp)
   f4696:	mov    0x8(%rdi),%r12
   f469a:	mov    %rax,0x20(%rsp)
   f469f:	mov    0x2f8(%r14),%esi
   f46a6:	cmp    $0x9,%edx
   f46a9:	je     f472e <PRIVATE000000000005b257+0x316e>
   f46af:	cmp    $0x1,%edx
   f46b2:	je     f53f4 <PRIVATE000000000005b257+0x3e34>
   f46b8:	cmpl   $0x0,0x290(%r14)
   f46c0:	jne    f472e <PRIVATE000000000005b257+0x316e>
   f46c2:	cmpl   $0x0,0x2fc(%r14)
   f46ca:	jl     f472e <PRIVATE000000000005b257+0x316e>
   f46cc:	test   %esi,%esi
   f46ce:	jl     f474e <PRIVATE000000000005b257+0x318e>
   f46d0:	mov    0x3ec(%r14),%eax
   f46d7:	cmp    $0x2,%eax
   f46da:	je     f52a1 <PRIVATE000000000005b257+0x3ce1>
   f46e0:	test   %eax,%eax
   f46e2:	je     f4732 <PRIVATE000000000005b257+0x3172>
   f46e4:	movsd  0x300(%r14),%xmm0
   f46ed:	mov    %r14,%rdi
   f46f0:	movsd  0x310(%r14),%xmm1
   f46f9:	mov    %rbp,%r9
   f46fc:	mov    0x210(%r14),%rdx
   f4703:	mov    0x218(%r14),%rcx
   f470a:	mov    0x30(%rsp),%r8
   f470f:	call   be6b0 <PRIVATE0000000000016b0a>
   f4714:	mov    0x10(%rsp),%rax
   f4719:	mov    0x2f8(%r14),%esi
   f4720:	mov    0x294(%r14),%edx
   f4727:	mov    0x1c8(%rax),%rdi
   f472e:	test   %esi,%esi
   f4730:	jl     f4745 <PRIVATE000000000005b257+0x3185>
   f4732:	movslq %esi,%rsi
   f4735:	mov    0x28(%rsp),%rax
   f473a:	mov    (%rax,%rsi,4),%r11d
   f473e:	mov    %r11d,0x2f8(%r14)
   f4745:	cmp    $0x1,%edx
   f4748:	je     f5355 <PRIVATE000000000005b257+0x3d95>
   f474e:	test   %r15d,%r15d
   f4751:	jle    f4804 <PRIVATE000000000005b257+0x3244>
   f4757:	mov    %r15d,%r11d
   f475a:	xor    %edx,%edx
   f475c:	mov    $0x1,%ecx
   f4761:	xor    %esi,%esi
   f4763:	shr    %r11d
   f4766:	je     f47c7 <PRIVATE000000000005b257+0x3207>
   f4768:	mov    %rbp,0x90(%rsp)
   f4770:	mov    0x20(%rsp),%rbp
   f4775:	mov    0x30(%rsp),%r8
   f477a:	mov    0x18(%rsp),%rax
   f477f:	mov    0x8(%rsp),%rcx
   f4784:	movslq (%rax,%rdx,8),%r10
   f4788:	mov    (%rcx,%r10,4),%r9d
   f478c:	mov    %r9d,(%r12,%rdx,8)
   f4790:	mov    (%r8,%r10,8),%r9
   f4794:	movslq 0x4(%rax,%rdx,8),%r10
   f4799:	mov    %r9,(%rsi,%rbp,1)
   f479d:	mov    (%rcx,%r10,4),%r9d
   f47a1:	mov    %r9d,0x4(%r12,%rdx,8)
   f47a6:	inc    %rdx
   f47a9:	mov    (%r8,%r10,8),%r9
   f47ad:	mov    %r9,0x8(%rsi,%rbp,1)
   f47b2:	add    $0x10,%rsi
   f47b6:	cmp    %r11,%rdx
   f47b9:	jb     f4784 <PRIVATE000000000005b257+0x31c4>
   f47bb:	mov    0x90(%rsp),%rbp
   f47c3:	lea    0x1(%rdx,%rdx,1),%ecx
   f47c7:	lea    -0x1(%rcx),%eax
   f47ca:	cmp    %r15d,%eax
   f47cd:	jae    f5e60 <PRIVATE000000000005b257+0x48a0>
   f47d3:	movslq %ecx,%rcx
   f47d6:	test   %r15d,%r15d
   f47d9:	mov    0x18(%rsp),%rax
   f47de:	mov    0x8(%rsp),%r11
   f47e3:	movslq -0x4(%rax,%rcx,4),%rax
   f47e8:	mov    (%r11,%rax,4),%r11d
   f47ec:	mov    %r11d,-0x4(%r12,%rcx,4)
   f47f1:	mov    0x30(%rsp),%r11
   f47f6:	mov    (%r11,%rax,8),%rax
   f47fa:	mov    0x20(%rsp),%r11
   f47ff:	mov    %rax,-0x8(%r11,%rcx,8)
   f4804:	mov    0x8(%rsp),%rax
   f4809:	mov    %rax,0x8(%rdi)
   f480d:	mov    0x30(%rsp),%r11
   f4812:	mov    0x20(%rsp),%rax
   f4817:	mov    %r11,0x18(%r13)
   f481b:	mov    %rax,0x8(%r13)
   f481f:	mov    0x10(%rsp),%r13
   f4824:	mov    %r12,0x18(%r13)
   f4828:	jle    f4881 <PRIVATE000000000005b257+0x32c1>
   f482a:	mov    %r15d,%eax
   f482d:	mov    $0x1,%r13d
   f4833:	xor    %r11d,%r11d
   f4836:	shr    %eax
   f4838:	je     f4868 <PRIVATE000000000005b257+0x32a8>
   f483a:	mov    (%rsp),%rsi
   f483e:	movslq %r11d,%r11
   f4841:	lea    (%r11,%r11,1),%r13d
   f4845:	movslq (%r12,%r11,8),%rdx
   f4849:	mov    %r13d,(%rsi,%rdx,4)
   f484d:	lea    0x1(%r11,%r11,1),%r13d
   f4852:	movslq 0x4(%r12,%r11,8),%rcx
   f4857:	inc    %r11d
   f485a:	mov    %r13d,(%rsi,%rcx,4)
   f485e:	cmp    %eax,%r11d
   f4861:	jb     f483e <PRIVATE000000000005b257+0x327e>
   f4863:	lea    0x1(%r11,%r11,1),%r13d
   f4868:	lea    -0x1(%r13),%eax
   f486c:	cmp    %r15d,%eax
   f486f:	jae    f4881 <PRIVATE000000000005b257+0x32c1>
   f4871:	movslq %r13d,%r13
   f4874:	mov    (%rsp),%r11
   f4878:	movslq -0x4(%r12,%r13,4),%r12
   f487d:	mov    %eax,(%r11,%r12,4)
   f4881:	test   %rbp,%rbp
   f4884:	je     f48ae <PRIVATE000000000005b257+0x32ee>
   f4886:	pxor   %xmm0,%xmm0
   f488a:	cvtsi2sd %r15d,%xmm0
   f488f:	mulsd  0x2969501(%rip),%xmm0        # 2a5dd98 <_fini+0x1194>
   f4897:	mov    0x10(%rsp),%rax
   f489c:	mulsd  0x1f8(%rax),%xmm0
   f48a4:	addsd  0x0(%rbp),%xmm0
   f48a9:	movsd  %xmm0,0x0(%rbp)
   f48ae:	mov    0xf8(%rsp),%rax
   f48b6:	cmpl   $0x0,0x44(%rax)
   f48ba:	jle    f5e53 <PRIVATE000000000005b257+0x4893>
   f48c0:	mov    0x4b8(%r14),%r11
   f48c7:	mov    0xe8(%r11),%rax
   f48ce:	cmpl   $0x4,(%rax)
   f48d1:	je     f52b1 <PRIVATE000000000005b257+0x3cf1>
   f48d7:	mov    0xf8(%rsp),%rax
   f48df:	mov    %rbx,%rdi
   f48e2:	mov    0xf0(%rsp),%r12d
   f48ea:	mov    %r12d,%edx
   f48ed:	mov    %rbp,%r8
   f48f0:	mov    0x68(%rax),%rcx
   f48f4:	lea    0x44(%rax),%rsi
   f48f8:	call   132460 <PRIVATE000000000009e60b>
   f48fd:	test   %r12d,%r12d
   f4900:	jmp    f4ee2 <PRIVATE000000000005b257+0x3922>
   f4905:	mov    0x470(%r14),%r11
   f490c:	mov    0x480(%r14),%r15
   f4913:	mov    %r11,0x38(%rsp)
   f4918:	mov    0x18(%r11),%rax
   f491c:	mov    0x28(%r11),%r13
   f4920:	mov    0x168(%r14),%r12
   f4927:	mov    %rax,0x30(%rsp)
   f492c:	test   %r15,%r15
   f492f:	je     f55bb <PRIVATE000000000005b257+0x3ffb>
   f4935:	mov    %r11,%rdx
   f4938:	mov    0x48(%r15),%r11
   f493c:	mov    %r11,0x20(%rsp)
   f4941:	mov    0x1c8(%rdx),%r11
   f4948:	mov    %r11,0x18(%rsp)
   f494d:	mov    0x64(%r14),%eax
   f4951:	mov    0x8(%r11),%r11
   f4955:	mov    %r11,0x10(%rsp)
   f495a:	mov    0x38(%r15),%r11
   f495e:	mov    %eax,0x28(%rsp)
   f4962:	mov    0x38(%rdx),%rax
   f4966:	mov    %r11,0x8(%rsp)
   f496b:	movsd  0x3a8(%r14),%xmm0
   f4974:	mov    0x280(%r14),%rdx
   f497b:	mov    0x200(%r14),%r11
   f4982:	cmpl   $0x0,0x3ec(%r14)
   f498a:	jne    f4bfd <PRIVATE000000000005b257+0x363d>
   f4990:	cmpl   $0x9,0x294(%r14)
   f4998:	je     f4bfd <PRIVATE000000000005b257+0x363d>
   f499e:	cmpl   $0x0,0x78(%r15)
   f49a3:	je     f4bfd <PRIVATE000000000005b257+0x363d>
   f49a9:	mov    0x210(%r14),%rcx
   f49b0:	mov    0x1e8(%r14),%r10
   f49b7:	mov    %rcx,(%rsp)
   f49bb:	cmpl   $0x0,0x2fc(%r14)
   f49c3:	jl     f49f8 <PRIVATE000000000005b257+0x3438>
   f49c5:	mov    (%rcx),%r8d
   f49c8:	test   %r8d,%r8d
   f49cb:	jl     f5409 <PRIVATE000000000005b257+0x3e49>
   f49d1:	xor    %edi,%edi
   f49d3:	test   %r8d,%r8d
   f49d6:	jle    f49f8 <PRIVATE000000000005b257+0x3438>
   f49d8:	mov    %rcx,%rsi
   f49db:	mov    0x8(%rcx),%rcx
   f49df:	movslq (%rcx),%r8
   f49e2:	inc    %rdi
   f49e5:	mov    (%rax,%r8,4),%r9d
   f49e9:	mov    %r9d,(%rcx)
   f49ec:	add    $0x4,%rcx
   f49f0:	movslq (%rsi),%r8
   f49f3:	cmp    %r8,%rdi
   f49f6:	jl     f49df <PRIVATE000000000005b257+0x341f>
   f49f8:	cmpl   $0x0,0x28(%rsp)
   f49fd:	jle    f4bec <PRIVATE000000000005b257+0x362c>
   f4a03:	cmpl   $0x6,0x28(%rsp)
   f4a08:	jle    f4b16 <PRIVATE000000000005b257+0x3556>
   f4a0e:	movslq 0x28(%rsp),%rdi
   f4a13:	mov    0x8(%rsp),%rcx
   f4a18:	sub    %r10,%rcx
   f4a1b:	lea    0x0(,%rdi,8),%rsi
   f4a23:	cmp    %rsi,%rcx
   f4a26:	jge    f4a34 <PRIVATE000000000005b257+0x3474>
   f4a28:	neg    %rcx
   f4a2b:	cmp    %rsi,%rcx
   f4a2e:	jl     f4b16 <PRIVATE000000000005b257+0x3556>
   f4a34:	cmp    $0x2,%rdi
   f4a38:	jl     f5401 <PRIVATE000000000005b257+0x3e41>
   f4a3e:	mov    0x8(%rsp),%rcx
   f4a43:	and    $0xf,%rcx
   f4a47:	test   %ecx,%ecx
   f4a49:	je     f4a59 <PRIVATE000000000005b257+0x3499>
   f4a4b:	test   $0x7,%cl
   f4a4e:	jne    f5401 <PRIVATE000000000005b257+0x3e41>
   f4a54:	mov    $0x1,%ecx
   f4a59:	mov    %ecx,%ecx
   f4a5b:	mov    %rcx,(%rsp)
   f4a5f:	lea    0x2(%rcx),%rsi
   f4a63:	cmp    %rsi,%rdi
   f4a66:	jl     f5401 <PRIVATE000000000005b257+0x3e41>
   f4a6c:	mov    0x28(%rsp),%r8d
   f4a71:	mov    %r8d,%esi
   f4a74:	sub    %ecx,%esi
   f4a76:	and    $0x1,%esi
   f4a79:	neg    %esi
   f4a7b:	add    %r8d,%esi
   f4a7e:	movslq %esi,%r8
   f4a81:	xor    %esi,%esi
   f4a83:	test   %ecx,%ecx
   f4a85:	jbe    f4ab3 <PRIVATE000000000005b257+0x34f3>
   f4a87:	mov    %rbp,0x90(%rsp)
   f4a8f:	mov    %rcx,%rbp
   f4a92:	mov    0x8(%rsp),%r9
   f4a97:	mov    (%r10,%rsi,8),%rcx
   f4a9b:	mov    %rcx,(%r9,%rsi,8)
   f4a9f:	inc    %rsi
   f4aa2:	cmp    %rbp,%rsi
   f4aa5:	jb     f4a97 <PRIVATE000000000005b257+0x34d7>
   f4aa7:	mov    %rbp,(%rsp)
   f4aab:	mov    0x90(%rsp),%rbp
   f4ab3:	mov    (%rsp),%rcx
   f4ab7:	lea    (%r10,%rcx,8),%rsi
   f4abb:	test   $0xf,%rsi
   f4ac2:	je     f4add <PRIVATE000000000005b257+0x351d>
   f4ac4:	mov    0x8(%rsp),%rsi
   f4ac9:	movups (%r10,%rcx,8),%xmm0
   f4ace:	movups %xmm0,(%rsi,%rcx,8)
   f4ad2:	add    $0x2,%rcx
   f4ad6:	cmp    %r8,%rcx
   f4ad9:	jb     f4ac9 <PRIVATE000000000005b257+0x3509>
   f4adb:	jmp    f4af4 <PRIVATE000000000005b257+0x3534>
   f4add:	mov    0x8(%rsp),%rsi
   f4ae2:	movups (%r10,%rcx,8),%xmm0
   f4ae7:	movups %xmm0,(%rsi,%rcx,8)
   f4aeb:	add    $0x2,%rcx
   f4aef:	cmp    %r8,%rcx
   f4af2:	jb     f4ae2 <PRIVATE000000000005b257+0x3522>
   f4af4:	cmp    %rdi,%r8
   f4af7:	jae    f4b0e <PRIVATE000000000005b257+0x354e>
   f4af9:	mov    0x8(%rsp),%rsi
   f4afe:	mov    (%r10,%r8,8),%rcx
   f4b02:	mov    %rcx,(%rsi,%r8,8)
   f4b06:	inc    %r8
   f4b09:	cmp    %rdi,%r8
   f4b0c:	jb     f4afe <PRIVATE000000000005b257+0x353e>
   f4b0e:	mov    0x28(%rsp),%edi
   f4b12:	shr    %edi
   f4b14:	jmp    f4b7f <PRIVATE000000000005b257+0x35bf>
   f4b16:	xor    %ecx,%ecx
   f4b18:	mov    $0x1,%r8d
   f4b1e:	mov    0x28(%rsp),%edi
   f4b22:	xor    %esi,%esi
   f4b24:	shr    %edi
   f4b26:	je     f4b63 <PRIVATE000000000005b257+0x35a3>
   f4b28:	mov    %rbp,0x90(%rsp)
   f4b30:	mov    %edi,%r8d
   f4b33:	mov    0x8(%rsp),%rbp
   f4b38:	mov    (%rsi,%r10,1),%r9
   f4b3c:	inc    %rcx
   f4b3f:	mov    %r9,(%rsi,%rbp,1)
   f4b43:	mov    0x8(%rsi,%r10,1),%r9
   f4b48:	mov    %r9,0x8(%rsi,%rbp,1)
   f4b4d:	add    $0x10,%rsi
   f4b51:	cmp    %r8,%rcx
   f4b54:	jb     f4b38 <PRIVATE000000000005b257+0x3578>
   f4b56:	mov    0x90(%rsp),%rbp
   f4b5e:	lea    0x1(%rcx,%rcx,1),%r8d
   f4b63:	lea    -0x1(%r8),%ecx
   f4b67:	cmp    0x28(%rsp),%ecx
   f4b6b:	jae    f4b7f <PRIVATE000000000005b257+0x35bf>
   f4b6d:	movslq %r8d,%r8
   f4b70:	mov    0x8(%rsp),%rsi
   f4b75:	mov    -0x8(%r10,%r8,8),%rcx
   f4b7a:	mov    %rcx,-0x8(%rsi,%r8,8)
   f4b7f:	xor    %ecx,%ecx
   f4b81:	mov    $0x1,%r8d
   f4b87:	xor    %esi,%esi
   f4b89:	test   %edi,%edi
   f4b8b:	jbe    f4bbe <PRIVATE000000000005b257+0x35fe>
   f4b8d:	mov    0x8(%rsp),%r8
   f4b92:	movslq 0x0(%r13,%rcx,8),%r9
   f4b97:	mov    (%r8,%r9,8),%r9
   f4b9b:	mov    %r9,(%rsi,%r10,1)
   f4b9f:	movslq 0x4(%r13,%rcx,8),%r9
   f4ba4:	inc    %rcx
   f4ba7:	mov    (%r8,%r9,8),%r9
   f4bab:	mov    %r9,0x8(%rsi,%r10,1)
   f4bb0:	add    $0x10,%rsi
   f4bb4:	cmp    %rdi,%rcx
   f4bb7:	jb     f4b92 <PRIVATE000000000005b257+0x35d2>
   f4bb9:	lea    0x1(%rcx,%rcx,1),%r8d
   f4bbe:	lea    -0x1(%r8),%ecx
   f4bc2:	cmp    0x28(%rsp),%ecx
   f4bc6:	jae    f4bde <PRIVATE000000000005b257+0x361e>
   f4bc8:	movslq %r8d,%r8
   f4bcb:	mov    0x8(%rsp),%rcx
   f4bd0:	movslq -0x4(%r13,%r8,4),%rsi
   f4bd5:	mov    (%rcx,%rsi,8),%rdi
   f4bd9:	mov    %rdi,-0x8(%r10,%r8,8)
   f4bde:	movslq 0x2f8(%r14),%rcx
   f4be5:	test   %rcx,%rcx
   f4be8:	jge    f4c09 <PRIVATE000000000005b257+0x3649>
   f4bea:	jmp    f4c1e <PRIVATE000000000005b257+0x365e>
   f4bec:	movslq 0x2f8(%r14),%rcx
   f4bf3:	test   %rcx,%rcx
   f4bf6:	jge    f4c09 <PRIVATE000000000005b257+0x3649>
   f4bf8:	jmp    f4d81 <PRIVATE000000000005b257+0x37c1>
   f4bfd:	movslq 0x2f8(%r14),%rcx
   f4c04:	test   %rcx,%rcx
   f4c07:	jl     f4c13 <PRIVATE000000000005b257+0x3653>
   f4c09:	mov    (%rax,%rcx,4),%eax
   f4c0c:	mov    %eax,0x2f8(%r14)
   f4c13:	cmpl   $0x0,0x28(%rsp)
   f4c18:	jle    f4e48 <PRIVATE000000000005b257+0x3888>
   f4c1e:	cmpl   $0x6,0x28(%rsp)
   f4c23:	jg     f4c2f <PRIVATE000000000005b257+0x366f>
   f4c25:	movslq 0x28(%rsp),%rax
   f4c2a:	jmp    f4cf3 <PRIVATE000000000005b257+0x3733>
   f4c2f:	mov    %r11,%rcx
   f4c32:	sub    %rdx,%rcx
   f4c35:	movslq 0x28(%rsp),%rax
   f4c3a:	cmp    %rax,%rcx
   f4c3d:	jge    f4c4b <PRIVATE000000000005b257+0x368b>
   f4c3f:	neg    %rcx
   f4c42:	cmp    %rax,%rcx
   f4c45:	jl     f4cf3 <PRIVATE000000000005b257+0x3733>
   f4c4b:	cmp    $0x10,%rax
   f4c4f:	jl     f5518 <PRIVATE000000000005b257+0x3f58>
   f4c55:	mov    %r11,%rsi
   f4c58:	and    $0xf,%rsi
   f4c5c:	mov    %rsi,%rcx
   f4c5f:	neg    %rcx
   f4c62:	add    $0x10,%rcx
   f4c66:	test   %rsi,%rsi
   f4c69:	cmovne %rcx,%rsi
   f4c6d:	lea    0x10(%rsi),%rdi
   f4c71:	cmp    %rdi,%rax
   f4c74:	jl     f5518 <PRIVATE000000000005b257+0x3f58>
   f4c7a:	mov    %esi,%edi
   f4c7c:	neg    %edi
   f4c7e:	mov    0x28(%rsp),%ecx
   f4c82:	add    %ecx,%edi
   f4c84:	and    $0xf,%edi
   f4c87:	neg    %edi
   f4c89:	add    %ecx,%edi
   f4c8b:	movslq %edi,%rcx
   f4c8e:	xor    %edi,%edi
   f4c90:	test   %rsi,%rsi
   f4c93:	jbe    f4ca5 <PRIVATE000000000005b257+0x36e5>
   f4c95:	mov    (%rdi,%rdx,1),%r8b
   f4c99:	mov    %r8b,(%rdi,%r11,1)
   f4c9d:	inc    %rdi
   f4ca0:	cmp    %rsi,%rdi
   f4ca3:	jb     f4c95 <PRIVATE000000000005b257+0x36d5>
   f4ca5:	lea    (%rdx,%rsi,1),%rdi
   f4ca9:	test   $0xf,%rdi
   f4cb0:	je     f4cc8 <PRIVATE000000000005b257+0x3708>
   f4cb2:	movdqu (%rsi,%rdx,1),%xmm0
   f4cb7:	movdqu %xmm0,(%rsi,%r11,1)
   f4cbd:	add    $0x10,%rsi
   f4cc1:	cmp    %rcx,%rsi
   f4cc4:	jb     f4cb2 <PRIVATE000000000005b257+0x36f2>
   f4cc6:	jmp    f4cdc <PRIVATE000000000005b257+0x371c>
   f4cc8:	movdqu (%rsi,%rdx,1),%xmm0
   f4ccd:	movdqu %xmm0,(%rsi,%r11,1)
   f4cd3:	add    $0x10,%rsi
   f4cd7:	cmp    %rcx,%rsi
   f4cda:	jb     f4cc8 <PRIVATE000000000005b257+0x3708>
   f4cdc:	cmp    %rax,%rcx
   f4cdf:	jae    f4d35 <PRIVATE000000000005b257+0x3775>
   f4ce1:	mov    (%rcx,%rdx,1),%sil
   f4ce5:	mov    %sil,(%rcx,%r11,1)
   f4ce9:	inc    %rcx
   f4cec:	cmp    %rax,%rcx
   f4cef:	jb     f4ce1 <PRIVATE000000000005b257+0x3721>
   f4cf1:	jmp    f4d35 <PRIVATE000000000005b257+0x3775>
   f4cf3:	mov    0x28(%rsp),%ecx
   f4cf7:	mov    $0x1,%edi
   f4cfc:	xor    %esi,%esi
   f4cfe:	shr    %ecx
   f4d00:	je     f4d20 <PRIVATE000000000005b257+0x3760>
   f4d02:	mov    (%rdx,%rsi,2),%dil
   f4d06:	mov    %dil,(%r11,%rsi,2)
   f4d0a:	mov    0x1(%rdx,%rsi,2),%r8b
   f4d0f:	mov    %r8b,0x1(%r11,%rsi,2)
   f4d14:	inc    %rsi
   f4d17:	cmp    %rcx,%rsi
   f4d1a:	jb     f4d02 <PRIVATE000000000005b257+0x3742>
   f4d1c:	lea    0x1(%rsi,%rsi,1),%edi
   f4d20:	lea    -0x1(%rdi),%ecx
   f4d23:	cmp    0x28(%rsp),%ecx
   f4d27:	jae    f4d35 <PRIVATE000000000005b257+0x3775>
   f4d29:	movslq %edi,%rdi
   f4d2c:	mov    -0x1(%rdi,%rdx,1),%cl
   f4d30:	mov    %cl,-0x1(%rdi,%r11,1)
   f4d35:	mov    %rbp,0x90(%rsp)
   f4d3d:	xor    %edi,%edi
   f4d3f:	mov    0x8(%rsp),%rsi
   f4d44:	mov    0x10(%rsp),%rcx
   f4d49:	mov    0x20(%rsp),%rbp
   f4d4e:	mov    0x30(%rsp),%r8
   f4d53:	movslq 0x0(%r13,%rdi,4),%r9
   f4d58:	mov    (%r8,%r9,4),%r10d
   f4d5c:	mov    %r10d,(%rcx,%rdi,4)
   f4d60:	mov    0x0(%rbp,%r9,8),%r10
   f4d65:	mov    %r10,(%rsi,%rdi,8)
   f4d69:	mov    (%r9,%r11,1),%r9b
   f4d6d:	mov    %r9b,(%rdi,%rdx,1)
   f4d71:	inc    %rdi
   f4d74:	cmp    %rax,%rdi
   f4d77:	jl     f4d53 <PRIVATE000000000005b257+0x3793>
   f4d79:	mov    0x90(%rsp),%rbp
   f4d81:	cmpl   $0x0,0x3f0(%r14)
   f4d89:	jle    f5e68 <PRIVATE000000000005b257+0x48a8>
   f4d8f:	mov    0x3b8(%r15),%rsi
   f4d96:	test   %rsi,%rsi
   f4d99:	je     f5e68 <PRIVATE000000000005b257+0x48a8>
   f4d9f:	cmpl   $0x0,0x28(%rsp)
   f4da4:	jle    f5e83 <PRIVATE000000000005b257+0x48c3>
   f4daa:	cmp    0x20(%rsp),%rsi
   f4daf:	je     f4dcb <PRIVATE000000000005b257+0x380b>
   f4db1:	movslq 0x28(%rsp),%rdx
   f4db6:	shl    $0x3,%rdx
   f4dba:	mov    0x20(%rsp),%rdi
   f4dbf:	call   bcab0 <memcpy@plt>
   f4dc4:	mov    0x3b8(%r15),%rsi
   f4dcb:	xor    %r11d,%r11d
   f4dce:	mov    $0x1,%ecx
   f4dd3:	mov    0x28(%rsp),%edx
   f4dd7:	xor    %eax,%eax
   f4dd9:	shr    %edx
   f4ddb:	je     f4e0e <PRIVATE000000000005b257+0x384e>
   f4ddd:	mov    0x20(%rsp),%r10
   f4de2:	movslq 0x0(%r13,%r11,8),%rcx
   f4de7:	movslq 0x4(%r13,%r11,8),%r8
   f4dec:	inc    %r11
   f4def:	mov    (%r10,%rcx,8),%rdi
   f4df3:	mov    %rdi,(%rax,%rsi,1)
   f4df7:	mov    (%r10,%r8,8),%r9
   f4dfb:	mov    %r9,0x8(%rax,%rsi,1)
   f4e00:	add    $0x10,%rax
   f4e04:	cmp    %rdx,%r11
   f4e07:	jb     f4de2 <PRIVATE000000000005b257+0x3822>
   f4e09:	lea    0x1(%r11,%r11,1),%ecx
   f4e0e:	lea    -0x1(%rcx),%eax
   f4e11:	cmp    0x28(%rsp),%eax
   f4e15:	jae    f5e68 <PRIVATE000000000005b257+0x48a8>
   f4e1b:	movslq %ecx,%rcx
   f4e1e:	cmpl   $0x0,0x28(%rsp)
   f4e23:	movslq -0x4(%r13,%rcx,4),%rax
   f4e28:	mov    0x20(%rsp),%r13
   f4e2d:	mov    0x0(%r13,%rax,8),%r11
   f4e32:	mov    0x38(%rsp),%rax
   f4e37:	mov    %r11,-0x8(%rsi,%rcx,8)
   f4e3c:	mov    0x1c8(%rax),%rax
   f4e43:	mov    %rax,0x18(%rsp)
   f4e48:	mov    0x18(%rsp),%r11
   f4e4d:	mov    0x30(%rsp),%rax
   f4e52:	mov    0x20(%rsp),%r13
   f4e57:	mov    %rax,0x8(%r11)
   f4e5b:	mov    0x8(%rsp),%rax
   f4e60:	mov    %rax,0x48(%r15)
   f4e64:	mov    0x38(%rsp),%rax
   f4e69:	mov    %r13,0x38(%r15)
   f4e6d:	mov    0x10(%rsp),%r15
   f4e72:	mov    %r15,0x18(%rax)
   f4e76:	jle    f5e53 <PRIVATE000000000005b257+0x4893>
   f4e7c:	mov    0x28(%rsp),%eax
   f4e80:	mov    $0x1,%r13d
   f4e86:	xor    %r11d,%r11d
   f4e89:	shr    %eax
   f4e8b:	je     f4eba <PRIVATE000000000005b257+0x38fa>
   f4e8d:	mov    %r15,%rdx
   f4e90:	movslq %r11d,%r11
   f4e93:	lea    (%r11,%r11,1),%r13d
   f4e97:	movslq (%rdx,%r11,8),%r15
   f4e9b:	mov    %r13d,(%r12,%r15,4)
   f4e9f:	lea    0x1(%r11,%r11,1),%r13d
   f4ea4:	movslq 0x4(%rdx,%r11,8),%r15
   f4ea9:	inc    %r11d
   f4eac:	mov    %r13d,(%r12,%r15,4)
   f4eb0:	cmp    %eax,%r11d
   f4eb3:	jb     f4e90 <PRIVATE000000000005b257+0x38d0>
   f4eb5:	lea    0x1(%r11,%r11,1),%r13d
   f4eba:	lea    -0x1(%r13),%r11d
   f4ebe:	cmp    0x28(%rsp),%r11d
   f4ec3:	jae    f5e53 <PRIVATE000000000005b257+0x4893>
   f4ec9:	movslq %r13d,%r13
   f4ecc:	mov    0x10(%rsp),%rax
   f4ed1:	cmpl   $0x0,0xf0(%rsp)
   f4ed9:	movslq -0x4(%rax,%r13,4),%r15
   f4ede:	mov    %r11d,(%r12,%r15,4)
   f4ee2:	mov    0x28(%rbx),%r13
   f4ee6:	mov    $0x0,%r8d
   f4eec:	mov    0x38(%rbx),%r12
   f4ef0:	mov    %r13,0x40(%rbx)
   f4ef4:	mov    %r12,0x48(%rbx)
   f4ef8:	jle    f4f5e <PRIVATE000000000005b257+0x399e>
   f4efa:	mov    0xb0(%rbx),%r11
   f4f01:	xor    %r8d,%r8d
   f4f04:	mov    0x88(%rbx),%rdi
   f4f0b:	mov    0x90(%rbx),%rdx
   f4f12:	mov    0xb8(%rbx),%rcx
   f4f19:	mov    0xc0(%rbx),%rsi
   f4f20:	mov    0x1a0(%rbx),%r15
   f4f27:	movslq 0xf0(%rsp),%rax
   f4f2f:	movl   $0x77359400,(%r15,%r8,4)
   f4f37:	mov    %r8d,0x0(%r13,%r8,4)
   f4f3c:	mov    %r8d,(%r12,%r8,4)
   f4f40:	mov    0x8(%rdi,%r8,8),%r9
   f4f45:	mov    %r9,(%rdx,%r8,8)
   f4f49:	mov    0x8(%r11,%r8,8),%r10
   f4f4e:	mov    %r10,(%rcx,%r8,8)
   f4f52:	mov    %r10,(%rsi,%r8,8)
   f4f56:	inc    %r8
   f4f59:	cmp    %rax,%r8
   f4f5c:	jb     f4f2f <PRIVATE000000000005b257+0x396f>
   f4f5e:	xor    %r11d,%r11d
   f4f61:	test   %rbp,%rbp
   f4f64:	je     f55ac <PRIVATE000000000005b257+0x3fec>
   f4f6a:	pxor   %xmm2,%xmm2
   f4f6e:	cvtsi2sd %r8d,%xmm2
   f4f73:	mulsd  0x2968e75(%rip),%xmm2        # 2a5ddf0 <_fini+0x11ec>
   f4f7b:	movsd  0x148(%r14),%xmm1
   f4f84:	mulsd  %xmm1,%xmm2
   f4f88:	cmpl   $0x0,0xdc(%rbx)
   f4f8f:	addsd  0x0(%rbp),%xmm2
   f4f94:	jle    f4fd5 <PRIVATE000000000005b257+0x3a15>
   f4f96:	movsd  %xmm2,0x0(%rbp)
