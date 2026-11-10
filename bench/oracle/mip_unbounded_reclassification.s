libgurobi130.so:     file format elf64-x86-64


Disassembly of section .text:

00000000004f1a98 <PRIVATE00000000005ce67b+0xc88>:
  4f1a98:	45 8b 8c 24 a8 00 00 	mov    r9d,DWORD PTR [r12+0xa8]
  4f1a9f:	00
  4f1aa0:	41 83 f9 ff          	cmp    r9d,0xffffffff
  4f1aa4:	0f 84 fd 44 00 00    	je     4f5fa7 <PRIVATE00000000005ce67b+0x5197>
  4f1aaa:	41 83 f9 02          	cmp    r9d,0x2
  4f1aae:	0f 84 c7 44 00 00    	je     4f5f7b <PRIVATE00000000005ce67b+0x516b>
  4f1ab4:	48 8b 8c 24 b0 03 00 	mov    rcx,QWORD PTR [rsp+0x3b0]
  4f1abb:	00
  4f1abc:	49 3b 8c 24 90 07 00 	cmp    rcx,QWORD PTR [r12+0x790]
  4f1ac3:	00
  4f1ac4:	0f 84 48 44 00 00    	je     4f5f12 <PRIVATE00000000005ce67b+0x5102>
  4f1aca:	45 85 f6             	test   r14d,r14d
  4f1acd:	0f 85 89 0a 00 00    	jne    4f255c <PRIVATE00000000005ce67b+0x174c>
  4f1ad3:	41 8b 4d 38          	mov    ecx,DWORD PTR [r13+0x38]
  4f1ad7:	83 f9 0c             	cmp    ecx,0xc
  4f1ada:	0f 84 08 44 00 00    	je     4f5ee8 <PRIVATE00000000005ce67b+0x50d8>
  4f1ae0:	48 8b 84 24 b0 03 00 	mov    rax,QWORD PTR [rsp+0x3b0]
  4f1ae7:	00
  4f1ae8:	49 3b 84 24 90 07 00 	cmp    rax,QWORD PTR [r12+0x790]
  4f1aef:	00
  4f1af0:	0f 84 13 38 00 00    	je     4f5309 <PRIVATE00000000005ce67b+0x44f9>
  4f1af6:	8a 84 24 78 04 00 00 	mov    al,BYTE PTR [rsp+0x478]
  4f1afd:	66 0f ef c0          	pxor   xmm0,xmm0
  4f1b01:	f3 41 0f 7f 85 80 00 	movdqu XMMWORD PTR [r13+0x80],xmm0
  4f1b08:	00 00
  4f1b0a:	84 c0                	test   al,al
  4f1b0c:	75 16                	jne    4f1b24 <PRIVATE00000000005ce67b+0xd14>
  4f1b0e:	48 8b 84 24 b0 03 00 	mov    rax,QWORD PTR [rsp+0x3b0]
  4f1b15:	00
  4f1b16:	49 3b 84 24 90 07 00 	cmp    rax,QWORD PTR [r12+0x790]
  4f1b1d:	00
  4f1b1e:	0f 84 80 33 00 00    	je     4f4ea4 <PRIVATE00000000005ce67b+0x4094>
  4f1b24:	48 8b 84 24 b0 03 00 	mov    rax,QWORD PTR [rsp+0x3b0]
  4f1b2b:	00
  4f1b2c:	48 8b 9c 24 68 03 00 	mov    rbx,QWORD PTR [rsp+0x368]
  4f1b33:	00
  4f1b34:	48 8b 08             	mov    rcx,QWORD PTR [rax]
  4f1b37:	41 8b 45 38          	mov    eax,DWORD PTR [r13+0x38]
  4f1b3b:	48 89 4b 28          	mov    QWORD PTR [rbx+0x28],rcx
  4f1b3f:	83 f8 11             	cmp    eax,0x11
  4f1b42:	0f 84 14 33 00 00    	je     4f4e5c <PRIVATE00000000005ce67b+0x404c>

libgurobi130.so:     file format elf64-x86-64


Disassembly of section .text:

00000000004f53a8 <PRIVATE00000000005ce67b+0x4598>:
  4f53a8:	48 8b 8c 24 90 04 00 	mov    rcx,QWORD PTR [rsp+0x490]
  4f53af:	00
  4f53b0:	83 b9 b0 28 00 00 fe 	cmp    DWORD PTR [rcx+0x28b0],0xfffffffe
  4f53b7:	0f 84 e5 0a 00 00    	je     4f5ea2 <PRIVATE00000000005ce67b+0x5092>
  4f53bd:	48 8b 8c 24 80 04 00 	mov    rcx,QWORD PTR [rsp+0x480]
  4f53c4:	00
  4f53c5:	48 83 b9 a0 01 00 00 	cmp    QWORD PTR [rcx+0x1a0],0x0
  4f53cc:	00
  4f53cd:	0f 85 cf 0a 00 00    	jne    4f5ea2 <PRIVATE00000000005ce67b+0x5092>
  4f53d3:	48 8b 84 24 98 04 00 	mov    rax,QWORD PTR [rsp+0x498]
  4f53da:	00
  4f53db:	f2 0f 10 05 95 6a 57 	movsd  xmm0,QWORD PTR [rip+0x2576a95]        # 2a6be78 <coefclasssign+0x16f8>
  4f53e2:	02
  4f53e3:	f2 0f 11 84 24 20 04 	movsd  QWORD PTR [rsp+0x420],xmm0
  4f53ea:	00 00
  4f53ec:	48 83 b8 b0 00 00 00 	cmp    QWORD PTR [rax+0xb0],0x0
  4f53f3:	00
  4f53f4:	74 35                	je     4f542b <PRIVATE00000000005ce67b+0x461b>
  4f53f6:	48 89 ef             	mov    rdi,rbp
  4f53f9:	e8 52 8a 39 00       	call   88de50 <PRIVATE0000000000a7f923>
  4f53fe:	85 c0                	test   eax,eax
  4f5400:	75 29                	jne    4f542b <PRIVATE00000000005ce67b+0x461b>
  4f5402:	48 8b bc 24 b0 03 00 	mov    rdi,QWORD PTR [rsp+0x3b0]
  4f5409:	00
  4f540a:	48 8b b4 24 68 03 00 	mov    rsi,QWORD PTR [rsp+0x368]
  4f5411:	00
  4f5412:	48 8b 94 24 70 03 00 	mov    rdx,QWORD PTR [rsp+0x370]
  4f5419:	00
  4f541a:	e8 f1 41 08 00       	call   579610 <PRIVATE000000000069d7ea>
  4f541f:	41 89 c6             	mov    r14d,eax
  4f5422:	45 85 f6             	test   r14d,r14d
  4f5425:	0f 85 31 d1 ff ff    	jne    4f255c <PRIVATE00000000005ce67b+0x174c>
  4f542b:	48 8b 84 24 98 04 00 	mov    rax,QWORD PTR [rsp+0x498]
  4f5432:	00
  4f5433:	48 83 b8 98 00 00 00 	cmp    QWORD PTR [rax+0x98],0x0
  4f543a:	00
  4f543b:	74 0e                	je     4f544b <PRIVATE00000000005ce67b+0x463b>
  4f543d:	48 83 b8 a8 00 00 00 	cmp    QWORD PTR [rax+0xa8],0x0
  4f5444:	00
  4f5445:	0f 84 fa 05 00 00    	je     4f5a45 <PRIVATE00000000005ce67b+0x4c35>
  4f544b:	48 89 ef             	mov    rdi,rbp
  4f544e:	e8 fd 89 39 00       	call   88de50 <PRIVATE0000000000a7f923>
  4f5453:	85 c0                	test   eax,eax
  4f5455:	0f 85 47 02 00 00    	jne    4f56a2 <PRIVATE00000000005ce67b+0x4892>
  4f545b:	83 bc 24 c8 03 00 00 	cmp    DWORD PTR [rsp+0x3c8],0x2
  4f5462:	02
  4f5463:	0f 84 c2 05 00 00    	je     4f5a2b <PRIVATE00000000005ce67b+0x4c1b>
  4f5469:	48 8b 84 24 b0 03 00 	mov    rax,QWORD PTR [rsp+0x3b0]
  4f5470:	00
  4f5471:	49 3b 84 24 90 07 00 	cmp    rax,QWORD PTR [r12+0x790]
  4f5478:	00
  4f5479:	0f 84 94 05 00 00    	je     4f5a13 <PRIVATE00000000005ce67b+0x4c03>
  4f547f:	83 bc 24 c8 03 00 00 	cmp    DWORD PTR [rsp+0x3c8],0x1
  4f5486:	01
  4f5487:	74 23                	je     4f54ac <PRIVATE00000000005ce67b+0x469c>
  4f5489:	83 bc 24 c8 03 00 00 	cmp    DWORD PTR [rsp+0x3c8],0x0
  4f5490:	00
  4f5491:	74 19                	je     4f54ac <PRIVATE00000000005ce67b+0x469c>
  4f5493:	8b 8c 24 c8 03 00 00 	mov    ecx,DWORD PTR [rsp+0x3c8]
  4f549a:	b8 01 00 00 00       	mov    eax,0x1
  4f549f:	83 f9 02             	cmp    ecx,0x2
  4f54a2:	0f 45 c8             	cmovne ecx,eax
  4f54a5:	89 8c 24 c8 03 00 00 	mov    DWORD PTR [rsp+0x3c8],ecx
  4f54ac:	48 8b 85 00 01 00 00 	mov    rax,QWORD PTR [rbp+0x100]
  4f54b3:	48 8d bd e0 00 00 00 	lea    rdi,[rbp+0xe0]
  4f54ba:	c7 80 24 24 00 00 01 	mov    DWORD PTR [rax+0x2424],0x1
  4f54c1:	00 00 00
  4f54c4:	e8 97 b1 c1 ff       	call   110660 <PRIVATE0000000000083696>
  4f54c9:	48 8b 85 00 01 00 00 	mov    rax,QWORD PTR [rbp+0x100]
  4f54d0:	8b 8c 24 c8 03 00 00 	mov    ecx,DWORD PTR [rsp+0x3c8]
  4f54d7:	44 8b 8c 24 c0 03 00 	mov    r9d,DWORD PTR [rsp+0x3c0]
  4f54de:	00
  4f54df:	c7 80 08 27 00 00 01 	mov    DWORD PTR [rax+0x2708],0x1
  4f54e6:	00 00 00
  4f54e9:	48 8b 9d 00 01 00 00 	mov    rbx,QWORD PTR [rbp+0x100]
  4f54f0:	89 8b 38 24 00 00    	mov    DWORD PTR [rbx+0x2438],ecx
  4f54f6:	4c 8b 95 00 01 00 00 	mov    r10,QWORD PTR [rbp+0x100]
  4f54fd:	45 89 8a 3c 23 00 00 	mov    DWORD PTR [r10+0x233c],r9d
  4f5504:	48 8b 8d 00 01 00 00 	mov    rcx,QWORD PTR [rbp+0x100]
  4f550b:	8b 81 78 25 00 00    	mov    eax,DWORD PTR [rcx+0x2578]
  4f5511:	85 c0                	test   eax,eax
  4f5513:	7c 06                	jl     4f551b <PRIVATE00000000005ce67b+0x470b>
  4f5515:	89 81 24 24 00 00    	mov    DWORD PTR [rcx+0x2424],eax
  4f551b:	48 89 ef             	mov    rdi,rbp
  4f551e:	48 8b b4 24 70 03 00 	mov    rsi,QWORD PTR [rsp+0x370]
  4f5525:	00
  4f5526:	e8 85 57 c5 ff       	call   14acb0 <PRIVATE00000000000c3d0d>
  4f552b:	41 89 c6             	mov    r14d,eax
  4f552e:	48 8b 9d 00 01 00 00 	mov    rbx,QWORD PTR [rbp+0x100]
  4f5535:	8b 8c 24 60 04 00 00 	mov    ecx,DWORD PTR [rsp+0x460]
  4f553c:	44 8b 8c 24 38 04 00 	mov    r9d,DWORD PTR [rsp+0x438]
  4f5543:	00
  4f5544:	89 8b 38 24 00 00    	mov    DWORD PTR [rbx+0x2438],ecx
  4f554a:	4c 8b 95 00 01 00 00 	mov    r10,QWORD PTR [rbp+0x100]
  4f5551:	45 89 8a 3c 23 00 00 	mov    DWORD PTR [r10+0x233c],r9d
  4f5558:	48 85 ed             	test   rbp,rbp
  4f555b:	74 2e                	je     4f558b <PRIVATE00000000005ce67b+0x477b>
  4f555d:	48 8b 9d 00 01 00 00 	mov    rbx,QWORD PTR [rbp+0x100]
  4f5564:	48 85 db             	test   rbx,rbx
  4f5567:	74 22                	je     4f558b <PRIVATE00000000005ce67b+0x477b>
  4f5569:	8b 8c 24 40 04 00 00 	mov    ecx,DWORD PTR [rsp+0x440]
  4f5570:	89 8b 24 24 00 00    	mov    DWORD PTR [rbx+0x2424],ecx
  4f5576:	4c 8b 8d 00 01 00 00 	mov    r9,QWORD PTR [rbp+0x100]
  4f557d:	8b 9c 24 68 04 00 00 	mov    ebx,DWORD PTR [rsp+0x468]
  4f5584:	41 89 99 08 27 00 00 	mov    DWORD PTR [r9+0x2708],ebx
  4f558b:	45 85 f6             	test   r14d,r14d
  4f558e:	74 14                	je     4f55a4 <PRIVATE00000000005ce67b+0x4794>
  4f5590:	41 81 fe 24 27 00 00 	cmp    r14d,0x2724
  4f5597:	0f 85 bf cf ff ff    	jne    4f255c <PRIVATE00000000005ce67b+0x174c>
  4f559d:	c7 45 64 0c 00 00 00 	mov    DWORD PTR [rbp+0x64],0xc
  4f55a4:	4d 8b 7c 24 08       	mov    r15,QWORD PTR [r12+0x8]
  4f55a9:	48 89 ef             	mov    rdi,rbp
  4f55ac:	f2 0f 10 84 24 20 04 	movsd  xmm0,QWORD PTR [rsp+0x420]
  4f55b3:	00 00
  4f55b5:	48 8d 35 a0 86 59 02 	lea    rsi,[rip+0x25986a0]        # 2a8dc5c <lazyval.10677.0.41+0x4ec4>
  4f55bc:	48 8b 94 24 98 03 00 	mov    rdx,QWORD PTR [rsp+0x398]
  4f55c3:	00
  4f55c4:	49 8b 9f e8 00 00 00 	mov    rbx,QWORD PTR [r15+0xe8]
  4f55cb:	f2 0f 11 84 24 28 03 	movsd  QWORD PTR [rsp+0x328],xmm0
  4f55d2:	00 00
  4f55d4:	8b 83 58 02 00 00    	mov    eax,DWORD PTR [rbx+0x258]
  4f55da:	8b 8b a8 01 00 00    	mov    ecx,DWORD PTR [rbx+0x1a8]
  4f55e0:	44 8b 7b 18          	mov    r15d,DWORD PTR [rbx+0x18]
  4f55e4:	8b 5b 1c             	mov    ebx,DWORD PTR [rbx+0x1c]
  4f55e7:	89 84 24 20 03 00 00 	mov    DWORD PTR [rsp+0x320],eax
  4f55ee:	89 8c 24 18 03 00 00 	mov    DWORD PTR [rsp+0x318],ecx
  4f55f5:	e8 c6 79 bc ff       	call   bcfc0 <GRBgetintattr@plt>
  4f55fa:	41 89 c6             	mov    r14d,eax
  4f55fd:	45 85 f6             	test   r14d,r14d
  4f5600:	0f 85 93 00 00 00    	jne    4f5699 <PRIVATE00000000005ce67b+0x4889>
  4f5606:	45 8b 4d 38          	mov    r9d,DWORD PTR [r13+0x38]
  4f560a:	44 89 c9             	mov    ecx,r9d
  4f560d:	83 e1 fb             	and    ecx,0xfffffffb
  4f5610:	83 f9 02             	cmp    ecx,0x2
  4f5613:	0f 84 97 03 00 00    	je     4f59b0 <PRIVATE00000000005ce67b+0x4ba0>
  4f5619:	48 b9 7d c3 94 25 ad 	movabs rcx,0xd4b249ad2594c37d
  4f5620:	49 b2 d4
  4f5623:	48 89 8c 24 10 03 00 	mov    QWORD PTR [rsp+0x310],rcx
  4f562a:	00
  4f562b:	4c 89 e7             	mov    rdi,r12
  4f562e:	e8 5d 46 01 00       	call   509c90 <PRIVATE00000000005e7204>
  4f5633:	0f 28 c8             	movaps xmm1,xmm0
  4f5636:	45 8b 4d 38          	mov    r9d,DWORD PTR [r13+0x38]
  4f563a:	41 83 f9 02          	cmp    r9d,0x2
  4f563e:	0f 84 3d 03 00 00    	je     4f5981 <PRIVATE00000000005ce67b+0x4b71>
  4f5644:	48 8b 8d 00 01 00 00 	mov    rcx,QWORD PTR [rbp+0x100]
  4f564b:	f2 0f 10 89 a8 24 00 	movsd  xmm1,QWORD PTR [rcx+0x24a8]
  4f5652:	00
  4f5653:	41 83 f9 06          	cmp    r9d,0x6
  4f5657:	0f 84 c7 02 00 00    	je     4f5924 <PRIVATE00000000005ce67b+0x4b14>
  4f565d:	41 83 f9 04          	cmp    r9d,0x4
  4f5661:	0f 84 00 02 00 00    	je     4f5867 <PRIVATE00000000005ce67b+0x4a57>
  4f5667:	45 8b 8c 24 a8 00 00 	mov    r9d,DWORD PTR [r12+0xa8]
  4f566e:	00
  4f566f:	41 83 f9 ff          	cmp    r9d,0xffffffff
  4f5673:	0f 84 89 01 00 00    	je     4f5802 <PRIVATE00000000005ce67b+0x49f2>
  4f5679:	41 83 f9 02          	cmp    r9d,0x2
  4f567d:	0f 84 87 03 00 00    	je     4f5a0a <PRIVATE00000000005ce67b+0x4bfa>
  4f5683:	48 8b 8c 24 b0 03 00 	mov    rcx,QWORD PTR [rsp+0x3b0]
  4f568a:	00
  4f568b:	49 3b 8c 24 90 07 00 	cmp    rcx,QWORD PTR [r12+0x790]
  4f5692:	00
  4f5693:	0f 84 00 01 00 00    	je     4f5799 <PRIVATE00000000005ce67b+0x4989>
  4f5699:	45 85 f6             	test   r14d,r14d
  4f569c:	0f 85 ba ce ff ff    	jne    4f255c <PRIVATE00000000005ce67b+0x174c>

libgurobi130.so:     file format elf64-x86-64


Disassembly of section .text:

00000000004f5a13 <PRIVATE00000000005ce67b+0x4c03>:
  4f5a13:	41 83 3c 24 05       	cmp    DWORD PTR [r12],0x5
  4f5a18:	0f 85 61 fa ff ff    	jne    4f547f <PRIVATE00000000005ce67b+0x466f>
  4f5a1e:	41 c7 04 24 01 00 00 	mov    DWORD PTR [r12],0x1
  4f5a25:	00
  4f5a26:	e9 54 fa ff ff       	jmp    4f547f <PRIVATE00000000005ce67b+0x466f>
  4f5a2b:	48 8b bc 24 80 04 00 	mov    rdi,QWORD PTR [rsp+0x480]
  4f5a32:	00
  4f5a33:	e8 a8 6b de ff       	call   2dc5e0 <PRIVATE00000000002baa47>
  4f5a38:	85 c0                	test   eax,eax
  4f5a3a:	0f 84 62 fc ff ff    	je     4f56a2 <PRIVATE00000000005ce67b+0x4892>
  4f5a40:	e9 24 fa ff ff       	jmp    4f5469 <PRIVATE00000000005ce67b+0x4659>

libgurobi130.so:     file format elf64-x86-64


Disassembly of section .text:

00000000004f5ea2 <PRIVATE00000000005ce67b+0x5092>:
  4f5ea2:	4c 8b bc 24 60 03 00 	mov    r15,QWORD PTR [rsp+0x360]
  4f5ea9:	00
  4f5eaa:	44 89 f2             	mov    edx,r14d
  4f5ead:	4c 8b b4 24 70 03 00 	mov    r14,QWORD PTR [rsp+0x370]
  4f5eb4:	00
  4f5eb5:	49 8b 9c 24 20 06 00 	mov    rbx,QWORD PTR [r12+0x620]
  4f5ebc:	00
  4f5ebd:	be 03 00 00 00       	mov    esi,0x3
  4f5ec2:	48 8b bc 24 b0 03 00 	mov    rdi,QWORD PTR [rsp+0x3b0]
  4f5ec9:	00
  4f5eca:	89 94 24 18 04 00 00 	mov    DWORD PTR [rsp+0x418],edx
  4f5ed1:	c7 03 0d 00 00 00    	mov    DWORD PTR [rbx],0xd
  4f5ed7:	e8 14 22 02 00       	call   5180f0 <PRIVATE00000000005ff754>
  4f5edc:	8b 94 24 18 04 00 00 	mov    edx,DWORD PTR [rsp+0x418]
  4f5ee3:	e9 8b c0 ff ff       	jmp    4f1f73 <PRIVATE00000000005ce67b+0x1163>
  4f5ee8:	41 83 bc 24 a8 00 00 	cmp    DWORD PTR [r12+0xa8],0x2
  4f5eef:	00 02
  4f5ef1:	0f 84 b1 f4 ff ff    	je     4f53a8 <PRIVATE00000000005ce67b+0x4598>
  4f5ef7:	48 8b 8c 24 b0 03 00 	mov    rcx,QWORD PTR [rsp+0x3b0]
  4f5efe:	00
  4f5eff:	49 3b 8c 24 90 07 00 	cmp    rcx,QWORD PTR [r12+0x790]
  4f5f06:	00
  4f5f07:	0f 85 e9 bb ff ff    	jne    4f1af6 <PRIVATE00000000005ce67b+0xce6>
  4f5f0d:	e9 96 f4 ff ff       	jmp    4f53a8 <PRIVATE00000000005ce67b+0x4598>
  4f5f12:	41 8b 4d 38          	mov    ecx,DWORD PTR [r13+0x38]
  4f5f16:	83 f9 02             	cmp    ecx,0x2
  4f5f19:	75 11                	jne    4f5f2c <PRIVATE00000000005ce67b+0x511c>
  4f5f1b:	41 c7 84 24 a8 00 00 	mov    DWORD PTR [r12+0xa8],0x2
  4f5f22:	00 02 00 00 00
  4f5f27:	e9 9e bb ff ff       	jmp    4f1aca <PRIVATE00000000005ce67b+0xcba>
  4f5f2c:	83 f9 05             	cmp    ecx,0x5
  4f5f2f:	0f 85 95 bb ff ff    	jne    4f1aca <PRIVATE00000000005ce67b+0xcba>
  4f5f35:	41 c7 84 24 a8 00 00 	mov    DWORD PTR [r12+0xa8],0x5
  4f5f3c:	00 05 00 00 00
  4f5f41:	45 85 ff             	test   r15d,r15d
  4f5f44:	0f 85 80 bb ff ff    	jne    4f1aca <PRIVATE00000000005ce67b+0xcba>
  4f5f4a:	85 db                	test   ebx,ebx
  4f5f4c:	0f 85 78 bb ff ff    	jne    4f1aca <PRIVATE00000000005ce67b+0xcba>
  4f5f52:	83 bc 24 40 03 00 00 	cmp    DWORD PTR [rsp+0x340],0x0
  4f5f59:	00
  4f5f5a:	0f 85 6a bb ff ff    	jne    4f1aca <PRIVATE00000000005ce67b+0xcba>
  4f5f60:	83 bc 24 38 03 00 00 	cmp    DWORD PTR [rsp+0x338],0x0
  4f5f67:	00
  4f5f68:	0f 85 5c bb ff ff    	jne    4f1aca <PRIVATE00000000005ce67b+0xcba>
  4f5f6e:	41 c7 04 24 05 00 00 	mov    DWORD PTR [r12],0x5
  4f5f75:	00
  4f5f76:	e9 4f bb ff ff       	jmp    4f1aca <PRIVATE00000000005ce67b+0xcba>
  4f5f7b:	41 8b 4d 38          	mov    ecx,DWORD PTR [r13+0x38]
  4f5f7f:	83 f9 05             	cmp    ecx,0x5
  4f5f82:	75 0d                	jne    4f5f91 <PRIVATE00000000005ce67b+0x5181>
  4f5f84:	41 c7 45 38 0c 00 00 	mov    DWORD PTR [r13+0x38],0xc
  4f5f8b:	00
  4f5f8c:	e9 39 bb ff ff       	jmp    4f1aca <PRIVATE00000000005ce67b+0xcba>
  4f5f91:	83 f9 04             	cmp    ecx,0x4
  4f5f94:	0f 85 30 bb ff ff    	jne    4f1aca <PRIVATE00000000005ce67b+0xcba>
  4f5f9a:	41 c7 45 38 03 00 00 	mov    DWORD PTR [r13+0x38],0x3
  4f5fa1:	00
  4f5fa2:	e9 23 bb ff ff       	jmp    4f1aca <PRIVATE00000000005ce67b+0xcba>
  4f5fa7:	48 8b 8c 24 b0 03 00 	mov    rcx,QWORD PTR [rsp+0x3b0]
  4f5fae:	00
  4f5faf:	49 3b 8c 24 90 07 00 	cmp    rcx,QWORD PTR [r12+0x790]
  4f5fb6:	00
  4f5fb7:	0f 85 0d bb ff ff    	jne    4f1aca <PRIVATE00000000005ce67b+0xcba>
  4f5fbd:	41 8b 4d 38          	mov    ecx,DWORD PTR [r13+0x38]
  4f5fc1:	41 89 8c 24 a8 00 00 	mov    DWORD PTR [r12+0xa8],ecx
  4f5fc8:	00
  4f5fc9:	83 f9 05             	cmp    ecx,0x5
  4f5fcc:	0f 85 f8 ba ff ff    	jne    4f1aca <PRIVATE00000000005ce67b+0xcba>
  4f5fd2:	45 85 ff             	test   r15d,r15d
  4f5fd5:	0f 85 ef ba ff ff    	jne    4f1aca <PRIVATE00000000005ce67b+0xcba>
  4f5fdb:	85 db                	test   ebx,ebx
  4f5fdd:	0f 85 e7 ba ff ff    	jne    4f1aca <PRIVATE00000000005ce67b+0xcba>
  4f5fe3:	83 bc 24 40 03 00 00 	cmp    DWORD PTR [rsp+0x340],0x0
  4f5fea:	00
  4f5feb:	0f 85 d9 ba ff ff    	jne    4f1aca <PRIVATE00000000005ce67b+0xcba>
  4f5ff1:	83 bc 24 38 03 00 00 	cmp    DWORD PTR [rsp+0x338],0x0
  4f5ff8:	00
  4f5ff9:	0f 85 cb ba ff ff    	jne    4f1aca <PRIVATE00000000005ce67b+0xcba>
  4f5fff:	41 c7 04 24 05 00 00 	mov    DWORD PTR [r12],0x5
  4f6006:	00
  4f6007:	e9 be ba ff ff       	jmp    4f1aca <PRIVATE00000000005ce67b+0xcba>
  4f600c:	45 8b 8c 24 a8 00 00 	mov    r9d,DWORD PTR [r12+0xa8]
  4f6013:	00
  4f6014:	41 83 f9 02          	cmp    r9d,0x2
  4f6018:	74 7a                	je     4f6094 <PRIVATE00000000005ce67b+0x5284>
  4f601a:	48 8b 8c 24 b0 03 00 	mov    rcx,QWORD PTR [rsp+0x3b0]
  4f6021:	00
  4f6022:	49 3b 8c 24 90 07 00 	cmp    rcx,QWORD PTR [r12+0x790]
  4f6029:	00
  4f602a:	74 30                	je     4f605c <PRIVATE00000000005ce67b+0x524c>
  4f602c:	45 85 ff             	test   r15d,r15d
  4f602f:	7e 0d                	jle    4f603e <PRIVATE00000000005ce67b+0x522e>
  4f6031:	41 c7 45 38 0c 00 00 	mov    DWORD PTR [r13+0x38],0xc
  4f6038:	00
  4f6039:	e9 62 ba ff ff       	jmp    4f1aa0 <PRIVATE00000000005ce67b+0xc90>
  4f603e:	85 db                	test   ebx,ebx
  4f6040:	7f ef                	jg     4f6031 <PRIVATE00000000005ce67b+0x5221>
  4f6042:	83 bc 24 40 03 00 00 	cmp    DWORD PTR [rsp+0x340],0x0
  4f6049:	00
  4f604a:	7f e5                	jg     4f6031 <PRIVATE00000000005ce67b+0x5221>
  4f604c:	83 bc 24 38 03 00 00 	cmp    DWORD PTR [rsp+0x338],0x0
  4f6053:	00
  4f6054:	0f 8e 46 ba ff ff    	jle    4f1aa0 <PRIVATE00000000005ce67b+0xc90>
  4f605a:	eb d5                	jmp    4f6031 <PRIVATE00000000005ce67b+0x5221>
  4f605c:	4c 89 e7             	mov    rdi,r12
  4f605f:	e8 8c 3f 01 00       	call   509ff0 <PRIVATE00000000005e7796>

libgurobi130.so:     file format elf64-x86-64


Disassembly of section .text:

00000000005167fe <PRIVATE00000000005fe9cd+0x2be>:
  5167fe:	f2 0f 10 2d 52 5d 55 	movsd  xmm5,QWORD PTR [rip+0x2555d52]        # 2a6c558 <coefclasssign+0x1dd8>
  516805:	02
  516806:	66 0f ef d2          	pxor   xmm2,xmm2
  51680a:	f2 41 0f 10 76 68    	movsd  xmm6,QWORD PTR [r14+0x68]
  516810:	0f 28 c5             	movaps xmm0,xmm5
  516813:	0f 28 ce             	movaps xmm1,xmm6
  516816:	0f 28 dd             	movaps xmm3,xmm5
  516819:	45 0f bf 4f 22       	movsx  r9d,WORD PTR [r15+0x22]
  51681e:	f2 0f 58 c6          	addsd  xmm0,xmm6
  516822:	f2 41 0f 2a d1       	cvtsi2sd xmm2,r9d
  516827:	f2 0f 58 f5          	addsd  xmm6,xmm5
  51682b:	f2 0f 5e c8          	divsd  xmm1,xmm0
  51682f:	f2 41 0f 10 a6 18 08 	movsd  xmm4,QWORD PTR [r14+0x818]
  516836:	00 00
  516838:	f2 0f 5c d5          	subsd  xmm2,xmm5
  51683c:	f2 0f 5d 0d 2c 5d 55 	minsd  xmm1,QWORD PTR [rip+0x2555d2c]        # 2a6c570 <coefclasssign+0x1df0>
  516843:	02
  516844:	f2 0f 59 e1          	mulsd  xmm4,xmm1
  516848:	f2 0f 5c d9          	subsd  xmm3,xmm1
  51684c:	f2 0f 59 da          	mulsd  xmm3,xmm2
  516850:	41 c7 40 70 00 00 00 	mov    DWORD PTR [r8+0x70],0x0
  516857:	00
  516858:	f2 0f 58 e3          	addsd  xmm4,xmm3
  51685c:	45 8b 40 28          	mov    r8d,DWORD PTR [r8+0x28]
  516860:	49 8b 5c 24 08       	mov    rbx,QWORD PTR [r12+0x8]
  516865:	f2 41 0f 11 a6 18 08 	movsd  QWORD PTR [r14+0x818],xmm4
  51686c:	00 00
  51686e:	f2 41 0f 11 76 68    	movsd  QWORD PTR [r14+0x68],xmm6
  516874:	41 8d 68 f9          	lea    ebp,[r8-0x7]
  516878:	4c 8b 93 00 01 00 00 	mov    r10,QWORD PTR [rbx+0x100]
  51687f:	4c 89 54 24 10       	mov    QWORD PTR [rsp+0x10],r10
  516884:	45 33 d2             	xor    r10d,r10d
  516887:	49 8b 9c 24 20 06 00 	mov    rbx,QWORD PTR [r12+0x620]
  51688e:	00
  51688f:	83 fd 01             	cmp    ebp,0x1
  516892:	0f 86 39 01 00 00    	jbe    5169d1 <PRIVATE00000000005fe9cd+0x491>
  516898:	41 83 f8 0a          	cmp    r8d,0xa
  51689c:	0f 84 2f 01 00 00    	je     5169d1 <PRIVATE00000000005fe9cd+0x491>
  5168a2:	45 85 c9             	test   r9d,r9d
  5168a5:	75 25                	jne    5168cc <PRIVATE00000000005fe9cd+0x38c>
  5168a7:	4c 89 fe             	mov    rsi,r15
  5168aa:	33 c9                	xor    ecx,ecx
  5168ac:	48 8b 7c 24 18       	mov    rdi,QWORD PTR [rsp+0x18]
  5168b1:	49 8b 96 80 07 00 00 	mov    rdx,QWORD PTR [r14+0x780]
  5168b8:	44 89 54 24 08       	mov    DWORD PTR [rsp+0x8],r10d
  5168bd:	e8 fe 02 00 00       	call   516bc0 <PRIVATE00000000005f9d4e>
  5168c2:	44 8b 54 24 08       	mov    r10d,DWORD PTR [rsp+0x8]
  5168c7:	e9 0e 02 00 00       	jmp    516ada <PRIVATE00000000005fe9cd+0x59a>
  5168cc:	4c 89 fe             	mov    rsi,r15
  5168cf:	b9 03 00 00 00       	mov    ecx,0x3
  5168d4:	48 8b 7c 24 18       	mov    rdi,QWORD PTR [rsp+0x18]
  5168d9:	49 8b 96 80 07 00 00 	mov    rdx,QWORD PTR [r14+0x780]
  5168e0:	44 89 54 24 08       	mov    DWORD PTR [rsp+0x8],r10d
  5168e5:	e8 d6 02 00 00       	call   516bc0 <PRIVATE00000000005f9d4e>
  5168ea:	49 63 6f 2c          	movsxd rbp,DWORD PTR [r15+0x2c]
  5168ee:	4d 63 47 24          	movsxd r8,DWORD PTR [r15+0x24]
  5168f2:	49 63 5f 28          	movsxd rbx,DWORD PTR [r15+0x28]
  5168f6:	4c 03 c3             	add    r8,rbx
  5168f9:	48 c1 e5 04          	shl    rbp,0x4
  5168fd:	4d 8b 5f 18          	mov    r11,QWORD PTR [r15+0x18]
  516901:	44 8b 54 24 08       	mov    r10d,DWORD PTR [rsp+0x8]
  516906:	4e 8d 64 85 40       	lea    r12,[rbp+r8*4+0x40]
  51690b:	4d 85 db             	test   r11,r11
  51690e:	74 7f                	je     51698f <PRIVATE00000000005fe9cd+0x44f>
  516910:	49 8b 13             	mov    rdx,QWORD PTR [r11]
  516913:	4d 8d bc 24 c0 00 00 	lea    r15,[r12+0xc0]
  51691a:	00
  51691b:	49 83 bb 98 00 00 00 	cmp    QWORD PTR [r11+0x98],0x0
  516922:	00
  516923:	48 8b 4a 08          	mov    rcx,QWORD PTR [rdx+0x8]
  516927:	48 8b 99 e8 00 00 00 	mov    rbx,QWORD PTR [rcx+0xe8]
  51692e:	48 63 73 08          	movsxd rsi,DWORD PTR [rbx+0x8]
  516932:	4c 63 4b 0c          	movsxd r9,DWORD PTR [rbx+0xc]
  516936:	4e 8d 2c 0e          	lea    r13,[rsi+r9*1]
  51693a:	4e 8d 44 4e 1f       	lea    r8,[rsi+r9*2+0x1f]
  51693f:	4b 8d ac ac c0 00 00 	lea    rbp,[r12+r13*4+0xc0]
  516946:	00
  516947:	4c 0f 45 fd          	cmovne r15,rbp
  51694b:	45 33 e4             	xor    r12d,r12d
  51694e:	49 c1 e8 05          	shr    r8,0x5
  516952:	49 c1 e0 02          	shl    r8,0x2
  516956:	49 83 bb b0 00 00 00 	cmp    QWORD PTR [r11+0xb0],0x0
  51695d:	00
  51695e:	4d 0f 44 c4          	cmove  r8,r12
  516962:	49 c1 e1 04          	shl    r9,0x4
  516966:	4d 03 f8             	add    r15,r8
  516969:	4d 03 cf             	add    r9,r15
  51696c:	49 83 bb 80 00 00 00 	cmp    QWORD PTR [r11+0x80],0x0
  516973:	00
  516974:	4d 0f 45 f9          	cmovne r15,r9
  516978:	49 c1 e5 04          	shl    r13,0x4
  51697c:	49 81 c5 d0 00 00 00 	add    r13,0xd0
  516983:	49 83 7b 08 00       	cmp    QWORD PTR [r11+0x8],0x0
  516988:	4d 0f 45 e5          	cmovne r12,r13
  51698c:	4d 03 e7             	add    r12,r15
  51698f:	66 0f ef c0          	pxor   xmm0,xmm0
  516993:	49 8b 8e 80 07 00 00 	mov    rcx,QWORD PTR [r14+0x780]
  51699a:	f2 49 0f 2a c4       	cvtsi2sd xmm0,r12
  51699f:	4d 85 e4             	test   r12,r12
  5169a2:	7d 1a                	jge    5169be <PRIVATE00000000005fe9cd+0x47e>
  5169a4:	4c 89 e2             	mov    rdx,r12
  5169a7:	66 0f ef c0          	pxor   xmm0,xmm0
  5169ab:	49 d1 ec             	shr    r12,1
  5169ae:	48 83 e2 01          	and    rdx,0x1
  5169b2:	49 0b d4             	or     rdx,r12
  5169b5:	f2 48 0f 2a c2       	cvtsi2sd xmm0,rdx
  5169ba:	f2 0f 58 c0          	addsd  xmm0,xmm0
  5169be:	48 ff 41 10          	inc    QWORD PTR [rcx+0x10]
  5169c2:	f2 0f 58 41 20       	addsd  xmm0,QWORD PTR [rcx+0x20]
  5169c7:	f2 0f 11 41 20       	movsd  QWORD PTR [rcx+0x20],xmm0
  5169cc:	e9 09 01 00 00       	jmp    516ada <PRIVATE00000000005fe9cd+0x59a>

libgurobi130.so:     file format elf64-x86-64


Disassembly of section .text:

0000000000517690 <PRIVATE00000000005ff14d>:
  517690:	41 54                	push   r12
  517692:	41 55                	push   r13
  517694:	41 56                	push   r14
  517696:	41 57                	push   r15
  517698:	53                   	push   rbx
  517699:	55                   	push   rbp
  51769a:	48 83 ec 18          	sub    rsp,0x18
  51769e:	49 89 fe             	mov    r14,rdi
  5176a1:	49 89 f7             	mov    r15,rsi
  5176a4:	33 db                	xor    ebx,ebx
  5176a6:	48 89 54 24 08       	mov    QWORD PTR [rsp+0x8],rdx
  5176ab:	49 8b 2e             	mov    rbp,QWORD PTR [r14]
  5176ae:	49 8b 47 08          	mov    rax,QWORD PTR [r15+0x8]
  5176b2:	4d 8b a7 20 06 00 00 	mov    r12,QWORD PTR [r15+0x620]
  5176b9:	48 8b 4d 18          	mov    rcx,QWORD PTR [rbp+0x18]
  5176bd:	4c 8b a8 00 01 00 00 	mov    r13,QWORD PTR [rax+0x100]
  5176c4:	8b 49 28             	mov    ecx,DWORD PTR [rcx+0x28]
  5176c7:	44 8d 41 f9          	lea    r8d,[rcx-0x7]
  5176cb:	41 83 f8 01          	cmp    r8d,0x1
  5176cf:	76 09                	jbe    5176da <PRIVATE00000000005ff14d+0x4a>
  5176d1:	83 f9 0a             	cmp    ecx,0xa
  5176d4:	0f 85 f2 00 00 00    	jne    5177cc <PRIVATE00000000005ff14d+0x13c>
  5176da:	4c 89 ff             	mov    rdi,r15
  5176dd:	e8 ae 25 ff ff       	call   509c90 <PRIVATE00000000005e7204>
  5176e2:	f2 0f 11 04 24       	movsd  QWORD PTR [rsp],xmm0
  5176e7:	48 8b 4d 18          	mov    rcx,QWORD PTR [rbp+0x18]
  5176eb:	83 79 28 0a          	cmp    DWORD PTR [rcx+0x28],0xa
  5176ef:	74 07                	je     5176f8 <PRIVATE00000000005ff14d+0x68>
  5176f1:	c7 41 28 00 00 00 00 	mov    DWORD PTR [rcx+0x28],0x0
  5176f8:	49 8b bc 24 10 36 00 	mov    rdi,QWORD PTR [r12+0x3610]
  5176ff:	00
  517700:	e8 fb 53 3d 00       	call   8ecb00 <PRIVATE0000000000ae1983>
  517705:	f2 41 0f 10 44 24 58 	movsd  xmm0,QWORD PTR [r12+0x58]
  51770c:	f2 0f 10 0d 44 4e 55 	movsd  xmm1,QWORD PTR [rip+0x2554e44]        # 2a6c558 <coefclasssign+0x1dd8>
  517713:	02
  517714:	49 8b bc 24 10 36 00 	mov    rdi,QWORD PTR [r12+0x3610]
  51771b:	00
  51771c:	f2 0f 58 c8          	addsd  xmm1,xmm0
  517720:	f2 0f 11 45 00       	movsd  QWORD PTR [rbp+0x0],xmm0
  517725:	f2 41 0f 11 4c 24 58 	movsd  QWORD PTR [r12+0x58],xmm1
  51772c:	e8 df 53 3d 00       	call   8ecb10 <PRIVATE0000000000ae199e>
  517731:	4c 8b 45 18          	mov    r8,QWORD PTR [rbp+0x18]
  517735:	b9 ff ff ff ff       	mov    ecx,0xffffffff
  51773a:	48 c7 45 08 00 00 00 	mov    QWORD PTR [rbp+0x8],0x0
  517741:	00
  517742:	66 89 4d 20          	mov    WORD PTR [rbp+0x20],cx
  517746:	41 83 78 28 0a       	cmp    DWORD PTR [r8+0x28],0xa
  51774b:	0f 84 8c 00 00 00    	je     5177dd <PRIVATE00000000005ff14d+0x14d>
  517751:	4c 89 ff             	mov    rdi,r15
  517754:	e8 b7 09 ff ff       	call   508110 <PRIVATE00000000005e2919>
  517759:	85 c0                	test   eax,eax
  51775b:	75 52                	jne    5177af <PRIVATE00000000005ff14d+0x11f>
  51775d:	48 8b 4d 18          	mov    rcx,QWORD PTR [rbp+0x18]
  517761:	f2 0f 10 04 24       	movsd  xmm0,QWORD PTR [rsp]
  517766:	66 0f 2f 41 18       	comisd xmm0,QWORD PTR [rcx+0x18]
  51776b:	76 2b                	jbe    517798 <PRIVATE00000000005ff14d+0x108>
  51776d:	83 79 68 04          	cmp    DWORD PTR [rcx+0x68],0x4
  517771:	74 18                	je     51778b <PRIVATE00000000005ff14d+0xfb>
  517773:	4c 89 ef             	mov    rdi,r13
  517776:	48 89 ee             	mov    rsi,rbp
  517779:	b9 04 00 00 00       	mov    ecx,0x4
  51777e:	49 8b 94 24 80 07 00 	mov    rdx,QWORD PTR [r12+0x780]
  517785:	00
  517786:	e8 35 f4 ff ff       	call   516bc0 <PRIVATE00000000005f9d4e>
  51778b:	4c 89 ff             	mov    rdi,r15
  51778e:	48 89 ee             	mov    rsi,rbp
  517791:	e8 da a6 09 00       	call   5b1e70 <PRIVATE00000000006e5802>
  517796:	eb 27                	jmp    5177bf <PRIVATE00000000005ff14d+0x12f>
  517798:	4c 89 ef             	mov    rdi,r13
  51779b:	48 89 ee             	mov    rsi,rbp
  51779e:	33 c9                	xor    ecx,ecx
  5177a0:	49 8b 94 24 80 07 00 	mov    rdx,QWORD PTR [r12+0x780]
  5177a7:	00
  5177a8:	e8 13 f4 ff ff       	call   516bc0 <PRIVATE00000000005f9d4e>
  5177ad:	eb 16                	jmp    5177c5 <PRIVATE00000000005ff14d+0x135>
  5177af:	4c 89 ef             	mov    rdi,r13
  5177b2:	48 89 ea             	mov    rdx,rbp
  5177b5:	48 8b 74 24 08       	mov    rsi,QWORD PTR [rsp+0x8]
  5177ba:	e8 51 87 09 00       	call   5aff10 <PRIVATE00000000006e7842>
  5177bf:	89 c3                	mov    ebx,eax
  5177c1:	85 db                	test   ebx,ebx
  5177c3:	75 07                	jne    5177cc <PRIVATE00000000005ff14d+0x13c>
  5177c5:	49 c7 06 00 00 00 00 	mov    QWORD PTR [r14],0x0
  5177cc:	89 d8                	mov    eax,ebx
  5177ce:	48 83 c4 18          	add    rsp,0x18
  5177d2:	5d                   	pop    rbp
  5177d3:	5b                   	pop    rbx
  5177d4:	41 5f                	pop    r15
  5177d6:	41 5e                	pop    r14
  5177d8:	41 5d                	pop    r13
  5177da:	41 5c                	pop    r12
  5177dc:	c3                   	ret
  5177dd:	48 89 ee             	mov    rsi,rbp
  5177e0:	48 8b 7c 24 08       	mov    rdi,QWORD PTR [rsp+0x8]
  5177e5:	e8 a6 6c 05 00       	call   56e490 <PRIVATE000000000068bdcf>
  5177ea:	eb d3                	jmp    5177bf <PRIVATE00000000005ff14d+0x12f>

libgurobi130.so:     file format elf64-x86-64


Disassembly of section .text:

00000000005180f0 <PRIVATE00000000005ff754>:
  5180f0:	41 54                	push   r12
  5180f2:	48 8b 47 18          	mov    rax,QWORD PTR [rdi+0x18]
  5180f6:	89 70 28             	mov    DWORD PTR [rax+0x28],esi
  5180f9:	83 e6 fe             	and    esi,0xfffffffe
  5180fc:	83 fe 04             	cmp    esi,0x4
  5180ff:	74 03                	je     518104 <PRIVATE00000000005ff754+0x14>
  518101:	41 5c                	pop    r12
  518103:	c3                   	ret
  518104:	48 8b 00             	mov    rax,QWORD PTR [rax]
  518107:	48 3b 80 28 06 00 00 	cmp    rax,QWORD PTR [rax+0x628]
  51810e:	75 f1                	jne    518101 <PRIVATE00000000005ff754+0x11>
  518110:	48 3b b8 90 07 00 00 	cmp    rdi,QWORD PTR [rax+0x790]
  518117:	75 e8                	jne    518101 <PRIVATE00000000005ff754+0x11>
  518119:	4c 8b a0 20 06 00 00 	mov    r12,QWORD PTR [rax+0x620]
  518120:	4c 89 e7             	mov    rdi,r12
  518123:	e8 c8 1e ff ff       	call   509ff0 <PRIVATE00000000005e7796>
  518128:	0f 28 c8             	movaps xmm1,xmm0
  51812b:	f2 0f 10 05 6d 43 55 	movsd  xmm0,QWORD PTR [rip+0x255436d]        # 2a6c4a0 <coefclasssign+0x1d20>
  518132:	02
  518133:	66 0f 2f c1          	comisd xmm0,xmm1
  518137:	76 0a                	jbe    518143 <PRIVATE00000000005ff754+0x53>
  518139:	41 c7 04 24 02 00 00 	mov    DWORD PTR [r12],0x2
  518140:	00
  518141:	eb 08                	jmp    51814b <PRIVATE00000000005ff754+0x5b>
  518143:	41 c7 04 24 03 00 00 	mov    DWORD PTR [r12],0x3
  51814a:	00
  51814b:	49 8b 84 24 80 07 00 	mov    rax,QWORD PTR [r12+0x780]
  518152:	00
  518153:	f2 0f 10 80 68 01 00 	movsd  xmm0,QWORD PTR [rax+0x168]
  51815a:	00
  51815b:	f2 0f 5f c1          	maxsd  xmm0,xmm1
  51815f:	f2 0f 11 80 68 01 00 	movsd  QWORD PTR [rax+0x168],xmm0
  518166:	00
  518167:	41 5c                	pop    r12
  518169:	c3                   	ret

libgurobi130.so:     file format elf64-x86-64


Disassembly of section .text:

0000000000519b10 <PRIVATE000000000060145b>:
  519b10:	41 57                	push   r15
  519b12:	53                   	push   rbx
  519b13:	56                   	push   rsi
  519b14:	49 89 ff             	mov    r15,rdi
  519b17:	49 8b 4f 18          	mov    rcx,QWORD PTR [r15+0x18]
  519b1b:	8b 51 28             	mov    edx,DWORD PTR [rcx+0x28]
  519b1e:	89 d0                	mov    eax,edx
  519b20:	83 e0 fe             	and    eax,0xfffffffe
  519b23:	83 f8 02             	cmp    eax,0x2
  519b26:	74 31                	je     519b59 <PRIVATE000000000060145b+0x49>
  519b28:	33 db                	xor    ebx,ebx
  519b2a:	83 fa 01             	cmp    edx,0x1
  519b2d:	74 1e                	je     519b4d <PRIVATE000000000060145b+0x3d>
  519b2f:	83 79 28 01          	cmp    DWORD PTR [rcx+0x28],0x1
  519b33:	74 07                	je     519b3c <PRIVATE000000000060145b+0x2c>
  519b35:	89 d8                	mov    eax,ebx
  519b37:	59                   	pop    rcx
  519b38:	5b                   	pop    rbx
  519b39:	41 5f                	pop    r15
  519b3b:	c3                   	ret
  519b3c:	4c 89 ff             	mov    rdi,r15
  519b3f:	e8 fc c1 10 00       	call   625d40 <PRIVATE0000000000796ffb>
  519b44:	33 db                	xor    ebx,ebx
  519b46:	85 c0                	test   eax,eax
  519b48:	0f 95 c3             	setne  bl
  519b4b:	eb e8                	jmp    519b35 <PRIVATE000000000060145b+0x25>
  519b4d:	48 8b 39             	mov    rdi,QWORD PTR [rcx]
  519b50:	e8 bb 5e ff ff       	call   50fa10 <PRIVATE00000000005f9a41>
  519b55:	a8 01                	test   al,0x1
  519b57:	74 07                	je     519b60 <PRIVATE000000000060145b+0x50>
  519b59:	bb 01 00 00 00       	mov    ebx,0x1
  519b5e:	eb d5                	jmp    519b35 <PRIVATE000000000060145b+0x25>
  519b60:	49 8b 4f 18          	mov    rcx,QWORD PTR [r15+0x18]
  519b64:	eb c9                	jmp    519b2f <PRIVATE000000000060145b+0x1f>

libgurobi130.so:     file format elf64-x86-64


Disassembly of section .text:

00000000005320c0 <PRIVATE0000000000627f3e>:
  5320c0:	41 54                	push   r12
  5320c2:	41 55                	push   r13
  5320c4:	41 56                	push   r14
  5320c6:	41 57                	push   r15
  5320c8:	53                   	push   rbx
  5320c9:	55                   	push   rbp
  5320ca:	48 81 ec e8 00 00 00 	sub    rsp,0xe8
  5320d1:	48 89 fd             	mov    rbp,rdi
  5320d4:	f2 0f 11 04 24       	movsd  QWORD PTR [rsp],xmm0
  5320d9:	4d 89 cd             	mov    r13,r9
  5320dc:	48 89 8c 24 c0 00 00 	mov    QWORD PTR [rsp+0xc0],rcx
  5320e3:	00
  5320e4:	45 89 c6             	mov    r14d,r8d
  5320e7:	48 89 94 24 c8 00 00 	mov    QWORD PTR [rsp+0xc8],rdx
  5320ee:	00
  5320ef:	48 8b 45 00          	mov    rax,QWORD PTR [rbp+0x0]
  5320f3:	48 89 b4 24 d0 00 00 	mov    QWORD PTR [rsp+0xd0],rsi
  5320fa:	00
  5320fb:	4c 8b 50 18          	mov    r10,QWORD PTR [rax+0x18]
  5320ff:	49 8b 3a             	mov    rdi,QWORD PTR [r10]
  532102:	48 89 bc 24 b8 00 00 	mov    QWORD PTR [rsp+0xb8],rdi
  532109:	00
  53210a:	e8 81 7b fd ff       	call   509c90 <PRIVATE00000000005e7204>
  53210f:	48 8b bc 24 c8 00 00 	mov    rdi,QWORD PTR [rsp+0xc8]
  532116:	00
  532117:	e8 f4 eb fb ff       	call   4f0d10 <PRIVATE00000000005cc64c>
  53211c:	48 8b 45 00          	mov    rax,QWORD PTR [rbp+0x0]
  532120:	48 89 84 24 90 00 00 	mov    QWORD PTR [rsp+0x90],rax
  532127:	00
  532128:	48 8b 40 18          	mov    rax,QWORD PTR [rax+0x18]
  53212c:	83 78 28 02          	cmp    DWORD PTR [rax+0x28],0x2
  532130:	0f 84 36 0e 00 00    	je     532f6c <PRIVATE0000000000627f3e+0xeac>
  532136:	4c 8b a4 24 d0 00 00 	mov    r12,QWORD PTR [rsp+0xd0]
  53213d:	00
  53213e:	4d 8b bc 24 d0 40 00 	mov    r15,QWORD PTR [r12+0x40d0]
  532145:	00
  532146:	4c 8b 20             	mov    r12,QWORD PTR [rax]
  532149:	4c 89 e7             	mov    rdi,r12
  53214c:	4c 89 7c 24 68       	mov    QWORD PTR [rsp+0x68],r15
  532151:	4d 8b 74 24 08       	mov    r14,QWORD PTR [r12+0x8]
  532156:	4d 8b 94 24 50 06 00 	mov    r10,QWORD PTR [r12+0x650]
  53215d:	00
  53215e:	4c 89 74 24 70       	mov    QWORD PTR [rsp+0x70],r14
  532163:	4d 8b 86 e8 00 00 00 	mov    r8,QWORD PTR [r14+0xe8]
  53216a:	4d 8b be 00 01 00 00 	mov    r15,QWORD PTR [r14+0x100]
  532171:	4c 89 94 24 80 00 00 	mov    QWORD PTR [rsp+0x80],r10
  532178:	00
  532179:	45 8b 68 1c          	mov    r13d,DWORD PTR [r8+0x1c]
  53217d:	44 89 6c 24 30       	mov    DWORD PTR [rsp+0x30],r13d
  532182:	4c 8b a8 80 00 00 00 	mov    r13,QWORD PTR [rax+0x80]
  532189:	48 8b 80 88 00 00 00 	mov    rax,QWORD PTR [rax+0x88]
  532190:	49 8b 90 c0 02 00 00 	mov    rdx,QWORD PTR [r8+0x2c0]
  532197:	49 8b 88 c8 02 00 00 	mov    rcx,QWORD PTR [r8+0x2c8]
  53219e:	49 8b 98 c0 01 00 00 	mov    rbx,QWORD PTR [r8+0x1c0]
  5321a5:	4d 8b 88 d0 01 00 00 	mov    r9,QWORD PTR [r8+0x1d0]
  5321ac:	f2 41 0f 10 87 68 24 	movsd  xmm0,QWORD PTR [r15+0x2468]
  5321b3:	00 00
  5321b5:	4d 63 70 0c          	movsxd r14,DWORD PTR [r8+0xc]
  5321b9:	48 89 54 24 28       	mov    QWORD PTR [rsp+0x28],rdx
  5321be:	48 89 4c 24 20       	mov    QWORD PTR [rsp+0x20],rcx
  5321c3:	48 89 5c 24 18       	mov    QWORD PTR [rsp+0x18],rbx
  5321c8:	33 db                	xor    ebx,ebx
  5321ca:	4c 89 4c 24 10       	mov    QWORD PTR [rsp+0x10],r9
  5321cf:	48 89 44 24 38       	mov    QWORD PTR [rsp+0x38],rax
  5321d4:	f2 0f 11 84 24 88 00 	movsd  QWORD PTR [rsp+0x88],xmm0
  5321db:	00 00
  5321dd:	e8 2e 5f fd ff       	call   508110 <PRIVATE00000000005e2919>
  5321e2:	85 c0                	test   eax,eax
  5321e4:	74 16                	je     5321fc <PRIVATE0000000000627f3e+0x13c>
  5321e6:	48 8b 8c 24 d0 00 00 	mov    rcx,QWORD PTR [rsp+0xd0]
  5321ed:	00
  5321ee:	4c 8b 81 00 40 00 00 	mov    r8,QWORD PTR [rcx+0x4000]
  5321f5:	4c 89 44 24 48       	mov    QWORD PTR [rsp+0x48],r8
  5321fa:	eb 0d                	jmp    532209 <PRIVATE0000000000627f3e+0x149>
  5321fc:	49 8b 8c 24 60 32 00 	mov    rcx,QWORD PTR [r12+0x3260]
  532203:	00
  532204:	48 89 4c 24 48       	mov    QWORD PTR [rsp+0x48],rcx
  532209:	4c 8b 84 24 c8 00 00 	mov    r8,QWORD PTR [rsp+0xc8]
  532210:	00
  532211:	ba ff ff ff ff       	mov    edx,0xffffffff
  532216:	66 0f ef c0          	pxor   xmm0,xmm0
  53221a:	89 d1                	mov    ecx,edx
  53221c:	c7 84 24 d8 00 00 00 	mov    DWORD PTR [rsp+0xd8],0xffffffff
  532223:	ff ff ff ff
  532227:	33 f6                	xor    esi,esi
  532229:	4d 8b 48 08          	mov    r9,QWORD PTR [r8+0x8]
  53222d:	0f 28 e0             	movaps xmm4,xmm0
  532230:	c7 44 24 78 00 94 35 	mov    DWORD PTR [rsp+0x78],0x77359400
  532237:	77
  532238:	41 83 79 64 05       	cmp    DWORD PTR [r9+0x64],0x5
  53223d:	0f 84 6e 0b 00 00    	je     532db1 <PRIVATE0000000000627f3e+0xcf1>
  532243:	48 8b bc 24 90 00 00 	mov    rdi,QWORD PTR [rsp+0x90]
  53224a:	00
  53224b:	f2 0f 11 44 24 50    	movsd  QWORD PTR [rsp+0x50],xmm0
  532251:	f2 0f 11 64 24 30    	movsd  QWORD PTR [rsp+0x30],xmm4
  532257:	89 34 24             	mov    DWORD PTR [rsp],esi
  53225a:	89 4c 24 40          	mov    DWORD PTR [rsp+0x40],ecx
  53225e:	e8 6d 3b 0f 00       	call   625dd0 <PRIVATE00000000007970f2>
  532263:	8b 4c 24 40          	mov    ecx,DWORD PTR [rsp+0x40]
  532267:	8b 34 24             	mov    esi,DWORD PTR [rsp]
  53226a:	f2 0f 10 64 24 30    	movsd  xmm4,QWORD PTR [rsp+0x30]
  532270:	f2 0f 10 44 24 50    	movsd  xmm0,QWORD PTR [rsp+0x50]
  532276:	85 c0                	test   eax,eax
  532278:	7c 2a                	jl     5322a4 <PRIVATE0000000000627f3e+0x1e4>
  53227a:	48 8b 4c 24 70       	mov    rcx,QWORD PTR [rsp+0x70]
  53227f:	4c 8b 81 e8 00 00 00 	mov    r8,QWORD PTR [rcx+0xe8]
  532286:	33 c9                	xor    ecx,ecx
  532288:	41 8b 90 f0 00 00 00 	mov    edx,DWORD PTR [r8+0xf0]
  53228f:	03 d0                	add    edx,eax
  532291:	89 94 24 d8 00 00 00 	mov    DWORD PTR [rsp+0xd8],edx
  532298:	48 63 d2             	movsxd rdx,edx
  53229b:	f2 41 0f 10 64 d5 00 	movsd  xmm4,QWORD PTR [r13+rdx*8+0x0]
  5322a2:	eb 07                	jmp    5322ab <PRIVATE0000000000627f3e+0x1eb>
  5322a4:	8b 94 24 d8 00 00 00 	mov    edx,DWORD PTR [rsp+0xd8]
  5322ab:	83 fa ff             	cmp    edx,0xffffffff
  5322ae:	0f 84 18 02 00 00    	je     5324cc <PRIVATE0000000000627f3e+0x40c>
  5322b4:	85 c9                	test   ecx,ecx
  5322b6:	74 57                	je     53230f <PRIVATE0000000000627f3e+0x24f>
  5322b8:	83 f9 05             	cmp    ecx,0x5
  5322bb:	75 29                	jne    5322e6 <PRIVATE0000000000627f3e+0x226>
  5322bd:	48 8b bc 24 90 00 00 	mov    rdi,QWORD PTR [rsp+0x90]
  5322c4:	00
  5322c5:	48 8b b4 24 c8 00 00 	mov    rsi,QWORD PTR [rsp+0xc8]
  5322cc:	00
  5322cd:	48 8b 8c 24 c0 00 00 	mov    rcx,QWORD PTR [rsp+0xc0]
  5322d4:	00
  5322d5:	4c 8b 44 24 68       	mov    r8,QWORD PTR [rsp+0x68]
  5322da:	e8 f1 1e 00 00       	call   5341d0 <PRIVATE00000000006406da>
  5322df:	89 c3                	mov    ebx,eax
  5322e1:	e9 98 01 00 00       	jmp    53247e <PRIVATE0000000000627f3e+0x3be>
  5322e6:	48 8b bc 24 90 00 00 	mov    rdi,QWORD PTR [rsp+0x90]
  5322ed:	00
  5322ee:	48 8b b4 24 c8 00 00 	mov    rsi,QWORD PTR [rsp+0xc8]
  5322f5:	00
  5322f6:	4c 8b 84 24 c0 00 00 	mov    r8,QWORD PTR [rsp+0xc0]
  5322fd:	00
  5322fe:	4c 8b 4c 24 68       	mov    r9,QWORD PTR [rsp+0x68]
  532303:	e8 a8 2d 00 00       	call   5350b0 <PRIVATE000000000063c7ae>
  532308:	89 c3                	mov    ebx,eax
  53230a:	e9 6f 01 00 00       	jmp    53247e <PRIVATE0000000000627f3e+0x3be>
  53230f:	0f 28 cc             	movaps xmm1,xmm4
  532312:	45 33 ed             	xor    r13d,r13d
  532315:	48 89 ac 24 b0 00 00 	mov    QWORD PTR [rsp+0xb0],rbp
  53231c:	00
  53231d:	f2 0f 10 05 c3 a5 53 	movsd  xmm0,QWORD PTR [rip+0x253a5c3]        # 2a6c8e8 <coefclasssign+0x2168>
  532324:	02
  532325:	4c 8b b4 24 90 00 00 	mov    r14,QWORD PTR [rsp+0x90]
  53232c:	00
  53232d:	48 8b ac 24 c0 00 00 	mov    rbp,QWORD PTR [rsp+0xc0]
  532334:	00
  532335:	f2 0f 58 0d 0b a6 53 	addsd  xmm1,QWORD PTR [rip+0x253a60b]        # 2a6c948 <coefclasssign+0x21c8>
  53233c:	02
  53233d:	f2 0f 11 0c 24       	movsd  QWORD PTR [rsp],xmm1
  532342:	eb 06                	jmp    53234a <PRIVATE0000000000627f3e+0x28a>
  532344:	f2 0f 10 64 24 30    	movsd  xmm4,QWORD PTR [rsp+0x30]
  53234a:	c6 44 24 14 3c       	mov    BYTE PTR [rsp+0x14],0x3c
  53234f:	f2 0f 11 64 24 40    	movsd  QWORD PTR [rsp+0x40],xmm4
  532355:	f2 0f 11 64 24 30    	movsd  QWORD PTR [rsp+0x30],xmm4
  53235b:	eb 10                	jmp    53236d <PRIVATE0000000000627f3e+0x2ad>
  53235d:	f2 0f 10 04 24       	movsd  xmm0,QWORD PTR [rsp]
  532362:	c6 44 24 14 3e       	mov    BYTE PTR [rsp+0x14],0x3e
  532367:	f2 0f 11 44 24 40    	movsd  QWORD PTR [rsp+0x40],xmm0
  53236d:	4c 89 f7             	mov    rdi,r14
  532370:	4e 8d 7c ed 00       	lea    r15,[rbp+r13*8+0x0]
  532375:	4c 89 e6             	mov    rsi,r12
  532378:	4c 89 fa             	mov    rdx,r15
  53237b:	e8 70 54 fe ff       	call   5177f0 <PRIVATE00000000005ff358>
  532380:	89 c3                	mov    ebx,eax
  532382:	85 db                	test   ebx,ebx
  532384:	0f 85 ec 00 00 00    	jne    532476 <PRIVATE0000000000627f3e+0x3b6>
  53238a:	4a 8b 44 ed 00       	mov    rax,QWORD PTR [rbp+r13*8+0x0]
  53238f:	4c 89 ff             	mov    rdi,r15
  532392:	8b 9c 24 d8 00 00 00 	mov    ebx,DWORD PTR [rsp+0xd8]
  532399:	ba 01 00 00 00       	mov    edx,0x1
  53239e:	f2 0f 10 44 24 30    	movsd  xmm0,QWORD PTR [rsp+0x30]
  5323a4:	4c 8b 58 18          	mov    r11,QWORD PTR [rax+0x18]
  5323a8:	44 8a 54 24 14       	mov    r10b,BYTE PTR [rsp+0x14]
  5323ad:	41 89 5b 34          	mov    DWORD PTR [r11+0x34],ebx
  5323b1:	f2 41 0f 11 43 38    	movsd  QWORD PTR [r11+0x38],xmm0
  5323b7:	45 88 53 40          	mov    BYTE PTR [r11+0x40],r10b
  5323bb:	ff 74 24 68          	push   QWORD PTR [rsp+0x68]
  5323bf:	6a 01                	push   0x1
  5323c1:	48 8b b4 24 d8 00 00 	mov    rsi,QWORD PTR [rsp+0xd8]
  5323c8:	00
  5323c9:	48 8d 8c 24 e8 00 00 	lea    rcx,[rsp+0xe8]
  5323d0:	00
  5323d1:	4c 8d 44 24 50       	lea    r8,[rsp+0x50]
  5323d6:	4c 8d 4c 24 24       	lea    r9,[rsp+0x24]
  5323db:	e8 30 7a fe ff       	call   519e10 <PRIVATE0000000000603386>
  5323e0:	89 c3                	mov    ebx,eax
  5323e2:	48 83 c4 10          	add    rsp,0x10
  5323e6:	85 db                	test   ebx,ebx
  5323e8:	0f 85 88 00 00 00    	jne    532476 <PRIVATE0000000000627f3e+0x3b6>
  5323ee:	f2 0f 10 44 24 40    	movsd  xmm0,QWORD PTR [rsp+0x40]
  5323f4:	8b b4 24 d8 00 00 00 	mov    esi,DWORD PTR [rsp+0xd8]
  5323fb:	0f be 54 24 14       	movsx  edx,BYTE PTR [rsp+0x14]
  532400:	49 8b bc 24 98 32 00 	mov    rdi,QWORD PTR [r12+0x3298]
  532407:	00
  532408:	e8 a3 0e 06 00       	call   5932b0 <PRIVATE00000000006c4eee>
  53240d:	0f 28 c8             	movaps xmm1,xmm0
  532410:	49 8b 4e 18          	mov    rcx,QWORD PTR [r14+0x18]
  532414:	4a 8b 54 ed 00       	mov    rdx,QWORD PTR [rbp+r13*8+0x0]
  532419:	f2 0f 10 41 18       	movsd  xmm0,QWORD PTR [rcx+0x18]
  53241e:	4c 8b 4a 18          	mov    r9,QWORD PTR [rdx+0x18]
  532422:	f2 0f 5f c1          	maxsd  xmm0,xmm1
  532426:	f2 41 0f 10 51 18    	movsd  xmm2,QWORD PTR [r9+0x18]
  53242c:	f2 41 0f 10 4c 24 40 	movsd  xmm1,QWORD PTR [r12+0x40]
  532433:	f2 0f 5f d0          	maxsd  xmm2,xmm0
  532437:	66 0f 2e 0d a9 a4 53 	ucomisd xmm1,QWORD PTR [rip+0x253a4a9]        # 2a6c8e8 <coefclasssign+0x2168>
  53243e:	02
  53243f:	4c 8b 41 20          	mov    r8,QWORD PTR [rcx+0x20]
  532443:	f2 41 0f 11 51 18    	movsd  QWORD PTR [r9+0x18],xmm2
  532449:	4d 89 41 20          	mov    QWORD PTR [r9+0x20],r8
  53244d:	7a 02                	jp     532451 <PRIVATE0000000000627f3e+0x391>
  53244f:	74 08                	je     532459 <PRIVATE0000000000627f3e+0x399>
  532451:	f2 41 0f 11 51 50    	movsd  QWORD PTR [r9+0x50],xmm2
  532457:	eb 06                	jmp    53245f <PRIVATE0000000000627f3e+0x39f>
  532459:	33 d2                	xor    edx,edx
  53245b:	49 89 51 50          	mov    QWORD PTR [r9+0x50],rdx
  53245f:	49 ff c5             	inc    r13
  532462:	49 83 fd 01          	cmp    r13,0x1
  532466:	7f 0e                	jg     532476 <PRIVATE0000000000627f3e+0x3b6>
  532468:	4d 85 ed             	test   r13,r13
  53246b:	0f 84 d3 fe ff ff    	je     532344 <PRIVATE0000000000627f3e+0x284>
  532471:	e9 e7 fe ff ff       	jmp    53235d <PRIVATE0000000000627f3e+0x29d>
  532476:	48 8b ac 24 b0 00 00 	mov    rbp,QWORD PTR [rsp+0xb0]
  53247d:	00

libgurobi130.so:     file format elf64-x86-64


Disassembly of section .text:

00000000005324cc <PRIVATE0000000000627f3e+0x40c>:
  5324cc:	4c 8b 44 24 70       	mov    r8,QWORD PTR [rsp+0x70]
  5324d1:	4d 8b 80 e8 00 00 00 	mov    r8,QWORD PTR [r8+0xe8]
  5324d8:	41 83 78 18 00       	cmp    DWORD PTR [r8+0x18],0x0
  5324dd:	0f 8e a0 00 00 00    	jle    532583 <PRIVATE0000000000627f3e+0x4c3>
  5324e3:	33 d2                	xor    edx,edx
  5324e5:	4c 89 64 24 58       	mov    QWORD PTR [rsp+0x58],r12
  5324ea:	48 89 ac 24 b0 00 00 	mov    QWORD PTR [rsp+0xb0],rbp
  5324f1:	00
  5324f2:	f2 0f 11 44 24 50    	movsd  QWORD PTR [rsp+0x50],xmm0
  5324f8:	f2 0f 11 64 24 30    	movsd  QWORD PTR [rsp+0x30],xmm4
  5324fe:	89 4c 24 40          	mov    DWORD PTR [rsp+0x40],ecx
  532502:	4c 89 74 24 10       	mov    QWORD PTR [rsp+0x10],r14
  532507:	41 89 de             	mov    r14d,ebx
  53250a:	4c 89 7c 24 08       	mov    QWORD PTR [rsp+0x8],r15
  53250f:	41 89 d7             	mov    r15d,edx
  532512:	4c 8b 64 24 38       	mov    r12,QWORD PTR [rsp+0x38]
  532517:	89 f3                	mov    ebx,esi
  532519:	48 8b 6c 24 70       	mov    rbp,QWORD PTR [rsp+0x70]
  53251e:	48 89 ef             	mov    rdi,rbp
  532521:	44 89 fe             	mov    esi,r15d
  532524:	4c 89 ea             	mov    rdx,r13
  532527:	4c 89 e1             	mov    rcx,r12
  53252a:	e8 d1 a8 35 00       	call   88ce00 <PRIVATE0000000000a7e4de>
  53252f:	85 c0                	test   eax,eax
  532531:	0f 84 2a 08 00 00    	je     532d61 <PRIVATE0000000000627f3e+0xca1>
  532537:	4c 8b 85 e8 00 00 00 	mov    r8,QWORD PTR [rbp+0xe8]
  53253e:	41 ff c7             	inc    r15d
  532541:	45 3b 78 18          	cmp    r15d,DWORD PTR [r8+0x18]
  532545:	7c d7                	jl     53251e <PRIVATE0000000000627f3e+0x45e>
  532547:	f2 0f 10 44 24 50    	movsd  xmm0,QWORD PTR [rsp+0x50]
  53254d:	89 de                	mov    esi,ebx
  53254f:	f2 0f 10 64 24 30    	movsd  xmm4,QWORD PTR [rsp+0x30]
  532555:	44 89 f3             	mov    ebx,r14d
  532558:	8b 4c 24 40          	mov    ecx,DWORD PTR [rsp+0x40]
  53255c:	4c 8b 74 24 10       	mov    r14,QWORD PTR [rsp+0x10]
  532561:	4c 8b 7c 24 08       	mov    r15,QWORD PTR [rsp+0x8]
  532566:	4c 8b 64 24 58       	mov    r12,QWORD PTR [rsp+0x58]
  53256b:	48 8b ac 24 b0 00 00 	mov    rbp,QWORD PTR [rsp+0xb0]
  532572:	00
  532573:	8b 94 24 d8 00 00 00 	mov    edx,DWORD PTR [rsp+0xd8]
  53257a:	83 fa ff             	cmp    edx,0xffffffff
  53257d:	0f 85 31 fd ff ff    	jne    5322b4 <PRIVATE0000000000627f3e+0x1f4>
  532583:	41 83 b8 58 02 00 00 	cmp    DWORD PTR [r8+0x258],0x0
  53258a:	00
  53258b:	0f 8e 88 02 00 00    	jle    532819 <PRIVATE0000000000627f3e+0x759>
  532591:	4d 8b 88 80 02 00 00 	mov    r9,QWORD PTR [r8+0x280]
  532598:	b0 43                	mov    al,0x43
  53259a:	4d 8b 80 88 02 00 00 	mov    r8,QWORD PTR [r8+0x288]
  5325a1:	40 b7 01             	mov    dil,0x1
  5325a4:	f2 0f 10 15 6c a3 53 	movsd  xmm2,QWORD PTR [rip+0x253a36c]        # 2a6c918 <coefclasssign+0x2198>
  5325ab:	02
  5325ac:	33 d2                	xor    edx,edx
  5325ae:	89 5c 24 60          	mov    DWORD PTR [rsp+0x60],ebx
  5325b2:	4c 89 64 24 58       	mov    QWORD PTR [rsp+0x58],r12
  5325b7:	f2 0f 10 0d f9 a2 53 	movsd  xmm1,QWORD PTR [rip+0x253a2f9]        # 2a6c8b8 <coefclasssign+0x2138>
  5325be:	02
  5325bf:	f2 0f 11 44 24 50    	movsd  QWORD PTR [rsp+0x50],xmm0
  5325c5:	f2 0f 11 54 24 18    	movsd  QWORD PTR [rsp+0x18],xmm2
  5325cb:	f2 0f 11 64 24 30    	movsd  QWORD PTR [rsp+0x30],xmm4
  5325d1:	89 34 24             	mov    DWORD PTR [rsp],esi
  5325d4:	4c 89 44 24 28       	mov    QWORD PTR [rsp+0x28],r8
  5325d9:	4c 89 4c 24 20       	mov    QWORD PTR [rsp+0x20],r9
  5325de:	89 4c 24 40          	mov    DWORD PTR [rsp+0x40],ecx
  5325e2:	4c 89 74 24 10       	mov    QWORD PTR [rsp+0x10],r14
  5325e7:	41 88 fe             	mov    r14b,dil
  5325ea:	4c 89 7c 24 08       	mov    QWORD PTR [rsp+0x8],r15
  5325ef:	41 89 d7             	mov    r15d,edx
  5325f2:	48 89 ac 24 b0 00 00 	mov    QWORD PTR [rsp+0xb0],rbp
  5325f9:	00
  5325fa:	40 88 c5             	mov    bpl,al
  5325fd:	48 8b 5c 24 38       	mov    rbx,QWORD PTR [rsp+0x38]
  532602:	4c 8b 64 24 70       	mov    r12,QWORD PTR [rsp+0x70]
  532607:	4c 89 e7             	mov    rdi,r12
  53260a:	44 89 fe             	mov    esi,r15d
  53260d:	4c 89 ea             	mov    rdx,r13
  532610:	48 89 d9             	mov    rcx,rbx
  532613:	e8 38 c9 0e 00       	call   61ef50 <PRIVATE000000000078dd89>
  532618:	85 c0                	test   eax,eax
  53261a:	0f 85 a3 01 00 00    	jne    5327c3 <PRIVATE0000000000627f3e+0x703>
  532620:	4c 89 e7             	mov    rdi,r12
  532623:	44 89 fe             	mov    esi,r15d
  532626:	4c 89 ea             	mov    rdx,r13
  532629:	48 89 d9             	mov    rcx,rbx
  53262c:	4c 8b 44 24 68       	mov    r8,QWORD PTR [rsp+0x68]
  532631:	e8 ea c6 0e 00       	call   61ed20 <PRIVATE000000000078d933>
  532636:	85 c0                	test   eax,eax
  532638:	0f 85 85 01 00 00    	jne    5327c3 <PRIVATE0000000000627f3e+0x703>
  53263e:	4d 63 ff             	movsxd r15,r15d
  532641:	4c 8b 4c 24 20       	mov    r9,QWORD PTR [rsp+0x20]
  532646:	4c 8b 44 24 28       	mov    r8,QWORD PTR [rsp+0x28]
  53264b:	4f 63 0c b9          	movsxd r9,DWORD PTR [r9+r15*4]
  53264f:	4b 63 04 b8          	movsxd rax,DWORD PTR [r8+r15*4]
  532653:	f2 42 0f 10 0c cb    	movsd  xmm1,QWORD PTR [rbx+r9*8]
  532659:	f2 0f 10 1c c3       	movsd  xmm3,QWORD PTR [rbx+rax*8]
  53265e:	0f 28 e1             	movaps xmm4,xmm1
  532661:	f2 43 0f 10 44 cd 00 	movsd  xmm0,QWORD PTR [r13+r9*8+0x0]
  532668:	0f 28 eb             	movaps xmm5,xmm3
  53266b:	f2 41 0f 10 54 c5 00 	movsd  xmm2,QWORD PTR [r13+rax*8+0x0]
  532672:	f2 0f 5c e0          	subsd  xmm4,xmm0
  532676:	f2 0f 5c ea          	subsd  xmm5,xmm2
  53267a:	66 0f 2f 05 36 a2 53 	comisd xmm0,QWORD PTR [rip+0x253a236]        # 2a6c8b8 <coefclasssign+0x2138>
  532681:	02
  532682:	73 0e                	jae    532692 <PRIVATE0000000000627f3e+0x5d2>
  532684:	f2 0f 10 05 24 a2 53 	movsd  xmm0,QWORD PTR [rip+0x253a224]        # 2a6c8b0 <coefclasssign+0x2130>
  53268b:	02
  53268c:	66 0f 2f c1          	comisd xmm0,xmm1
  532690:	72 04                	jb     532696 <PRIVATE0000000000627f3e+0x5d6>
  532692:	b2 01                	mov    dl,0x1
  532694:	eb 02                	jmp    532698 <PRIVATE0000000000627f3e+0x5d8>
  532696:	32 d2                	xor    dl,dl
  532698:	66 0f 2f 15 18 a2 53 	comisd xmm2,QWORD PTR [rip+0x253a218]        # 2a6c8b8 <coefclasssign+0x2138>
  53269f:	02
  5326a0:	73 0e                	jae    5326b0 <PRIVATE0000000000627f3e+0x5f0>
  5326a2:	f2 0f 10 05 06 a2 53 	movsd  xmm0,QWORD PTR [rip+0x253a206]        # 2a6c8b0 <coefclasssign+0x2130>
  5326a9:	02
  5326aa:	66 0f 2f c3          	comisd xmm0,xmm3
  5326ae:	72 05                	jb     5326b5 <PRIVATE0000000000627f3e+0x5f5>
  5326b0:	41 b0 01             	mov    r8b,0x1
  5326b3:	eb 03                	jmp    5326b8 <PRIVATE0000000000627f3e+0x5f8>
  5326b5:	45 32 c0             	xor    r8b,r8b
  5326b8:	66 0f 2f a4 24 88 00 	comisd xmm4,QWORD PTR [rsp+0x88]
  5326bf:	00 00
  5326c1:	76 7b                	jbe    53273e <PRIVATE0000000000627f3e+0x67e>
  5326c3:	41 3a d6             	cmp    dl,r14b
  5326c6:	7f 76                	jg     53273e <PRIVATE0000000000627f3e+0x67e>
  5326c8:	83 bc 24 d8 00 00 00 	cmp    DWORD PTR [rsp+0xd8],0xffffffff
  5326cf:	ff
  5326d0:	74 47                	je     532719 <PRIVATE0000000000627f3e+0x659>
  5326d2:	41 3a d6             	cmp    dl,r14b
  5326d5:	7c 42                	jl     532719 <PRIVATE0000000000627f3e+0x659>
  5326d7:	4c 8b 94 24 80 00 00 	mov    r10,QWORD PTR [rsp+0x80]
  5326de:	00
  5326df:	47 8a 14 11          	mov    r10b,BYTE PTR [r9+r10*1]
  5326e3:	41 80 fa 43          	cmp    r10b,0x43
  5326e7:	74 0a                	je     5326f3 <PRIVATE0000000000627f3e+0x633>
  5326e9:	40 80 fd 43          	cmp    bpl,0x43
  5326ed:	74 2a                	je     532719 <PRIVATE0000000000627f3e+0x659>
  5326ef:	41 80 fa 43          	cmp    r10b,0x43
  5326f3:	41 ba 00 00 00 00    	mov    r10d,0x0
  5326f9:	41 0f 94 c2          	sete   r10b
  5326fd:	45 33 db             	xor    r11d,r11d
  532700:	40 80 fd 43          	cmp    bpl,0x43
  532704:	41 0f 94 c3          	sete   r11b
  532708:	45 3b d3             	cmp    r10d,r11d
  53270b:	75 31                	jne    53273e <PRIVATE0000000000627f3e+0x67e>
  53270d:	f2 0f 10 44 24 18    	movsd  xmm0,QWORD PTR [rsp+0x18]
  532713:	66 0f 2f c4          	comisd xmm0,xmm4
  532717:	76 25                	jbe    53273e <PRIVATE0000000000627f3e+0x67e>
  532719:	48 8b ac 24 80 00 00 	mov    rbp,QWORD PTR [rsp+0x80]
  532720:	00
  532721:	41 88 d6             	mov    r14b,dl
  532724:	44 89 8c 24 d8 00 00 	mov    DWORD PTR [rsp+0xd8],r9d
  53272b:	00
  53272c:	f2 0f 11 64 24 18    	movsd  QWORD PTR [rsp+0x18],xmm4
  532732:	41 8a 2c 29          	mov    bpl,BYTE PTR [r9+rbp*1]
  532736:	c7 44 24 40 05 00 00 	mov    DWORD PTR [rsp+0x40],0x5
  53273d:	00
  53273e:	66 0f 2f ac 24 88 00 	comisd xmm5,QWORD PTR [rsp+0x88]
  532745:	00 00
  532747:	76 7a                	jbe    5327c3 <PRIVATE0000000000627f3e+0x703>
  532749:	45 3a c6             	cmp    r8b,r14b
  53274c:	7f 75                	jg     5327c3 <PRIVATE0000000000627f3e+0x703>
  53274e:	83 bc 24 d8 00 00 00 	cmp    DWORD PTR [rsp+0xd8],0xffffffff
  532755:	ff
  532756:	74 47                	je     53279f <PRIVATE0000000000627f3e+0x6df>
  532758:	45 3a c6             	cmp    r8b,r14b
  53275b:	7c 42                	jl     53279f <PRIVATE0000000000627f3e+0x6df>
  53275d:	4c 8b 8c 24 80 00 00 	mov    r9,QWORD PTR [rsp+0x80]
  532764:	00
  532765:	46 8a 0c 08          	mov    r9b,BYTE PTR [rax+r9*1]
  532769:	41 80 f9 43          	cmp    r9b,0x43
  53276d:	74 0a                	je     532779 <PRIVATE0000000000627f3e+0x6b9>
  53276f:	40 80 fd 43          	cmp    bpl,0x43
  532773:	74 2a                	je     53279f <PRIVATE0000000000627f3e+0x6df>
  532775:	41 80 f9 43          	cmp    r9b,0x43
  532779:	41 b9 00 00 00 00    	mov    r9d,0x0
  53277f:	41 0f 94 c1          	sete   r9b
  532783:	45 33 d2             	xor    r10d,r10d
  532786:	40 80 fd 43          	cmp    bpl,0x43
  53278a:	41 0f 94 c2          	sete   r10b
  53278e:	45 3b ca             	cmp    r9d,r10d
  532791:	75 30                	jne    5327c3 <PRIVATE0000000000627f3e+0x703>
  532793:	f2 0f 10 44 24 18    	movsd  xmm0,QWORD PTR [rsp+0x18]
  532799:	66 0f 2f c5          	comisd xmm0,xmm5
  53279d:	76 24                	jbe    5327c3 <PRIVATE0000000000627f3e+0x703>
  53279f:	48 8b ac 24 80 00 00 	mov    rbp,QWORD PTR [rsp+0x80]
  5327a6:	00
  5327a7:	45 88 c6             	mov    r14b,r8b
  5327aa:	89 84 24 d8 00 00 00 	mov    DWORD PTR [rsp+0xd8],eax
  5327b1:	f2 0f 11 6c 24 18    	movsd  QWORD PTR [rsp+0x18],xmm5
  5327b7:	40 8a 2c 28          	mov    bpl,BYTE PTR [rax+rbp*1]
  5327bb:	c7 44 24 40 05 00 00 	mov    DWORD PTR [rsp+0x40],0x5
  5327c2:	00
  5327c3:	4d 8b 84 24 e8 00 00 	mov    r8,QWORD PTR [r12+0xe8]
  5327ca:	00
  5327cb:	41 ff c7             	inc    r15d
  5327ce:	45 3b b8 58 02 00 00 	cmp    r15d,DWORD PTR [r8+0x258]
  5327d5:	0f 8c 2c fe ff ff    	jl     532607 <PRIVATE0000000000627f3e+0x547>
  5327db:	8b 94 24 d8 00 00 00 	mov    edx,DWORD PTR [rsp+0xd8]
  5327e2:	f2 0f 10 44 24 50    	movsd  xmm0,QWORD PTR [rsp+0x50]
  5327e8:	f2 0f 10 64 24 30    	movsd  xmm4,QWORD PTR [rsp+0x30]
  5327ee:	8b 34 24             	mov    esi,DWORD PTR [rsp]
  5327f1:	8b 5c 24 60          	mov    ebx,DWORD PTR [rsp+0x60]
  5327f5:	8b 4c 24 40          	mov    ecx,DWORD PTR [rsp+0x40]
  5327f9:	4c 8b 74 24 10       	mov    r14,QWORD PTR [rsp+0x10]
  5327fe:	4c 8b 7c 24 08       	mov    r15,QWORD PTR [rsp+0x8]
  532803:	4c 8b 64 24 58       	mov    r12,QWORD PTR [rsp+0x58]
  532808:	48 8b ac 24 b0 00 00 	mov    rbp,QWORD PTR [rsp+0xb0]
  53280f:	00
  532810:	83 fa ff             	cmp    edx,0xffffffff
  532813:	0f 85 9b fa ff ff    	jne    5322b4 <PRIVATE0000000000627f3e+0x1f4>
  532819:	41 83 b8 a8 01 00 00 	cmp    DWORD PTR [r8+0x1a8],0x0
  532820:	00
  532821:	0f 8e ad 01 00 00    	jle    5329d4 <PRIVATE0000000000627f3e+0x914>
  532827:	49 8b 90 b0 01 00 00 	mov    rdx,QWORD PTR [r8+0x1b0]
  53282e:	41 b0 43             	mov    r8b,0x43
  532831:	f2 0f 10 15 df a0 53 	movsd  xmm2,QWORD PTR [rip+0x253a0df]        # 2a6c918 <coefclasssign+0x2198>
  532838:	02
  532839:	41 b1 01             	mov    r9b,0x1
  53283c:	33 c0                	xor    eax,eax
  53283e:	4c 89 74 24 10       	mov    QWORD PTR [rsp+0x10],r14
  532843:	f2 0f 10 0d 6d a0 53 	movsd  xmm1,QWORD PTR [rip+0x253a06d]        # 2a6c8b8 <coefclasssign+0x2138>
  53284a:	02
  53284b:	f2 0f 11 44 24 50    	movsd  QWORD PTR [rsp+0x50],xmm0
  532851:	f2 0f 11 54 24 18    	movsd  QWORD PTR [rsp+0x18],xmm2
  532857:	f2 0f 11 64 24 30    	movsd  QWORD PTR [rsp+0x30],xmm4
  53285d:	89 34 24             	mov    DWORD PTR [rsp],esi
  532860:	44 88 44 24 08       	mov    BYTE PTR [rsp+0x8],r8b
  532865:	44 88 4c 24 20       	mov    BYTE PTR [rsp+0x20],r9b
  53286a:	89 5c 24 60          	mov    DWORD PTR [rsp+0x60],ebx
  53286e:	89 4c 24 40          	mov    DWORD PTR [rsp+0x40],ecx
  532872:	4c 89 64 24 58       	mov    QWORD PTR [rsp+0x58],r12
  532877:	49 89 c4             	mov    r12,rax
  53287a:	48 89 ac 24 b0 00 00 	mov    QWORD PTR [rsp+0xb0],rbp
  532881:	00
  532882:	48 89 d5             	mov    rbp,rdx
  532885:	4c 8b 74 24 70       	mov    r14,QWORD PTR [rsp+0x70]
  53288a:	4a 8b 5c e5 00       	mov    rbx,QWORD PTR [rbp+r12*8+0x0]
  53288f:	4c 89 ff             	mov    rdi,r15
  532892:	48 89 de             	mov    rsi,rbx
  532895:	4c 89 ea             	mov    rdx,r13
  532898:	48 8b 4c 24 38       	mov    rcx,QWORD PTR [rsp+0x38]
  53289d:	4c 8b 44 24 68       	mov    r8,QWORD PTR [rsp+0x68]
  5328a2:	e8 a9 3d 41 00       	call   946650 <PRIVATE0000000000b3fb65>
  5328a7:	85 c0                	test   eax,eax
  5328a9:	0f 85 d2 00 00 00    	jne    532981 <PRIVATE0000000000627f3e+0x8c1>
  5328af:	48 89 df             	mov    rdi,rbx
  5328b2:	e8 c9 33 41 00       	call   945c80 <PRIVATE0000000000b3eb52>
  5328b7:	4c 63 c8             	movsxd r9,eax
  5328ba:	48 8b 4c 24 38       	mov    rcx,QWORD PTR [rsp+0x38]
  5328bf:	f2 42 0f 10 0c c9    	movsd  xmm1,QWORD PTR [rcx+r9*8]
  5328c5:	f2 43 0f 10 44 cd 00 	movsd  xmm0,QWORD PTR [r13+r9*8+0x0]
  5328cc:	0f 28 d1             	movaps xmm2,xmm1
  5328cf:	66 0f 2f 05 e1 9f 53 	comisd xmm0,QWORD PTR [rip+0x2539fe1]        # 2a6c8b8 <coefclasssign+0x2138>
  5328d6:	02
  5328d7:	f2 0f 5c d0          	subsd  xmm2,xmm0
  5328db:	73 0e                	jae    5328eb <PRIVATE0000000000627f3e+0x82b>
  5328dd:	f2 0f 10 05 cb 9f 53 	movsd  xmm0,QWORD PTR [rip+0x2539fcb]        # 2a6c8b0 <coefclasssign+0x2130>
  5328e4:	02
  5328e5:	66 0f 2f c1          	comisd xmm0,xmm1
  5328e9:	72 05                	jb     5328f0 <PRIVATE0000000000627f3e+0x830>
  5328eb:	41 b0 01             	mov    r8b,0x1
  5328ee:	eb 03                	jmp    5328f3 <PRIVATE0000000000627f3e+0x833>
  5328f0:	45 32 c0             	xor    r8b,r8b
  5328f3:	66 0f 2f 94 24 88 00 	comisd xmm2,QWORD PTR [rsp+0x88]
  5328fa:	00 00
  5328fc:	0f 86 7f 00 00 00    	jbe    532981 <PRIVATE0000000000627f3e+0x8c1>
  532902:	44 3a 44 24 20       	cmp    r8b,BYTE PTR [rsp+0x20]
  532907:	7f 78                	jg     532981 <PRIVATE0000000000627f3e+0x8c1>
  532909:	83 bc 24 d8 00 00 00 	cmp    DWORD PTR [rsp+0xd8],0xffffffff
  532910:	ff
  532911:	74 44                	je     532957 <PRIVATE0000000000627f3e+0x897>
  532913:	44 3a 44 24 20       	cmp    r8b,BYTE PTR [rsp+0x20]
  532918:	7c 3d                	jl     532957 <PRIVATE0000000000627f3e+0x897>
  53291a:	48 8b 8c 24 80 00 00 	mov    rcx,QWORD PTR [rsp+0x80]
  532921:	00
  532922:	41 8a 0c 09          	mov    cl,BYTE PTR [r9+rcx*1]
  532926:	80 f9 43             	cmp    cl,0x43
  532929:	74 0a                	je     532935 <PRIVATE0000000000627f3e+0x875>
  53292b:	80 7c 24 08 43       	cmp    BYTE PTR [rsp+0x8],0x43
  532930:	74 25                	je     532957 <PRIVATE0000000000627f3e+0x897>
  532932:	80 f9 43             	cmp    cl,0x43
  532935:	b9 00 00 00 00       	mov    ecx,0x0
  53293a:	0f 94 c1             	sete   cl
  53293d:	33 db                	xor    ebx,ebx
  53293f:	80 7c 24 08 43       	cmp    BYTE PTR [rsp+0x8],0x43
  532944:	0f 94 c3             	sete   bl
  532947:	3b cb                	cmp    ecx,ebx
  532949:	75 36                	jne    532981 <PRIVATE0000000000627f3e+0x8c1>
  53294b:	f2 0f 10 44 24 18    	movsd  xmm0,QWORD PTR [rsp+0x18]
  532951:	66 0f 2f c2          	comisd xmm0,xmm2
  532955:	76 2a                	jbe    532981 <PRIVATE0000000000627f3e+0x8c1>
  532957:	48 8b 8c 24 80 00 00 	mov    rcx,QWORD PTR [rsp+0x80]
  53295e:	00
  53295f:	89 84 24 d8 00 00 00 	mov    DWORD PTR [rsp+0xd8],eax
  532966:	f2 0f 11 54 24 18    	movsd  QWORD PTR [rsp+0x18],xmm2
  53296c:	41 8a 1c 09          	mov    bl,BYTE PTR [r9+rcx*1]
  532970:	88 5c 24 08          	mov    BYTE PTR [rsp+0x8],bl
  532974:	44 88 44 24 20       	mov    BYTE PTR [rsp+0x20],r8b
  532979:	c7 44 24 40 05 00 00 	mov    DWORD PTR [rsp+0x40],0x5
  532980:	00
  532981:	49 8b 8e e8 00 00 00 	mov    rcx,QWORD PTR [r14+0xe8]
  532988:	49 ff c4             	inc    r12
  53298b:	48 63 99 a8 01 00 00 	movsxd rbx,DWORD PTR [rcx+0x1a8]
  532992:	4c 3b e3             	cmp    r12,rbx
  532995:	0f 8c ef fe ff ff    	jl     53288a <PRIVATE0000000000627f3e+0x7ca>
  53299b:	8b 94 24 d8 00 00 00 	mov    edx,DWORD PTR [rsp+0xd8]
  5329a2:	f2 0f 10 44 24 50    	movsd  xmm0,QWORD PTR [rsp+0x50]
  5329a8:	f2 0f 10 64 24 30    	movsd  xmm4,QWORD PTR [rsp+0x30]
  5329ae:	8b 34 24             	mov    esi,DWORD PTR [rsp]
  5329b1:	8b 5c 24 60          	mov    ebx,DWORD PTR [rsp+0x60]
  5329b5:	8b 4c 24 40          	mov    ecx,DWORD PTR [rsp+0x40]
  5329b9:	4c 8b 74 24 10       	mov    r14,QWORD PTR [rsp+0x10]
  5329be:	4c 8b 64 24 58       	mov    r12,QWORD PTR [rsp+0x58]
  5329c3:	48 8b ac 24 b0 00 00 	mov    rbp,QWORD PTR [rsp+0xb0]
  5329ca:	00
  5329cb:	83 fa ff             	cmp    edx,0xffffffff
  5329ce:	0f 85 e0 f8 ff ff    	jne    5322b4 <PRIVATE0000000000627f3e+0x1f4>
  5329d4:	45 33 ff             	xor    r15d,r15d
  5329d7:	4d 85 f6             	test   r14,r14
  5329da:	0f 8e dc 01 00 00    	jle    532bbc <PRIVATE0000000000627f3e+0xafc>
  5329e0:	4c 89 64 24 58       	mov    QWORD PTR [rsp+0x58],r12
  5329e5:	41 89 ca             	mov    r10d,ecx
  5329e8:	48 89 ac 24 b0 00 00 	mov    QWORD PTR [rsp+0xb0],rbp
  5329ef:	00
  5329f0:	f2 0f 10 15 d0 9e 53 	movsd  xmm2,QWORD PTR [rip+0x2539ed0]        # 2a6c8c8 <coefclasssign+0x2148>
  5329f7:	02
  5329f8:	f2 0f 10 0d 20 9f 53 	movsd  xmm1,QWORD PTR [rip+0x2539f20]        # 2a6c920 <coefclasssign+0x21a0>
  5329ff:	02
  532a00:	f2 0f 10 1d 20 9f 53 	movsd  xmm3,QWORD PTR [rip+0x2539f20]        # 2a6c928 <coefclasssign+0x21a8>
  532a07:	02
  532a08:	f2 0f 11 44 24 50    	movsd  QWORD PTR [rsp+0x50],xmm0
  532a0e:	89 5c 24 60          	mov    DWORD PTR [rsp+0x60],ebx
  532a12:	89 f3                	mov    ebx,esi
  532a14:	44 8b 5c 24 78       	mov    r11d,DWORD PTR [rsp+0x78]
  532a19:	4c 8b 64 24 38       	mov    r12,QWORD PTR [rsp+0x38]
  532a1e:	48 8b ac 24 80 00 00 	mov    rbp,QWORD PTR [rsp+0x80]
  532a25:	00
  532a26:	f2 43 0f 10 3c fc    	movsd  xmm7,QWORD PTR [r12+r15*8]
  532a2c:	f2 43 0f 10 44 fd 00 	movsd  xmm0,QWORD PTR [r13+r15*8+0x0]
  532a33:	0f 28 f7             	movaps xmm6,xmm7
  532a36:	f2 0f 5c f0          	subsd  xmm6,xmm0
  532a3a:	66 0f 2f d6          	comisd xmm2,xmm6
  532a3e:	0f 87 f5 00 00 00    	ja     532b39 <PRIVATE0000000000627f3e+0xa79>
  532a44:	41 8a 04 2f          	mov    al,BYTE PTR [r15+rbp*1]
  532a48:	3c 43                	cmp    al,0x43
  532a4a:	0f 84 0a 03 00 00    	je     532d5a <PRIVATE0000000000627f3e+0xc9a>
  532a50:	3c 42                	cmp    al,0x42
  532a52:	0f 84 c9 01 00 00    	je     532c21 <PRIVATE0000000000627f3e+0xb61>
  532a58:	83 bc 24 d8 00 00 00 	cmp    DWORD PTR [rsp+0xd8],0x0
  532a5f:	00
  532a60:	7c 13                	jl     532a75 <PRIVATE0000000000627f3e+0x9b5>
  532a62:	66 0f ef ed          	pxor   xmm5,xmm5
  532a66:	f2 41 0f 2a eb       	cvtsi2sd xmm5,r11d
  532a6b:	66 0f 2f ee          	comisd xmm5,xmm6
  532a6f:	0f 86 c4 00 00 00    	jbe    532b39 <PRIVATE0000000000627f3e+0xa79>
  532a75:	44 89 bc 24 d8 00 00 	mov    DWORD PTR [rsp+0xd8],r15d
  532a7c:	00
  532a7d:	45 33 d2             	xor    r10d,r10d
  532a80:	66 0f 2f c8          	comisd xmm1,xmm0
  532a84:	72 13                	jb     532a99 <PRIVATE0000000000627f3e+0x9d9>
  532a86:	66 0f 2f fb          	comisd xmm7,xmm3
  532a8a:	0f 82 31 17 00 00    	jb     5341c1 <PRIVATE0000000000627f3e+0x2101>
  532a90:	66 0f ef e4          	pxor   xmm4,xmm4
  532a94:	e9 a0 00 00 00       	jmp    532b39 <PRIVATE0000000000627f3e+0xa79>
  532a99:	72 29                	jb     532ac4 <PRIVATE0000000000627f3e+0xa04>
  532a9b:	f2 0f 10 05 35 9e 53 	movsd  xmm0,QWORD PTR [rip+0x2539e35]        # 2a6c8d8 <coefclasssign+0x2158>
  532aa2:	02
  532aa3:	66 0f 2f f8          	comisd xmm7,xmm0
  532aa7:	76 0d                	jbe    532ab6 <PRIVATE0000000000627f3e+0x9f6>
  532aa9:	f2 0f 10 25 27 9e 53 	movsd  xmm4,QWORD PTR [rip+0x2539e27]        # 2a6c8d8 <coefclasssign+0x2158>
  532ab0:	02
  532ab1:	e9 83 00 00 00       	jmp    532b39 <PRIVATE0000000000627f3e+0xa79>
  532ab6:	f2 0f 10 25 4a 9e 53 	movsd  xmm4,QWORD PTR [rip+0x2539e4a]        # 2a6c908 <coefclasssign+0x2188>
  532abd:	02
  532abe:	f2 0f 59 e7          	mulsd  xmm4,xmm7
  532ac2:	eb 75                	jmp    532b39 <PRIVATE0000000000627f3e+0xa79>
  532ac4:	66 0f 2f fb          	comisd xmm7,xmm3
  532ac8:	72 26                	jb     532af0 <PRIVATE0000000000627f3e+0xa30>
  532aca:	f2 0f 10 25 fe 9d 53 	movsd  xmm4,QWORD PTR [rip+0x2539dfe]        # 2a6c8d0 <coefclasssign+0x2150>
  532ad1:	02
  532ad2:	66 0f 2f e0          	comisd xmm4,xmm0
  532ad6:	76 0a                	jbe    532ae2 <PRIVATE0000000000627f3e+0xa22>
  532ad8:	f2 0f 10 25 58 9e 53 	movsd  xmm4,QWORD PTR [rip+0x2539e58]        # 2a6c938 <coefclasssign+0x21b8>
  532adf:	02
  532ae0:	eb 57                	jmp    532b39 <PRIVATE0000000000627f3e+0xa79>
  532ae2:	f2 0f 10 25 1e 9e 53 	movsd  xmm4,QWORD PTR [rip+0x2539e1e]        # 2a6c908 <coefclasssign+0x2188>
  532ae9:	02
  532aea:	f2 0f 59 e0          	mulsd  xmm4,xmm0
  532aee:	eb 49                	jmp    532b39 <PRIVATE0000000000627f3e+0xa79>
  532af0:	f2 44 0f 2c de       	cvttsd2si r11d,xmm6
  532af5:	f2 0f 58 c7          	addsd  xmm0,xmm7
  532af9:	f2 0f 10 25 2f 9e 53 	movsd  xmm4,QWORD PTR [rip+0x2539e2f]        # 2a6c930 <coefclasssign+0x21b0>
  532b00:	02
  532b01:	f2 0f 59 c4          	mulsd  xmm0,xmm4
  532b05:	44 89 54 24 40       	mov    DWORD PTR [rsp+0x40],r10d
  532b0a:	44 89 5c 24 78       	mov    DWORD PTR [rsp+0x78],r11d
  532b0f:	e8 1c ac b8 ff       	call   bd730 <floor@plt>
  532b14:	44 8b 5c 24 78       	mov    r11d,DWORD PTR [rsp+0x78]
  532b19:	0f 28 e0             	movaps xmm4,xmm0
  532b1c:	44 8b 54 24 40       	mov    r10d,DWORD PTR [rsp+0x40]
  532b21:	f2 0f 10 15 9f 9d 53 	movsd  xmm2,QWORD PTR [rip+0x2539d9f]        # 2a6c8c8 <coefclasssign+0x2148>
  532b28:	02
  532b29:	f2 0f 10 0d ef 9d 53 	movsd  xmm1,QWORD PTR [rip+0x2539def]        # 2a6c920 <coefclasssign+0x21a0>
  532b30:	02
  532b31:	f2 0f 10 1d ef 9d 53 	movsd  xmm3,QWORD PTR [rip+0x2539def]        # 2a6c928 <coefclasssign+0x21a8>
  532b38:	02
  532b39:	49 ff c7             	inc    r15
  532b3c:	4d 3b fe             	cmp    r15,r14
  532b3f:	0f 8c e1 fe ff ff    	jl     532a26 <PRIVATE0000000000627f3e+0x966>
  532b45:	8b 94 24 d8 00 00 00 	mov    edx,DWORD PTR [rsp+0xd8]
  532b4c:	89 de                	mov    esi,ebx
  532b4e:	8b 5c 24 60          	mov    ebx,DWORD PTR [rsp+0x60]
  532b52:	44 89 d1             	mov    ecx,r10d
  532b55:	4c 8b 64 24 58       	mov    r12,QWORD PTR [rsp+0x58]
  532b5a:	48 8b ac 24 b0 00 00 	mov    rbp,QWORD PTR [rsp+0xb0]
  532b61:	00
  532b62:	83 fa ff             	cmp    edx,0xffffffff
  532b65:	0f 85 49 f7 ff ff    	jne    5322b4 <PRIVATE0000000000627f3e+0x1f4>
  532b6b:	85 f6                	test   esi,esi
  532b6d:	74 4d                	je     532bbc <PRIVATE0000000000627f3e+0xafc>
  532b6f:	48 8b 94 24 90 00 00 	mov    rdx,QWORD PTR [rsp+0x90]
  532b76:	00
  532b77:	48 8b 4a 18          	mov    rcx,QWORD PTR [rdx+0x18]
  532b7b:	83 79 28 01          	cmp    DWORD PTR [rcx+0x28],0x1
  532b7f:	0f 84 8a 00 00 00    	je     532c0f <PRIVATE0000000000627f3e+0xb4f>
  532b85:	4c 89 e6             	mov    rsi,r12
  532b88:	48 89 d7             	mov    rdi,rdx
  532b8b:	48 8b 94 24 c0 00 00 	mov    rdx,QWORD PTR [rsp+0xc0]
  532b92:	00
  532b93:	e8 58 4c fe ff       	call   5177f0 <PRIVATE00000000005ff358>
  532b98:	89 c3                	mov    ebx,eax
  532b9a:	85 db                	test   ebx,ebx
  532b9c:	0f 85 dc f8 ff ff    	jne    53247e <PRIVATE0000000000627f3e+0x3be>
  532ba2:	48 8b 94 24 c0 00 00 	mov    rdx,QWORD PTR [rsp+0xc0]
  532ba9:	00
  532baa:	be 0a 00 00 00       	mov    esi,0xa
  532baf:	48 8b 3a             	mov    rdi,QWORD PTR [rdx]
  532bb2:	e8 39 55 fe ff       	call   5180f0 <PRIVATE00000000005ff754>
  532bb7:	e9 c2 f8 ff ff       	jmp    53247e <PRIVATE0000000000627f3e+0x3be>
  532bbc:	4c 89 ee             	mov    rsi,r13
  532bbf:	48 8b 7c 24 70       	mov    rdi,QWORD PTR [rsp+0x70]
  532bc4:	e8 07 65 36 00       	call   8990d0 <PRIVATE0000000000a8f686>
  532bc9:	4c 89 ea             	mov    rdx,r13
  532bcc:	b9 01 00 00 00       	mov    ecx,0x1
  532bd1:	45 33 c0             	xor    r8d,r8d
  532bd4:	48 8b bc 24 90 00 00 	mov    rdi,QWORD PTR [rsp+0x90]
  532bdb:	00
  532bdc:	48 8b b4 24 d0 00 00 	mov    rsi,QWORD PTR [rsp+0xd0]
  532be3:	00
  532be4:	4c 8b 4c 24 68       	mov    r9,QWORD PTR [rsp+0x68]
  532be9:	e8 e2 d9 08 00       	call   5c05d0 <PRIVATE00000000006fb96f>
  532bee:	89 c3                	mov    ebx,eax
  532bf0:	85 db                	test   ebx,ebx
  532bf2:	0f 85 86 f8 ff ff    	jne    53247e <PRIVATE0000000000627f3e+0x3be>
  532bf8:	be 05 00 00 00       	mov    esi,0x5
  532bfd:	48 8b bc 24 90 00 00 	mov    rdi,QWORD PTR [rsp+0x90]
  532c04:	00
  532c05:	e8 e6 54 fe ff       	call   5180f0 <PRIVATE00000000005ff754>
  532c0a:	e9 6f f8 ff ff       	jmp    53247e <PRIVATE0000000000627f3e+0x3be>
  532c0f:	be 05 00 00 00       	mov    esi,0x5
  532c14:	48 89 d7             	mov    rdi,rdx
  532c17:	e8 d4 54 fe ff       	call   5180f0 <PRIVATE00000000005ff754>
  532c1c:	e9 5d f8 ff ff       	jmp    53247e <PRIVATE0000000000627f3e+0x3be>
  532c21:	44 89 fe             	mov    esi,r15d
  532c24:	48 8b 7c 24 48       	mov    rdi,QWORD PTR [rsp+0x48]
  532c29:	f2 0f 11 64 24 30    	movsd  QWORD PTR [rsp+0x30],xmm4
  532c2f:	44 89 54 24 40       	mov    DWORD PTR [rsp+0x40],r10d
  532c34:	44 89 5c 24 78       	mov    DWORD PTR [rsp+0x78],r11d
  532c39:	e8 42 4b 08 00       	call   5b7780 <PRIVATE00000000006eec90>
  532c3e:	44 8b 5c 24 78       	mov    r11d,DWORD PTR [rsp+0x78]
  532c43:	44 8b 54 24 40       	mov    r10d,DWORD PTR [rsp+0x40]
  532c48:	f2 0f 10 64 24 30    	movsd  xmm4,QWORD PTR [rsp+0x30]
  532c4e:	f2 0f 10 15 72 9c 53 	movsd  xmm2,QWORD PTR [rip+0x2539c72]        # 2a6c8c8 <coefclasssign+0x2148>
  532c55:	02
  532c56:	f2 0f 10 0d c2 9c 53 	movsd  xmm1,QWORD PTR [rip+0x2539cc2]        # 2a6c920 <coefclasssign+0x21a0>
  532c5d:	02
  532c5e:	f2 0f 10 1d c2 9c 53 	movsd  xmm3,QWORD PTR [rip+0x2539cc2]        # 2a6c928 <coefclasssign+0x21a8>
  532c65:	02
  532c66:	85 c0                	test   eax,eax
  532c68:	75 08                	jne    532c72 <PRIVATE0000000000627f3e+0xbb2>
  532c6a:	0f 28 ea             	movaps xmm5,xmm2
  532c6d:	e9 b4 00 00 00       	jmp    532d26 <PRIVATE0000000000627f3e+0xc66>
  532c72:	f2 0f 10 05 b6 9c 53 	movsd  xmm0,QWORD PTR [rip+0x2539cb6]        # 2a6c930 <coefclasssign+0x21b0>
  532c79:	02
  532c7a:	4c 8d 44 24 18       	lea    r8,[rsp+0x18]
  532c7f:	33 d2                	xor    edx,edx
  532c81:	4c 8d 4c 24 38       	lea    r9,[rsp+0x38]
  532c86:	49 8b b8 b8 00 00 00 	mov    rdi,QWORD PTR [r8+0xb8]
  532c8d:	49 8b 70 30          	mov    rsi,QWORD PTR [r8+0x30]
  532c91:	44 89 f9             	mov    ecx,r15d
  532c94:	f2 41 0f 11 60 18    	movsd  QWORD PTR [r8+0x18],xmm4
  532c9a:	45 89 50 28          	mov    DWORD PTR [r8+0x28],r10d
  532c9e:	45 89 58 60          	mov    DWORD PTR [r8+0x60],r11d
  532ca2:	e8 79 4c 08 00       	call   5b7920 <PRIVATE00000000006ef12f>
  532ca7:	f2 0f 10 15 19 9c 53 	movsd  xmm2,QWORD PTR [rip+0x2539c19]        # 2a6c8c8 <coefclasssign+0x2148>
  532cae:	02
  532caf:	f2 0f 10 44 24 18    	movsd  xmm0,QWORD PTR [rsp+0x18]
  532cb5:	f2 0f 10 7c 24 38    	movsd  xmm7,QWORD PTR [rsp+0x38]
  532cbb:	f2 0f 5f c2          	maxsd  xmm0,xmm2
  532cbf:	f2 0f 5f fa          	maxsd  xmm7,xmm2
  532cc3:	f2 0f 10 2d 75 9c 53 	movsd  xmm5,QWORD PTR [rip+0x2539c75]        # 2a6c940 <coefclasssign+0x21c0>
  532cca:	02
  532ccb:	0f 28 f5             	movaps xmm6,xmm5
  532cce:	f2 0f 58 c5          	addsd  xmm0,xmm5
  532cd2:	f2 0f 58 fe          	addsd  xmm7,xmm6
  532cd6:	f2 0f 59 c7          	mulsd  xmm0,xmm7
  532cda:	f2 0f 10 0d 3e 9c 53 	movsd  xmm1,QWORD PTR [rip+0x2539c3e]        # 2a6c920 <coefclasssign+0x21a0>
  532ce1:	02
  532ce2:	f2 0f 10 1d 3e 9c 53 	movsd  xmm3,QWORD PTR [rip+0x2539c3e]        # 2a6c928 <coefclasssign+0x21a8>
  532ce9:	02
  532cea:	e8 71 9d b8 ff       	call   bca60 <sqrt@plt>
  532cef:	44 8b 5c 24 78       	mov    r11d,DWORD PTR [rsp+0x78]
  532cf4:	0f 28 e8             	movaps xmm5,xmm0
  532cf7:	44 8b 54 24 40       	mov    r10d,DWORD PTR [rsp+0x40]
  532cfc:	f2 0f 10 64 24 30    	movsd  xmm4,QWORD PTR [rsp+0x30]
  532d02:	f2 0f 10 15 be 9b 53 	movsd  xmm2,QWORD PTR [rip+0x2539bbe]        # 2a6c8c8 <coefclasssign+0x2148>
  532d09:	02
  532d0a:	f2 0f 10 0d 0e 9c 53 	movsd  xmm1,QWORD PTR [rip+0x2539c0e]        # 2a6c920 <coefclasssign+0x21a0>
  532d11:	02
  532d12:	f2 0f 10 1d 0e 9c 53 	movsd  xmm3,QWORD PTR [rip+0x2539c0e]        # 2a6c928 <coefclasssign+0x21a8>
  532d19:	02
  532d1a:	f2 0f 10 05 1e 9c 53 	movsd  xmm0,QWORD PTR [rip+0x2539c1e]        # 2a6c940 <coefclasssign+0x21c0>
  532d21:	02
  532d22:	f2 0f 5c e8          	subsd  xmm5,xmm0
  532d26:	41 83 fb 01          	cmp    r11d,0x1
  532d2a:	7f 0c                	jg     532d38 <PRIVATE0000000000627f3e+0xc78>
  532d2c:	66 0f 2f 6c 24 50    	comisd xmm5,QWORD PTR [rsp+0x50]
  532d32:	0f 86 01 fe ff ff    	jbe    532b39 <PRIVATE0000000000627f3e+0xa79>
  532d38:	44 89 f8             	mov    eax,r15d
  532d3b:	41 bb 01 00 00 00    	mov    r11d,0x1
  532d41:	f2 0f 11 6c 24 50    	movsd  QWORD PTR [rsp+0x50],xmm5
  532d47:	66 0f ef e4          	pxor   xmm4,xmm4
  532d4b:	89 84 24 d8 00 00 00 	mov    DWORD PTR [rsp+0xd8],eax
  532d52:	45 33 d2             	xor    r10d,r10d
  532d55:	e9 df fd ff ff       	jmp    532b39 <PRIVATE0000000000627f3e+0xa79>
  532d5a:	ff c3                	inc    ebx
  532d5c:	e9 d8 fd ff ff       	jmp    532b39 <PRIVATE0000000000627f3e+0xa79>
  532d61:	4c 8b 44 24 70       	mov    r8,QWORD PTR [rsp+0x70]
  532d66:	44 89 fa             	mov    edx,r15d
  532d69:	89 94 24 d8 00 00 00 	mov    DWORD PTR [rsp+0xd8],edx
  532d70:	89 de                	mov    esi,ebx
  532d72:	48 63 d2             	movsxd rdx,edx
  532d75:	44 89 f3             	mov    ebx,r14d
  532d78:	4d 8b 80 e8 00 00 00 	mov    r8,QWORD PTR [r8+0xe8]
  532d7f:	f2 0f 10 44 24 50    	movsd  xmm0,QWORD PTR [rsp+0x50]
  532d85:	f2 0f 10 64 24 30    	movsd  xmm4,QWORD PTR [rsp+0x30]
  532d8b:	49 8b 88 50 01 00 00 	mov    rcx,QWORD PTR [r8+0x150]
  532d92:	4c 8b 74 24 10       	mov    r14,QWORD PTR [rsp+0x10]
  532d97:	4c 8b 7c 24 08       	mov    r15,QWORD PTR [rsp+0x8]
  532d9c:	4c 8b 64 24 58       	mov    r12,QWORD PTR [rsp+0x58]
  532da1:	48 8b ac 24 b0 00 00 	mov    rbp,QWORD PTR [rsp+0xb0]
  532da8:	00
  532da9:	8b 0c 91             	mov    ecx,DWORD PTR [rcx+rdx*4]
  532dac:	e9 c9 f7 ff ff       	jmp    53257a <PRIVATE0000000000627f3e+0x4ba>
  532db1:	4c 8b 84 24 90 00 00 	mov    r8,QWORD PTR [rsp+0x90]
  532db8:	00
  532db9:	4d 8b 48 18          	mov    r9,QWORD PTR [r8+0x18]
  532dbd:	4d 8b 41 08          	mov    r8,QWORD PTR [r9+0x8]
  532dc1:	4d 85 c0             	test   r8,r8
  532dc4:	0f 84 79 f4 ff ff    	je     532243 <PRIVATE0000000000627f3e+0x183>
  532dca:	4d 8b 48 18          	mov    r9,QWORD PTR [r8+0x18]
  532dce:	4d 85 c9             	test   r9,r9
  532dd1:	0f 84 6c f4 ff ff    	je     532243 <PRIVATE0000000000627f3e+0x183>
  532dd7:	45 33 c0             	xor    r8d,r8d
  532dda:	4d 85 f6             	test   r14,r14
  532ddd:	0f 8e 60 f4 ff ff    	jle    532243 <PRIVATE0000000000627f3e+0x183>
  532de3:	89 5c 24 60          	mov    DWORD PTR [rsp+0x60],ebx
  532de7:	48 89 ac 24 b0 00 00 	mov    QWORD PTR [rsp+0xb0],rbp
  532dee:	00
  532def:	f2 0f 10 35 d1 9a 53 	movsd  xmm6,QWORD PTR [rip+0x2539ad1]        # 2a6c8c8 <coefclasssign+0x2148>
  532df6:	02
  532df7:	f2 0f 10 2d d1 9a 53 	movsd  xmm5,QWORD PTR [rip+0x2539ad1]        # 2a6c8d0 <coefclasssign+0x2150>
  532dfe:	02
  532dff:	f2 0f 10 1d d1 9a 53 	movsd  xmm3,QWORD PTR [rip+0x2539ad1]        # 2a6c8d8 <coefclasssign+0x2158>
  532e06:	02
  532e07:	f2 0f 10 15 d1 9a 53 	movsd  xmm2,QWORD PTR [rip+0x2539ad1]        # 2a6c8e0 <coefclasssign+0x2160>
  532e0e:	02
  532e0f:	89 34 24             	mov    DWORD PTR [rsp],esi
  532e12:	4c 89 7c 24 08       	mov    QWORD PTR [rsp+0x8],r15
  532e17:	4c 89 64 24 58       	mov    QWORD PTR [rsp+0x58],r12
  532e1c:	4c 8b 54 24 38       	mov    r10,QWORD PTR [rsp+0x38]
  532e21:	48 8b bc 24 80 00 00 	mov    rdi,QWORD PTR [rsp+0x80]
  532e28:	00
  532e29:	48 8b 6c 24 18       	mov    rbp,QWORD PTR [rsp+0x18]
  532e2e:	48 8b 5c 24 70       	mov    rbx,QWORD PTR [rsp+0x70]
  532e33:	f2 47 0f 10 04 c1    	movsd  xmm8,QWORD PTR [r9+r8*8]
  532e39:	66 0f ef c9          	pxor   xmm1,xmm1
  532e3d:	66 44 0f 2f c6       	comisd xmm8,xmm6
  532e42:	76 0c                	jbe    532e50 <PRIVATE0000000000627f3e+0xd90>
  532e44:	f2 43 0f 10 3c c2    	movsd  xmm7,QWORD PTR [r10+r8*8]
  532e4a:	66 0f 2f fd          	comisd xmm7,xmm5
  532e4e:	73 14                	jae    532e64 <PRIVATE0000000000627f3e+0xda4>
  532e50:	66 41 0f 2f d0       	comisd xmm2,xmm8
  532e55:	76 15                	jbe    532e6c <PRIVATE0000000000627f3e+0xdac>
  532e57:	f2 43 0f 10 7c c5 00 	movsd  xmm7,QWORD PTR [r13+r8*8+0x0]
  532e5e:	66 0f 2f df          	comisd xmm3,xmm7
  532e62:	72 08                	jb     532e6c <PRIVATE0000000000627f3e+0xdac>
  532e64:	41 0f 28 c8          	movaps xmm1,xmm8
  532e68:	f2 0f 59 cf          	mulsd  xmm1,xmm7
  532e6c:	66 0f 2f c8          	comisd xmm1,xmm0
  532e70:	0f 86 ae 00 00 00    	jbe    532f24 <PRIVATE0000000000627f3e+0xe64>
  532e76:	41 80 3c 38 43       	cmp    BYTE PTR [r8+rdi*1],0x43
  532e7b:	74 0f                	je     532e8c <PRIVATE0000000000627f3e+0xdcc>
  532e7d:	44 89 c2             	mov    edx,r8d
  532e80:	33 c9                	xor    ecx,ecx
  532e82:	89 94 24 d8 00 00 00 	mov    DWORD PTR [rsp+0xd8],edx
  532e89:	0f 28 c1             	movaps xmm0,xmm1
  532e8c:	48 63 d2             	movsxd rdx,edx
  532e8f:	49 3b d0             	cmp    rdx,r8
  532e92:	0f 84 8c 00 00 00    	je     532f24 <PRIVATE0000000000627f3e+0xe64>
  532e98:	4c 8b 9b e8 00 00 00 	mov    r11,QWORD PTR [rbx+0xe8]
  532e9f:	41 83 bb 58 02 00 00 	cmp    DWORD PTR [r11+0x258],0x0
  532ea6:	00
  532ea7:	7e 51                	jle    532efa <PRIVATE0000000000627f3e+0xe3a>
  532ea9:	48 8b 44 24 28       	mov    rax,QWORD PTR [rsp+0x28]
  532eae:	46 8b 24 80          	mov    r12d,DWORD PTR [rax+r8*4]
  532eb2:	42 8b 44 80 04       	mov    eax,DWORD PTR [rax+r8*4+0x4]
  532eb7:	44 3b e0             	cmp    r12d,eax
  532eba:	7d 3e                	jge    532efa <PRIVATE0000000000627f3e+0xe3a>
  532ebc:	4d 63 e4             	movsxd r12,r12d
  532ebf:	48 8b 74 24 20       	mov    rsi,QWORD PTR [rsp+0x20]
  532ec4:	44 8b 7c 24 30       	mov    r15d,DWORD PTR [rsp+0x30]
  532ec9:	46 3b 3c a6          	cmp    r15d,DWORD PTR [rsi+r12*4]
  532ecd:	7e 11                	jle    532ee0 <PRIVATE0000000000627f3e+0xe20>
  532ecf:	41 ff c4             	inc    r12d
  532ed2:	44 3b e0             	cmp    r12d,eax
  532ed5:	7d 23                	jge    532efa <PRIVATE0000000000627f3e+0xe3a>
  532ed7:	4d 63 e4             	movsxd r12,r12d
  532eda:	46 3b 3c a6          	cmp    r15d,DWORD PTR [rsi+r12*4]
  532ede:	7f ef                	jg     532ecf <PRIVATE0000000000627f3e+0xe0f>
  532ee0:	49 63 c0             	movsxd rax,r8d
  532ee3:	44 89 c2             	mov    edx,r8d
  532ee6:	89 94 24 d8 00 00 00 	mov    DWORD PTR [rsp+0xd8],edx
  532eed:	b9 05 00 00 00       	mov    ecx,0x5
  532ef2:	0f 28 c1             	movaps xmm0,xmm1
  532ef5:	49 3b c0             	cmp    rax,r8
  532ef8:	74 2a                	je     532f24 <PRIVATE0000000000627f3e+0xe64>
  532efa:	41 83 bb a8 01 00 00 	cmp    DWORD PTR [r11+0x1a8],0x0
  532f01:	00
  532f02:	7e 20                	jle    532f24 <PRIVATE0000000000627f3e+0xe64>
  532f04:	42 8b 44 85 04       	mov    eax,DWORD PTR [rbp+r8*4+0x4]
  532f09:	42 2b 44 85 00       	sub    eax,DWORD PTR [rbp+r8*4+0x0]
  532f0e:	85 c0                	test   eax,eax
  532f10:	7e 45                	jle    532f57 <PRIVATE0000000000627f3e+0xe97>
  532f12:	44 89 c2             	mov    edx,r8d
  532f15:	b9 05 00 00 00       	mov    ecx,0x5
  532f1a:	89 94 24 d8 00 00 00 	mov    DWORD PTR [rsp+0xd8],edx
  532f21:	0f 28 c1             	movaps xmm0,xmm1
  532f24:	49 ff c0             	inc    r8
  532f27:	4d 3b c6             	cmp    r8,r14
  532f2a:	0f 8c 03 ff ff ff    	jl     532e33 <PRIVATE0000000000627f3e+0xd73>
  532f30:	8b 34 24             	mov    esi,DWORD PTR [rsp]
  532f33:	8b 5c 24 60          	mov    ebx,DWORD PTR [rsp+0x60]
  532f37:	4c 8b 7c 24 08       	mov    r15,QWORD PTR [rsp+0x8]
  532f3c:	4c 8b 64 24 58       	mov    r12,QWORD PTR [rsp+0x58]
  532f41:	48 8b ac 24 b0 00 00 	mov    rbp,QWORD PTR [rsp+0xb0]
  532f48:	00
  532f49:	83 fa ff             	cmp    edx,0xffffffff
  532f4c:	0f 84 f1 f2 ff ff    	je     532243 <PRIVATE0000000000627f3e+0x183>
  532f52:	e9 5d f3 ff ff       	jmp    5322b4 <PRIVATE0000000000627f3e+0x1f4>
  532f57:	48 8b 44 24 10       	mov    rax,QWORD PTR [rsp+0x10]
  532f5c:	46 8b 5c 80 04       	mov    r11d,DWORD PTR [rax+r8*4+0x4]
  532f61:	46 2b 1c 80          	sub    r11d,DWORD PTR [rax+r8*4]
  532f65:	45 85 db             	test   r11d,r11d
  532f68:	7e ba                	jle    532f24 <PRIVATE0000000000627f3e+0xe64>
  532f6a:	eb a6                	jmp    532f12 <PRIVATE0000000000627f3e+0xe52>
