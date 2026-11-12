# 05b257 entry (sliced from objdump of 05b257 starting at 0xf15c0)
   f15c0:	push   %r12
   f15c2:	push   %r13
   f15c4:	push   %r14
   f15c6:	push   %r15
   f15c8:	push   %rbx
   f15c9:	push   %rbp
   f15ca:	sub    $0x178,%rsp
   f15d1:	mov    %rdi,%r14
   f15d4:	mov    %rsi,%rbp
   f15d7:	mov    0x470(%r14),%rbx
   f15de:	mov    0x478(%r14),%rax
   f15e5:	mov    0x480(%r14),%r11
   f15ec:	mov    %rax,0xf8(%rsp)
   f15f4:	lea    0x20(%rbx),%rax
   f15f8:	mov    %rax,0x100(%rsp)

# 09e47c call (sliced from objdump of 05b257 starting at 0xf15c0)
   f1933:	movl   $0x0,(%rdi)
   f1939:	jmp    f1c01 <PRIVATE000000000005b257+0x641>
   f193e:	cmp    0x1d8(%rbx),%r8d
   f1945:	je     f197c <PRIVATE000000000005b257+0x3bc>
   f1947:	mov    %rbx,%rdi
   f194a:	mov    %r8d,%edx
   f194d:	mov    %rbp,%rcx
   f1950:	mov    0x220(%r14),%rsi
   f1957:	mov    %eax,(%rsp)
   f195a:	mov    %r11d,0x38(%rsp)
   f195f:	mov    %r8d,0x130(%rsp)
   f1967:	call   131760 <PRIVATE000000000009e47c>
   f196c:	mov    0x130(%rsp),%r8d
   f1974:	mov    0x38(%rsp),%r11d

# 036ade call (sliced from objdump of 05b257 starting at 0xf15c0)
   f3446:	mov    %r13,%r8
   f3449:	movsd  %xmm1,0xdd0(%r14)
   f3452:	movsd  0x390(%r14),%xmm1
   f345b:	mov    %r11,(%rsp)
   f345f:	mov    0x108(%rsp),%r11
   f3467:	mov    %r11,0x8(%rsp)
   f346c:	mov    0xf8(%rsp),%r11
   f3474:	mov    %r11,0x10(%rsp)
   f3479:	mov    0x160(%rsp),%r11
   f3481:	mov    %r11,0x18(%rsp)
   f3486:	lea    0xe8(%rsp),%r11
   f348e:	mov    %r11,0x20(%rsp)
   f3493:	lea    0xec(%rsp),%r11
   f349b:	mov    %r11,0x28(%rsp)
   f34a0:	mov    0x1b0(%rbx),%r11
   f34a7:	mov    %r12,0x30(%rsp)
   f34ac:	mov    %r15,0x38(%rsp)
   f34b1:	mov    %r11,0x40(%rsp)
   f34b6:	mov    0x1b8(%rbx),%r11
   f34bd:	mov    %r11,0x48(%rsp)
   f34c2:	mov    %rbp,0x50(%rsp)
   f34c7:	movsd  0xe0(%rsp),%xmm0
   f34d0:	mov    0x150(%rsp),%edx
   f34d7:	mov    0x120(%rsp),%ecx
   f34de:	mov    0x118(%rsp),%r9
   f34e6:	mov    0x4dc(%r14),%esi
   f34ed:	call   d4190 <PRIVATE0000000000036ade>
   f34f2:	add    $0x60,%rsp
   f34f6:	mov    0x8c(%rsp),%r11d
   f34fe:	test   %r11d,%r11d
   f3501:	jne    f5e28 <PRIVATE000000000005b257+0x4868>
   f3507:	cmpl   $0x1,0xf0(%r14)
   f350f:	jne    f2eb3 <PRIVATE000000000005b257+0x18f3>
   f3515:	test   %eax,%eax
   f3517:	jne    f3654 <PRIVATE000000000005b257+0x2094>
   f351d:	mov    0x88(%rsp),%edx
   f3524:	test   %edx,%edx
   f3526:	jne    f363e <PRIVATE000000000005b257+0x207e>
   f352c:	movsd  0x390(%r14),%xmm3

# identity rewrite (sliced from objdump of 05b257 starting at 0xf15c0)
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
# 036ade entry
   d4190:	push   %r12
   d4192:	push   %r13
   d4194:	push   %r14
   d4196:	push   %r15
   d4198:	push   %rbx
   d4199:	push   %rbp
   d419a:	sub    $0x308,%rsp
   d41a1:	mov    %rdi,%rbp
   d41a4:	movsd  %xmm1,0x2e0(%rsp)
   d41ad:	mov    %r9,%r15

# 09e47c entry
  131760:	push   %r12
  131762:	push   %r13
  131764:	push   %r14
  131766:	push   %r15
  131768:	push   %rbx
  131769:	push   %rbp
  13176a:	sub    $0xa8,%rsp
  131771:	mov    %rdi,%r15
  131774:	movslq %edx,%rdx
  131777:	mov    0xb0(%r15),%r14
  13177e:	rex.WRB
  13177f:	.byte 0x8b
