
libgurobi130.so:     file format elf64-x86-64


Disassembly of section .text:

0000000000518170 <PRIVATE0000000000600d25>:
  518170:	41 54                	push   r12
  518172:	41 55                	push   r13
  518174:	41 56                	push   r14
  518176:	41 57                	push   r15
  518178:	53                   	push   rbx
  518179:	55                   	push   rbp
  51817a:	48 81 ec c8 00 00 00 	sub    rsp,0xc8
  518181:	49 89 ff             	mov    r15,rdi
  518184:	44 8b 4e 38          	mov    r9d,DWORD PTR [rsi+0x38]
  518188:	48 89 cb             	mov    rbx,rcx
  51818b:	49 89 d4             	mov    r12,rdx
  51818e:	45 33 ed             	xor    r13d,r13d
  518191:	4d 8b 57 18          	mov    r10,QWORD PTR [r15+0x18]
  518195:	49 8b 2a             	mov    rbp,QWORD PTR [r10]
  518198:	41 83 f9 02          	cmp    r9d,0x2
  51819c:	0f 84 32 19 00 00    	je     519ad4 <PRIVATE0000000000600d25+0x1964>
  5181a2:	41 8d 49 fe          	lea    ecx,[r9-0x2]
  5181a6:	83 f9 0f             	cmp    ecx,0xf
  5181a9:	77 7a                	ja     518225 <PRIVATE0000000000600d25+0xb5>
  5181ab:	48 8d 15 8e 42 55 02 	lea    rdx,[rip+0x255428e]        # 2a6c440 <coefclasssign+0x1cc0>
  5181b2:	0f b6 14 0a          	movzx  edx,BYTE PTR [rdx+rcx*1]
  5181b6:	48 8d 05 05 00 00 00 	lea    rax,[rip+0x5]        # 5181c2 <PRIVATE0000000000600d25+0x52>
  5181bd:	48 03 c2             	add    rax,rdx
  5181c0:	ff e0                	jmp    rax
  5181c2:	41 c7 42 28 07 00 00 	mov    DWORD PTR [r10+0x28],0x7
  5181c9:	00
  5181ca:	e9 9d 00 00 00       	jmp    51826c <PRIVATE0000000000600d25+0xfc>
  5181cf:	41 c7 42 28 05 00 00 	mov    DWORD PTR [r10+0x28],0x5
  5181d6:	00
  5181d7:	48 3b ad 28 06 00 00 	cmp    rbp,QWORD PTR [rbp+0x628]
  5181de:	0f 84 9d 00 00 00    	je     518281 <PRIVATE0000000000600d25+0x111>
  5181e4:	e9 83 00 00 00       	jmp    51826c <PRIVATE0000000000600d25+0xfc>
  5181e9:	8b 95 a8 00 00 00    	mov    edx,DWORD PTR [rbp+0xa8]
  5181ef:	83 fa 05             	cmp    edx,0x5
  5181f2:	0f 84 dd 00 00 00    	je     5182d5 <PRIVATE0000000000600d25+0x165>
  5181f8:	eb 37                	jmp    518231 <PRIVATE0000000000600d25+0xc1>
  5181fa:	41 c7 42 28 06 00 00 	mov    DWORD PTR [r10+0x28],0x6
  518201:	00
  518202:	eb 68                	jmp    51826c <PRIVATE0000000000600d25+0xfc>
  518204:	41 c7 42 28 04 00 00 	mov    DWORD PTR [r10+0x28],0x4
  51820b:	00
  51820c:	48 3b ad 28 06 00 00 	cmp    rbp,QWORD PTR [rbp+0x628]
  518213:	0f 84 60 18 00 00    	je     519a79 <PRIVATE0000000000600d25+0x1909>
  518219:	eb 51                	jmp    51826c <PRIVATE0000000000600d25+0xfc>
  51821b:	41 c7 42 28 02 00 00 	mov    DWORD PTR [r10+0x28],0x2
  518222:	00
  518223:	eb 47                	jmp    51826c <PRIVATE0000000000600d25+0xfc>
  518225:	41 83 f9 07          	cmp    r9d,0x7
  518229:	74 2c                	je     518257 <PRIVATE0000000000600d25+0xe7>
  51822b:	8b 95 a8 00 00 00    	mov    edx,DWORD PTR [rbp+0xa8]
  518231:	83 fa 02             	cmp    edx,0x2
  518234:	75 21                	jne    518257 <PRIVATE0000000000600d25+0xe7>
  518236:	41 83 e1 fe          	and    r9d,0xfffffffe
  51823a:	41 c7 42 28 03 00 00 	mov    DWORD PTR [r10+0x28],0x3
  518241:	00
  518242:	41 83 f9 0c          	cmp    r9d,0xc
  518246:	75 24                	jne    51826c <PRIVATE0000000000600d25+0xfc>
  518248:	48 8b 95 20 06 00 00 	mov    rdx,QWORD PTR [rbp+0x620]
  51824f:	ff 82 d4 00 00 00    	inc    DWORD PTR [rdx+0xd4]
  518255:	eb 15                	jmp    51826c <PRIVATE0000000000600d25+0xfc>
  518257:	48 8b 95 20 06 00 00 	mov    rdx,QWORD PTR [rbp+0x620]
  51825e:	41 c7 42 28 03 00 00 	mov    DWORD PTR [r10+0x28],0x3
  518265:	00
  518266:	ff 82 d4 00 00 00    	inc    DWORD PTR [rdx+0xd4]
  51826c:	44 89 e8             	mov    eax,r13d
  51826f:	48 81 c4 c8 00 00 00 	add    rsp,0xc8
  518276:	5d                   	pop    rbp
  518277:	5b                   	pop    rbx
  518278:	41 5f                	pop    r15
  51827a:	41 5e                	pop    r14
  51827c:	41 5d                	pop    r13
  51827e:	41 5c                	pop    r12
  518280:	c3                   	ret
  518281:	4c 3b bd 90 07 00 00 	cmp    r15,QWORD PTR [rbp+0x790]
  518288:	75 e2                	jne    51826c <PRIVATE0000000000600d25+0xfc>
  51828a:	48 8b 9d 20 06 00 00 	mov    rbx,QWORD PTR [rbp+0x620]
  518291:	48 89 df             	mov    rdi,rbx
  518294:	e8 57 1d ff ff       	call   509ff0 <PRIVATE00000000005e7796>
  518299:	0f 28 c8             	movaps xmm1,xmm0
  51829c:	f2 0f 10 05 fc 41 55 	movsd  xmm0,QWORD PTR [rip+0x25541fc]        # 2a6c4a0 <coefclasssign+0x1d20>
  5182a3:	02
  5182a4:	66 0f 2f c1          	comisd xmm0,xmm1
  5182a8:	76 08                	jbe    5182b2 <PRIVATE0000000000600d25+0x142>
  5182aa:	c7 03 02 00 00 00    	mov    DWORD PTR [rbx],0x2
  5182b0:	eb 06                	jmp    5182b8 <PRIVATE0000000000600d25+0x148>
  5182b2:	c7 03 03 00 00 00    	mov    DWORD PTR [rbx],0x3
  5182b8:	48 8b 93 80 07 00 00 	mov    rdx,QWORD PTR [rbx+0x780]
  5182bf:	f2 0f 10 82 68 01 00 	movsd  xmm0,QWORD PTR [rdx+0x168]
  5182c6:	00
  5182c7:	f2 0f 5f c1          	maxsd  xmm0,xmm1
  5182cb:	f2 0f 11 82 68 01 00 	movsd  QWORD PTR [rdx+0x168],xmm0
  5182d2:	00
  5182d3:	eb 97                	jmp    51826c <PRIVATE0000000000600d25+0xfc>
  5182d5:	48 8b 55 08          	mov    rdx,QWORD PTR [rbp+0x8]
  5182d9:	48 8b 92 e8 00 00 00 	mov    rdx,QWORD PTR [rdx+0xe8]
  5182e0:	83 7a 18 00          	cmp    DWORD PTR [rdx+0x18],0x0
  5182e4:	0f 8e 66 17 00 00    	jle    519a50 <PRIVATE0000000000600d25+0x18e0>
  5182ea:	4d 8b 74 24 08       	mov    r14,QWORD PTR [r12+0x8]
  5182ef:	4c 89 f7             	mov    rdi,r14
  5182f2:	48 c7 84 24 c0 00 00 	mov    QWORD PTR [rsp+0xc0],0x0
  5182f9:	00 00 00 00 00
  5182fe:	48 8d b4 24 c0 00 00 	lea    rsi,[rsp+0xc0]
  518305:	00
  518306:	e8 15 d0 2a 00       	call   7c5320 <PRIVATE000000000099b471>
  51830b:	41 89 c5             	mov    r13d,eax
  51830e:	45 85 ed             	test   r13d,r13d
  518311:	0f 85 9a 03 00 00    	jne    5186b1 <PRIVATE0000000000600d25+0x541>
  518317:	48 8b bc 24 c0 00 00 	mov    rdi,QWORD PTR [rsp+0xc0]
  51831e:	00
  51831f:	f2 0f 10 05 79 41 55 	movsd  xmm0,QWORD PTR [rip+0x2554179]        # 2a6c4a0 <coefclasssign+0x1d20>
  518326:	02
  518327:	48 8b 87 00 01 00 00 	mov    rax,QWORD PTR [rdi+0x100]
  51832e:	c7 80 24 24 00 00 00 	mov    DWORD PTR [rax+0x2424],0x0
  518335:	00 00 00
  518338:	48 8b 97 00 01 00 00 	mov    rdx,QWORD PTR [rdi+0x100]
  51833f:	f2 0f 11 82 a8 24 00 	movsd  QWORD PTR [rdx+0x24a8],xmm0
  518346:	00
  518347:	4d 8b 76 50          	mov    r14,QWORD PTR [r14+0x50]
  51834b:	4c 89 77 50          	mov    QWORD PTR [rdi+0x50],r14
  51834f:	e8 fc 5a 37 00       	call   88de50 <PRIVATE0000000000a7f923>
  518354:	85 c0                	test   eax,eax
  518356:	75 3d                	jne    518395 <PRIVATE0000000000600d25+0x225>
  518358:	48 8b bc 24 c0 00 00 	mov    rdi,QWORD PTR [rsp+0xc0]
  51835f:	00
  518360:	48 8b 87 00 01 00 00 	mov    rax,QWORD PTR [rdi+0x100]
  518367:	c7 80 08 27 00 00 01 	mov    DWORD PTR [rax+0x2708],0x1
  51836e:	00 00 00
  518371:	48 8b 87 00 01 00 00 	mov    rax,QWORD PTR [rdi+0x100]
  518378:	83 b8 38 24 00 00 01 	cmp    DWORD PTR [rax+0x2438],0x1
  51837f:	7e 0a                	jle    51838b <PRIVATE0000000000600d25+0x21b>
  518381:	c7 80 38 24 00 00 00 	mov    DWORD PTR [rax+0x2438],0x0
  518388:	00 00 00
  51838b:	48 89 de             	mov    rsi,rbx
  51838e:	e8 1d 29 c3 ff       	call   14acb0 <PRIVATE00000000000c3d0d>
  518393:	eb 10                	jmp    5183a5 <PRIVATE0000000000600d25+0x235>
  518395:	48 89 de             	mov    rsi,rbx
  518398:	48 8b bc 24 c0 00 00 	mov    rdi,QWORD PTR [rsp+0xc0]
  51839f:	00
  5183a0:	e8 0b 56 dc ff       	call   2dd9b0 <PRIVATE00000000002bb770>
  5183a5:	41 89 c5             	mov    r13d,eax
  5183a8:	45 85 ed             	test   r13d,r13d
  5183ab:	0f 85 00 03 00 00    	jne    5186b1 <PRIVATE0000000000600d25+0x541>
  5183b1:	48 8b 84 24 c0 00 00 	mov    rax,QWORD PTR [rsp+0xc0]
  5183b8:	00
  5183b9:	44 8b 70 64          	mov    r14d,DWORD PTR [rax+0x64]
  5183bd:	41 83 fe 02          	cmp    r14d,0x2
  5183c1:	75 08                	jne    5183cb <PRIVATE0000000000600d25+0x25b>
  5183c3:	41 be 05 00 00 00    	mov    r14d,0x5
  5183c9:	eb 0d                	jmp    5183d8 <PRIVATE0000000000600d25+0x268>
  5183cb:	b8 03 00 00 00       	mov    eax,0x3
  5183d0:	41 83 fe 04          	cmp    r14d,0x4
  5183d4:	44 0f 44 f0          	cmove  r14d,eax
  5183d8:	48 8d bc 24 c0 00 00 	lea    rdi,[rsp+0xc0]
  5183df:	00
  5183e0:	e8 4b b8 bf ff       	call   113c30 <PRIVATE0000000000088c65>
  5183e5:	45 33 ed             	xor    r13d,r13d
  5183e8:	41 83 fe 03          	cmp    r14d,0x3
  5183ec:	0f 84 9f 02 00 00    	je     518691 <PRIVATE0000000000600d25+0x521>
  5183f2:	41 83 fe 06          	cmp    r14d,0x6
  5183f6:	75 20                	jne    518418 <PRIVATE0000000000600d25+0x2a8>
  5183f8:	49 8b 57 18          	mov    rdx,QWORD PTR [r15+0x18]
  5183fc:	c7 42 28 05 00 00 00 	mov    DWORD PTR [rdx+0x28],0x5
  518403:	48 8b 12             	mov    rdx,QWORD PTR [rdx]
  518406:	48 3b 92 28 06 00 00 	cmp    rdx,QWORD PTR [rdx+0x628]
  51840d:	0f 84 b0 02 00 00    	je     5186c3 <PRIVATE0000000000600d25+0x553>
  518413:	e9 54 fe ff ff       	jmp    51826c <PRIVATE0000000000600d25+0xfc>
  518418:	41 83 fe 05          	cmp    r14d,0x5
  51841c:	0f 85 52 02 00 00    	jne    518674 <PRIVATE0000000000600d25+0x504>
  518422:	4c 8b 75 08          	mov    r14,QWORD PTR [rbp+0x8]
  518426:	49 8b 57 18          	mov    rdx,QWORD PTR [r15+0x18]
  51842a:	4d 8b 9e e8 00 00 00 	mov    r11,QWORD PTR [r14+0xe8]
  518431:	48 8b ba 80 00 00 00 	mov    rdi,QWORD PTR [rdx+0x80]
  518438:	4c 8b 82 88 00 00 00 	mov    r8,QWORD PTR [rdx+0x88]
  51843f:	4d 8b 93 b0 01 00 00 	mov    r10,QWORD PTR [r11+0x1b0]
  518446:	49 63 53 0c          	movsxd rdx,DWORD PTR [r11+0xc]
  51844a:	41 8b 43 18          	mov    eax,DWORD PTR [r11+0x18]
  51844e:	41 8b 8b 58 02 00 00 	mov    ecx,DWORD PTR [r11+0x258]
  518455:	41 8b b3 a8 01 00 00 	mov    esi,DWORD PTR [r11+0x1a8]
  51845c:	4d 8b 8e 00 01 00 00 	mov    r9,QWORD PTR [r14+0x100]
  518463:	4c 89 54 24 18       	mov    QWORD PTR [rsp+0x18],r10


libgurobi130.so:     file format elf64-x86-64


Disassembly of section .text:

0000000000518468 <PRIVATE0000000000600d25+0x2f8>:
  518468:	45 33 d2             	xor    r10d,r10d
  51846b:	4d 8b 9b 10 04 00 00 	mov    r11,QWORD PTR [r11+0x410]
  518472:	48 85 d2             	test   rdx,rdx
  518475:	0f 8e 6c 15 00 00    	jle    5199e7 <PRIVATE0000000000600d25+0x1877>
  51847b:	f2 0f 10 05 b5 40 55 	movsd  xmm0,QWORD PTR [rip+0x25540b5]        # 2a6c538 <coefclasssign+0x1db8>
  518482:	02
  518483:	43 80 3c 1a 43       	cmp    BYTE PTR [r10+r11*1],0x43
  518488:	74 16                	je     5184a0 <PRIVATE0000000000600d25+0x330>
  51848a:	f2 43 0f 10 0c d0    	movsd  xmm1,QWORD PTR [r8+r10*8]
  518490:	f2 42 0f 5c 0c d7    	subsd  xmm1,QWORD PTR [rdi+r10*8]
  518496:	66 0f 2f c8          	comisd xmm1,xmm0
  51849a:	0f 87 47 15 00 00    	ja     5199e7 <PRIVATE0000000000600d25+0x1877>
  5184a0:	49 ff c2             	inc    r10
  5184a3:	4c 3b d2             	cmp    r10,rdx
  5184a6:	7c db                	jl     518483 <PRIVATE0000000000600d25+0x313>
  5184a8:	33 d2                	xor    edx,edx
  5184aa:	85 c0                	test   eax,eax
  5184ac:	0f 8e 24 15 00 00    	jle    5199d6 <PRIVATE0000000000600d25+0x1866>
  5184b2:	89 0c 24             	mov    DWORD PTR [rsp],ecx
  5184b5:	4c 89 4c 24 10       	mov    QWORD PTR [rsp+0x10],r9
  5184ba:	44 89 ac 24 88 00 00 	mov    DWORD PTR [rsp+0x88],r13d
  5184c1:	00
  5184c2:	49 89 fd             	mov    r13,rdi
  5184c5:	4c 89 bc 24 80 00 00 	mov    QWORD PTR [rsp+0x80],r15
  5184cc:	00
  5184cd:	4c 89 f7             	mov    rdi,r14
  5184d0:	4c 89 64 24 08       	mov    QWORD PTR [rsp+0x8],r12
  5184d5:	41 89 c6             	mov    r14d,eax
  5184d8:	48 89 5c 24 78       	mov    QWORD PTR [rsp+0x78],rbx
  5184dd:	41 89 d4             	mov    r12d,edx
  5184e0:	89 f3                	mov    ebx,esi
  5184e2:	4d 89 c7             	mov    r15,r8
  5184e5:	44 89 e6             	mov    esi,r12d
  5184e8:	4c 89 ea             	mov    rdx,r13
  5184eb:	4c 89 f9             	mov    rcx,r15
  5184ee:	e8 0d 49 37 00       	call   88ce00 <PRIVATE0000000000a7e4de>
  5184f3:	85 c0                	test   eax,eax
  5184f5:	0f 84 ab 14 00 00    	je     5199a6 <PRIVATE0000000000600d25+0x1836>
  5184fb:	41 ff c4             	inc    r12d
  5184fe:	45 3b e6             	cmp    r12d,r14d
  518501:	7d 06                	jge    518509 <PRIVATE0000000000600d25+0x399>
  518503:	48 8b 7d 08          	mov    rdi,QWORD PTR [rbp+0x8]
  518507:	eb dc                	jmp    5184e5 <PRIVATE0000000000600d25+0x375>
  518509:	8b 0c 24             	mov    ecx,DWORD PTR [rsp]
  51850c:	89 de                	mov    esi,ebx
  51850e:	4c 8b 4c 24 10       	mov    r9,QWORD PTR [rsp+0x10]
  518513:	4c 89 ef             	mov    rdi,r13
  518516:	44 8b ac 24 88 00 00 	mov    r13d,DWORD PTR [rsp+0x88]
  51851d:	00
  51851e:	4d 89 f8             	mov    r8,r15
  518521:	4c 8b bc 24 80 00 00 	mov    r15,QWORD PTR [rsp+0x80]
  518528:	00
  518529:	4c 8b 64 24 08       	mov    r12,QWORD PTR [rsp+0x8]
  51852e:	48 8b 5c 24 78       	mov    rbx,QWORD PTR [rsp+0x78]
  518533:	45 33 f6             	xor    r14d,r14d
  518536:	85 c9                	test   ecx,ecx
  518538:	0f 8e 56 14 00 00    	jle    519994 <PRIVATE0000000000600d25+0x1824>
  51853e:	89 34 24             	mov    DWORD PTR [rsp],esi
  518541:	4c 89 4c 24 10       	mov    QWORD PTR [rsp+0x10],r9
  518546:	44 89 ac 24 88 00 00 	mov    DWORD PTR [rsp+0x88],r13d
  51854d:	00
  51854e:	49 89 fd             	mov    r13,rdi
  518551:	4c 89 bc 24 80 00 00 	mov    QWORD PTR [rsp+0x80],r15
  518558:	00
  518559:	4d 89 c7             	mov    r15,r8
  51855c:	4c 89 64 24 08       	mov    QWORD PTR [rsp+0x8],r12
  518561:	41 89 cc             	mov    r12d,ecx
  518564:	44 89 f6             	mov    esi,r14d
  518567:	4c 89 ea             	mov    rdx,r13
  51856a:	4c 89 f9             	mov    rcx,r15
  51856d:	48 8b 7d 08          	mov    rdi,QWORD PTR [rbp+0x8]
  518571:	e8 da 69 10 00       	call   61ef50 <PRIVATE000000000078dd89>
  518576:	85 c0                	test   eax,eax
  518578:	75 1d                	jne    518597 <PRIVATE0000000000600d25+0x427>
  51857a:	44 89 f6             	mov    esi,r14d
  51857d:	4c 89 ea             	mov    rdx,r13
  518580:	4c 89 f9             	mov    rcx,r15
  518583:	49 89 d8             	mov    r8,rbx
  518586:	48 8b 7d 08          	mov    rdi,QWORD PTR [rbp+0x8]
  51858a:	e8 91 67 10 00       	call   61ed20 <PRIVATE000000000078d933>
  51858f:	85 c0                	test   eax,eax
  518591:	0f 84 d7 13 00 00    	je     51996e <PRIVATE0000000000600d25+0x17fe>
  518597:	41 ff c6             	inc    r14d
  51859a:	45 3b f4             	cmp    r14d,r12d
  51859d:	7c c5                	jl     518564 <PRIVATE0000000000600d25+0x3f4>
  51859f:	8b 34 24             	mov    esi,DWORD PTR [rsp]
  5185a2:	4c 89 ef             	mov    rdi,r13
  5185a5:	4c 8b 4c 24 10       	mov    r9,QWORD PTR [rsp+0x10]
  5185aa:	4d 89 f8             	mov    r8,r15
  5185ad:	44 8b ac 24 88 00 00 	mov    r13d,DWORD PTR [rsp+0x88]
  5185b4:	00
  5185b5:	4c 8b bc 24 80 00 00 	mov    r15,QWORD PTR [rsp+0x80]
  5185bc:	00
  5185bd:	4c 8b 64 24 08       	mov    r12,QWORD PTR [rsp+0x8]
  5185c2:	45 33 f6             	xor    r14d,r14d
  5185c5:	33 c0                	xor    eax,eax
  5185c7:	85 f6                	test   esi,esi
  5185c9:	0f 8e 8d 13 00 00    	jle    51995c <PRIVATE0000000000600d25+0x17ec>
  5185cf:	89 34 24             	mov    DWORD PTR [rsp],esi
  5185d2:	44 89 ac 24 88 00 00 	mov    DWORD PTR [rsp+0x88],r13d
  5185d9:	00
  5185da:	49 89 fd             	mov    r13,rdi
  5185dd:	48 89 6c 24 38       	mov    QWORD PTR [rsp+0x38],rbp
  5185e2:	4c 89 cd             	mov    rbp,r9
  5185e5:	4c 89 bc 24 80 00 00 	mov    QWORD PTR [rsp+0x80],r15
  5185ec:	00
  5185ed:	4d 89 c7             	mov    r15,r8
  5185f0:	4c 89 64 24 08       	mov    QWORD PTR [rsp+0x8],r12
  5185f5:	49 89 c4             	mov    r12,rax
  5185f8:	4c 8b 54 24 18       	mov    r10,QWORD PTR [rsp+0x18]
  5185fd:	48 89 ef             	mov    rdi,rbp
  518600:	4c 89 ea             	mov    rdx,r13
  518603:	4c 89 f9             	mov    rcx,r15
  518606:	49 89 d8             	mov    r8,rbx
  518609:	4b 8b 34 e2          	mov    rsi,QWORD PTR [r10+r12*8]
  51860d:	e8 3e e0 42 00       	call   946650 <PRIVATE0000000000b3fb65>
  518612:	85 c0                	test   eax,eax
  518614:	0f 84 25 13 00 00    	je     51993f <PRIVATE0000000000600d25+0x17cf>
  51861a:	41 ff c6             	inc    r14d
  51861d:	49 ff c4             	inc    r12
  518620:	44 3b 34 24          	cmp    r14d,DWORD PTR [rsp]
  518624:	7c d2                	jl     5185f8 <PRIVATE0000000000600d25+0x488>
  518626:	44 8b ac 24 88 00 00 	mov    r13d,DWORD PTR [rsp+0x88]
  51862d:	00
  51862e:	48 8b 6c 24 38       	mov    rbp,QWORD PTR [rsp+0x38]
  518633:	4c 8b bc 24 80 00 00 	mov    r15,QWORD PTR [rsp+0x80]
  51863a:	00
  51863b:	4c 8b 64 24 08       	mov    r12,QWORD PTR [rsp+0x8]
  518640:	4c 8b 75 08          	mov    r14,QWORD PTR [rbp+0x8]
  518644:	49 8b 96 e8 00 00 00 	mov    rdx,QWORD PTR [r14+0xe8]
  51864b:	83 7a 18 00          	cmp    DWORD PTR [rdx+0x18],0x0
  51864f:	7e 13                	jle    518664 <PRIVATE0000000000600d25+0x4f4>
  518651:	83 7a 1c 00          	cmp    DWORD PTR [rdx+0x1c],0x0
  518655:	7e 0d                	jle    518664 <PRIVATE0000000000600d25+0x4f4>
  518657:	83 bd cc 34 00 00 01 	cmp    DWORD PTR [rbp+0x34cc],0x1
  51865e:	0f 84 ba 00 00 00    	je     51871e <PRIVATE0000000000600d25+0x5ae>
  518664:	49 8b 57 18          	mov    rdx,QWORD PTR [r15+0x18]
  518668:	c7 42 28 0b 00 00 00 	mov    DWORD PTR [rdx+0x28],0xb
  51866f:	e9 f8 fb ff ff       	jmp    51826c <PRIVATE0000000000600d25+0xfc>
  518674:	49 8b 57 18          	mov    rdx,QWORD PTR [r15+0x18]
  518678:	48 8b 8d 20 06 00 00 	mov    rcx,QWORD PTR [rbp+0x620]
  51867f:	c7 42 28 03 00 00 00 	mov    DWORD PTR [rdx+0x28],0x3
  518686:	ff 81 d4 00 00 00    	inc    DWORD PTR [rcx+0xd4]
  51868c:	e9 db fb ff ff       	jmp    51826c <PRIVATE0000000000600d25+0xfc>
  518691:	49 8b 57 18          	mov    rdx,QWORD PTR [r15+0x18]
  518695:	c7 42 28 04 00 00 00 	mov    DWORD PTR [rdx+0x28],0x4
  51869c:	48 8b 12             	mov    rdx,QWORD PTR [rdx]
  51869f:	48 3b 92 28 06 00 00 	cmp    rdx,QWORD PTR [rdx+0x628]
  5186a6:	0f 84 49 13 00 00    	je     5199f5 <PRIVATE0000000000600d25+0x1885>
  5186ac:	e9 bb fb ff ff       	jmp    51826c <PRIVATE0000000000600d25+0xfc>
  5186b1:	48 8d bc 24 c0 00 00 	lea    rdi,[rsp+0xc0]
  5186b8:	00
  5186b9:	e8 72 b5 bf ff       	call   113c30 <PRIVATE0000000000088c65>
  5186be:	e9 a9 fb ff ff       	jmp    51826c <PRIVATE0000000000600d25+0xfc>
  5186c3:	4c 3b ba 90 07 00 00 	cmp    r15,QWORD PTR [rdx+0x790]
  5186ca:	0f 85 9c fb ff ff    	jne    51826c <PRIVATE0000000000600d25+0xfc>
  5186d0:	48 8b 9a 20 06 00 00 	mov    rbx,QWORD PTR [rdx+0x620]
  5186d7:	48 89 df             	mov    rdi,rbx
  5186da:	e8 11 19 ff ff       	call   509ff0 <PRIVATE00000000005e7796>
  5186df:	0f 28 c8             	movaps xmm1,xmm0
  5186e2:	f2 0f 10 05 b6 3d 55 	movsd  xmm0,QWORD PTR [rip+0x2553db6]        # 2a6c4a0 <coefclasssign+0x1d20>
  5186e9:	02
  5186ea:	66 0f 2f c1          	comisd xmm0,xmm1
  5186ee:	76 08                	jbe    5186f8 <PRIVATE0000000000600d25+0x588>
  5186f0:	c7 03 02 00 00 00    	mov    DWORD PTR [rbx],0x2
  5186f6:	eb 06                	jmp    5186fe <PRIVATE0000000000600d25+0x58e>
  5186f8:	c7 03 03 00 00 00    	mov    DWORD PTR [rbx],0x3
  5186fe:	48 8b 93 80 07 00 00 	mov    rdx,QWORD PTR [rbx+0x780]
  518705:	f2 0f 10 82 68 01 00 	movsd  xmm0,QWORD PTR [rdx+0x168]
  51870c:	00
  51870d:	f2 0f 5f c1          	maxsd  xmm0,xmm1
  518711:	f2 0f 11 82 68 01 00 	movsd  QWORD PTR [rdx+0x168],xmm0
  518718:	00
  518719:	e9 4e fb ff ff       	jmp    51826c <PRIVATE0000000000600d25+0xfc>


libgurobi130.so:     file format elf64-x86-64


Disassembly of section .text:

000000000051993f <PRIVATE0000000000600d25+0x17cf>:
  51993f:	8b 34 24             	mov    esi,DWORD PTR [rsp]
  519942:	44 8b ac 24 88 00 00 	mov    r13d,DWORD PTR [rsp+0x88]
  519949:	00
  51994a:	48 8b 6c 24 38       	mov    rbp,QWORD PTR [rsp+0x38]
  51994f:	4c 8b bc 24 80 00 00 	mov    r15,QWORD PTR [rsp+0x80]
  519956:	00
  519957:	4c 8b 64 24 08       	mov    r12,QWORD PTR [rsp+0x8]
  51995c:	44 3b f6             	cmp    r14d,esi
  51995f:	0f 8d db ec ff ff    	jge    518640 <PRIVATE0000000000600d25+0x4d0>
  519965:	4c 8b 75 08          	mov    r14,QWORD PTR [rbp+0x8]
  519969:	e9 b0 ed ff ff       	jmp    51871e <PRIVATE0000000000600d25+0x5ae>
  51996e:	8b 34 24             	mov    esi,DWORD PTR [rsp]
  519971:	44 89 e1             	mov    ecx,r12d
  519974:	4c 8b 4c 24 10       	mov    r9,QWORD PTR [rsp+0x10]
  519979:	4c 89 ef             	mov    rdi,r13
  51997c:	44 8b ac 24 88 00 00 	mov    r13d,DWORD PTR [rsp+0x88]
  519983:	00
  519984:	4d 89 f8             	mov    r8,r15
  519987:	4c 8b bc 24 80 00 00 	mov    r15,QWORD PTR [rsp+0x80]
  51998e:	00
  51998f:	4c 8b 64 24 08       	mov    r12,QWORD PTR [rsp+0x8]
  519994:	44 3b f1             	cmp    r14d,ecx
  519997:	0f 8d 25 ec ff ff    	jge    5185c2 <PRIVATE0000000000600d25+0x452>
  51999d:	4c 8b 75 08          	mov    r14,QWORD PTR [rbp+0x8]
  5199a1:	e9 78 ed ff ff       	jmp    51871e <PRIVATE0000000000600d25+0x5ae>
  5199a6:	8b 0c 24             	mov    ecx,DWORD PTR [rsp]
  5199a9:	89 de                	mov    esi,ebx
  5199ab:	4c 8b 4c 24 10       	mov    r9,QWORD PTR [rsp+0x10]
  5199b0:	44 89 e2             	mov    edx,r12d
  5199b3:	4c 8b 64 24 08       	mov    r12,QWORD PTR [rsp+0x8]
  5199b8:	4c 89 ef             	mov    rdi,r13
  5199bb:	44 8b ac 24 88 00 00 	mov    r13d,DWORD PTR [rsp+0x88]
  5199c2:	00
  5199c3:	44 89 f0             	mov    eax,r14d
  5199c6:	48 8b 5c 24 78       	mov    rbx,QWORD PTR [rsp+0x78]
  5199cb:	4d 89 f8             	mov    r8,r15
  5199ce:	4c 8b bc 24 80 00 00 	mov    r15,QWORD PTR [rsp+0x80]
  5199d5:	00
  5199d6:	3b d0                	cmp    edx,eax
  5199d8:	0f 8d 55 eb ff ff    	jge    518533 <PRIVATE0000000000600d25+0x3c3>
  5199de:	4c 8b 75 08          	mov    r14,QWORD PTR [rbp+0x8]
  5199e2:	e9 37 ed ff ff       	jmp    51871e <PRIVATE0000000000600d25+0x5ae>
  5199e7:	4c 3b d2             	cmp    r10,rdx
  5199ea:	0f 8c 2e ed ff ff    	jl     51871e <PRIVATE0000000000600d25+0x5ae>
  5199f0:	e9 b3 ea ff ff       	jmp    5184a8 <PRIVATE0000000000600d25+0x338>
  5199f5:	4c 3b ba 90 07 00 00 	cmp    r15,QWORD PTR [rdx+0x790]
  5199fc:	0f 85 6a e8 ff ff    	jne    51826c <PRIVATE0000000000600d25+0xfc>
  519a02:	48 8b 9a 20 06 00 00 	mov    rbx,QWORD PTR [rdx+0x620]
  519a09:	48 89 df             	mov    rdi,rbx
  519a0c:	e8 df 05 ff ff       	call   509ff0 <PRIVATE00000000005e7796>
  519a11:	0f 28 c8             	movaps xmm1,xmm0
  519a14:	f2 0f 10 05 84 2a 55 	movsd  xmm0,QWORD PTR [rip+0x2552a84]        # 2a6c4a0 <coefclasssign+0x1d20>
  519a1b:	02
  519a1c:	66 0f 2f c1          	comisd xmm0,xmm1
  519a20:	76 08                	jbe    519a2a <PRIVATE0000000000600d25+0x18ba>
  519a22:	c7 03 02 00 00 00    	mov    DWORD PTR [rbx],0x2
  519a28:	eb 06                	jmp    519a30 <PRIVATE0000000000600d25+0x18c0>
  519a2a:	c7 03 03 00 00 00    	mov    DWORD PTR [rbx],0x3
  519a30:	48 8b 93 80 07 00 00 	mov    rdx,QWORD PTR [rbx+0x780]
  519a37:	f2 0f 10 82 68 01 00 	movsd  xmm0,QWORD PTR [rdx+0x168]
  519a3e:	00
  519a3f:	f2 0f 5f c1          	maxsd  xmm0,xmm1
  519a43:	f2 0f 11 82 68 01 00 	movsd  QWORD PTR [rdx+0x168],xmm0
  519a4a:	00
  519a4b:	e9 1c e8 ff ff       	jmp    51826c <PRIVATE0000000000600d25+0xfc>
  519a50:	83 7a 1c 00          	cmp    DWORD PTR [rdx+0x1c],0x0
  519a54:	0f 8f 90 e8 ff ff    	jg     5182ea <PRIVATE0000000000600d25+0x17a>
  519a5a:	83 ba 58 02 00 00 00 	cmp    DWORD PTR [rdx+0x258],0x0
  519a61:	0f 8f 83 e8 ff ff    	jg     5182ea <PRIVATE0000000000600d25+0x17a>
  519a67:	83 ba a8 01 00 00 00 	cmp    DWORD PTR [rdx+0x1a8],0x0
  519a6e:	0f 8e e3 e7 ff ff    	jle    518257 <PRIVATE0000000000600d25+0xe7>
  519a74:	e9 71 e8 ff ff       	jmp    5182ea <PRIVATE0000000000600d25+0x17a>
  519a79:	4c 3b bd 90 07 00 00 	cmp    r15,QWORD PTR [rbp+0x790]
  519a80:	0f 85 e6 e7 ff ff    	jne    51826c <PRIVATE0000000000600d25+0xfc>
  519a86:	48 8b 9d 20 06 00 00 	mov    rbx,QWORD PTR [rbp+0x620]
  519a8d:	48 89 df             	mov    rdi,rbx
  519a90:	e8 5b 05 ff ff       	call   509ff0 <PRIVATE00000000005e7796>
  519a95:	0f 28 c8             	movaps xmm1,xmm0
  519a98:	f2 0f 10 05 00 2a 55 	movsd  xmm0,QWORD PTR [rip+0x2552a00]        # 2a6c4a0 <coefclasssign+0x1d20>
  519a9f:	02
  519aa0:	66 0f 2f c1          	comisd xmm0,xmm1
  519aa4:	76 08                	jbe    519aae <PRIVATE0000000000600d25+0x193e>
  519aa6:	c7 03 02 00 00 00    	mov    DWORD PTR [rbx],0x2
  519aac:	eb 06                	jmp    519ab4 <PRIVATE0000000000600d25+0x1944>
  519aae:	c7 03 03 00 00 00    	mov    DWORD PTR [rbx],0x3
  519ab4:	48 8b 93 80 07 00 00 	mov    rdx,QWORD PTR [rbx+0x780]
  519abb:	f2 0f 10 82 68 01 00 	movsd  xmm0,QWORD PTR [rdx+0x168]
  519ac2:	00
  519ac3:	f2 0f 5f c1          	maxsd  xmm0,xmm1
  519ac7:	f2 0f 11 82 68 01 00 	movsd  QWORD PTR [rdx+0x168],xmm0
  519ace:	00
  519acf:	e9 98 e7 ff ff       	jmp    51826c <PRIVATE0000000000600d25+0xfc>
  519ad4:	83 be 80 00 00 00 00 	cmp    DWORD PTR [rsi+0x80],0x0
  519adb:	75 20                	jne    519afd <PRIVATE0000000000600d25+0x198d>
  519add:	48 83 be b0 00 00 00 	cmp    QWORD PTR [rsi+0xb0],0x0
  519ae4:	00
  519ae5:	75 16                	jne    519afd <PRIVATE0000000000600d25+0x198d>
  519ae7:	83 be b8 00 00 00 00 	cmp    DWORD PTR [rsi+0xb8],0x0
  519aee:	75 0d                	jne    519afd <PRIVATE0000000000600d25+0x198d>
  519af0:	41 c7 42 28 01 00 00 	mov    DWORD PTR [r10+0x28],0x1
  519af7:	00
  519af8:	e9 6f e7 ff ff       	jmp    51826c <PRIVATE0000000000600d25+0xfc>
  519afd:	33 c9                	xor    ecx,ecx
  519aff:	e9 a7 e6 ff ff       	jmp    5181ab <PRIVATE0000000000600d25+0x3b>
