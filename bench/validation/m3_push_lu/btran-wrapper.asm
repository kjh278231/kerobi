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
