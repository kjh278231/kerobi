NAME syn_round_pre
ROWS
 N  OBJ
 L  s       
 L  t       
COLUMNS
    MARKER    'MARKER'                 'INTORG'
    i0        OBJ       1
    i0        s         -1
    i0        t         1
    i1        OBJ       1
    i1        s         -1
    b0        OBJ       1
    b0        s         -1
    MARKER    'MARKER'                 'INTEND'
    i2        OBJ       1
    i2        s         -1
    MARKER    'MARKER'                 'INTORG'
    i3        OBJ       1
    i3        s         -1
    MARKER    'MARKER'                 'INTEND'
    c0        OBJ       1
    c0        s         -1
    c0        t         -1
RHS
    RHS1      s         -1
    RHS1      t         2
BOUNDS
 LO BND1      i0        1
 UP BND1      i0        4
 BV BND1      i1      
 BV BND1      b0      
 FX BND1      i2        2.000001
 MI BND1      i3      
 UP BND1      i3        4
 UP BND1      c0        3.3
ENDATA
